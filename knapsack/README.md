# Knapsack

This project contains a series of algorithms for finding solutions to the knapsack problem. The algorithms are written in [Rust](https://rust-lang.org), and I wrote a script to generate knapsacks in python.

## Installation/Setup

If you have the rust toolchain installed, you should be able to navigate to this directory and type:

```
cargo run -- -h
```

This will show you the help page for the various options that are supported. The `--` is important, it signifies that you're no longer supplying arguments for `cargo`, but for the program `cargo` is running. You can also build the project for better performance and not worry about the `--` with this command:

```
cargo build --release
```

The resulting binary will be located in ../targets/release/
