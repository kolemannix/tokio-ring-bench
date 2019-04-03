# What it is

Ring benchmark inspired by Programming Erlang: Software for a
Concurrent World, by Joe Armstrong, Chapter 8.11.2

> Write a ring benchmark. Create N processes in a ring. Send a
message round the ring M times so that a total of N * M messages
get sent. Time how long this takes for different values of N and M.

# How to run it

Be sure to do a release build.
``` sh
$ cargo build --release
$ ./target/release/tokio-ring-bench 1000 1000
```

# Motivation

I'm in the process of learning Rust, and have a relatively large tokio-based project I'm working on, and felt I needed some more experience
with `tokio` itself. This was done as a sort of drill or practice. I'm extremely comfortable with Akka, and I was curious how performance compared
across Akka, Actix, and raw tokio.

# Results

Currently, with tokio 0.1.18, results on my machine hover around .65 to .8 seconds.

This is roughly equal to the results I've seen for an Actix implementation of the same test,
first surfaced [here](https://github.com/actix/actix/issues/52) and then canonized [here](https://github.com/actix/actix/pull/213).

I saw times as low as .35 seconds for those runs, though lately it appears there's a performance regression in Actix and
times are between 2 and 3 seconds.

I also implemented this in Akka (2.5.21) (TODO link to repo) and saw times averaging 0.6 seconds for the test. It's worth noting the Akka approach degrades as
we increase actors and decrease roundtrips, but this tokio approach performs almost identically with inputs (100, 10000), (1000, 1000), and (10000, 100).
In other words, using more tasks seems to add no overhead, but using more actors does, at least in the context of this silly benchmark.
