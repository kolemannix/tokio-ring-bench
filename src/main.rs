use futures::lazy;
use tokio::prelude::*;
use tokio::prelude::future::Either;
use tokio::sync::mpsc;


/// Ring benchmark inspired by Programming Erlang: Software for a
/// Concurrent World, by Joe Armstrong, Chapter 8.11.2
///
/// "Write a ring benchmark. Create N processes in a ring. Send a
/// message round the ring M times so that a total of N * M messages
/// get sent. Time how long this takes for different values of N and M."

fn spawn_process(
    max: usize,
    name: String,
    rx: mpsc::Receiver<Option<usize>>,
    tx: mpsc::Sender<Option<usize>>,
    complete: mpsc::Sender<()>,
) -> impl Future<Item=(), Error=()> {
    rx
        .map_err(|_| ())
        .take_while(|msg| Ok(msg.is_some()))
        .fold(0, move |count, msg| {
            let to_send = if count == max {None} else {msg};
            tx.clone().send(to_send).map(move |_| count + 1).map_err(|_| ())
        }).and_then(move |_c| {
            // println!("{}: Finito. Forwarded {} messages", name, c);
            complete.send(()).map(|_| ()).map_err(|_| ())
        })
}


use std::env;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if  args.len() < 3 {
        print_usage_and_exit();
    }
    let n_nodes = if let Ok(arg_num_nodes) = args[1].parse::<usize>() {
        if arg_num_nodes <= 1 {
            eprintln!("Number of nodes must be > 1");
            ::std::process::exit(1);
        }
        arg_num_nodes
    } else {
        print_usage_and_exit();
    };

    let n_times = if let Ok(arg_ntimes) = args[2].parse::<usize>() {
        arg_ntimes
    } else {
        print_usage_and_exit()
    };

    println!("Setting up {} nodes for {} messages", n_nodes, n_times);

    tokio::run(lazy(move || {
        let (complete_tx, complete_rx) = mpsc::channel(n_times);
        let (root_tx, root_rx) = mpsc::channel(n_times);
        let mut procs = Vec::new();
        let mut i = 0;
        let mut prev_rx = root_rx;
        loop {
            let name = format!("n{}", i).to_string();
            if i == n_nodes - 1 {
                // The last process pushes to the ROOT process
                let p = spawn_process(n_times, name, prev_rx, root_tx.clone(), complete_tx.clone());
                procs.push(p);
                break;
            }
            let (tx, rx) = mpsc::channel(n_times);
            let p = spawn_process(n_times, name, prev_rx, tx, complete_tx.clone());
            prev_rx = rx;
            procs.push(p);
            i += 1;
        }
        for p in procs {
            tokio::spawn(p);
        }

        let root_send = root_tx.send(Some(1))
            .map(|_| ())
            .map_err(|_| ());
        let now = std::time::SystemTime::now();
        tokio::spawn(root_send);

        complete_rx.into_future().map(move |(a, _)| {
            if a.is_some() {
                println!("Received completion");
                match now.elapsed() {
                    Ok(elapsed) => println!(
                        "Time taken: {}.{:06} seconds",
                        elapsed.as_secs(),
                        elapsed.subsec_micros()
                    ),
                    Err(e) => println!("An error occurred: {:?}", e)
                }
            }
            if a.is_none() {
                eprintln!("Someone hung up");
            }
        }).map_err(|_| ())
    }));
}

fn print_usage_and_exit() -> ! {
    eprintln!("Usage; tokio-ring <num-nodes> <num-times-message-around-ring>");
    std::process::exit(1);
}
