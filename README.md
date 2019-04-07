# Bob Nystrom's Baby's First Garbage Collector in Rust

A port of a simple GC to rust as an exercise in `unsafe` Rust.

You can find the original blog post [here][blog] and the source code in C [here][code].

[blog]: http://journal.stuffwithstuff.com/2013/12/08/babys-first-garbage-collector/
[code]: https://github.com/munificent/mark-sweep

## Building

You can build and test the project with [`cargo`][cargo], using

```sh
cargo build
cargo test
```
