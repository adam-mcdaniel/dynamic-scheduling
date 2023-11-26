# Programming Assignment \#2

### **Tomasulo's Dynamic Scheduling Algorithm**

![Tomasulo's Algorithm](image.png)

***COSC 530 - Adam McDaniel***
---

## Overview

My program completely matches the short trace, but my implementation seems to issue a store instruction one cycle earlier than the reference implementation, which causes mine to be slightly off. It performs the all the main logic for the scheduling, but I seem to have a slight difference with the reference in an edge case or two.

You can use my Makefile to compare my output against the reference. Pass the trace to test with using `trace=` on the command line.

```bash
$ make trace=trace.dat
Running mine...
Running reference...
0 lines differ in outputs
```

```bash
$ make trace=trace2.dat
Running mine...
Running reference...
61 lines differ in outputs
```

## Usage

To build my program, use the Rust package manager: `cargo` (🚀blazingly fast🚀 [(except 🚀🚀compile🚀 time🚀)](https://www.wikihow.com/Kill-Time)).

```bash
$ # Compile the simulator in release mode
$ cargo build --release
$ # You can use this command to copy the compiled executable to the working directory, if you want.
$ cp target/release/tomasulos .
```

You can run my program by passing the trace file by piping/redirecting through standard input.

Or type it by hand. You won't.

```bash
$ # Compile the simulator in release mode
$ cargo build --release
$ # Use STDIN to supply the trace like the reference executable
$ ./target/release/tomasulos < trace.dat > output.txt
```

#### Logging

![Bad Code](image-1.png)

If you want to run with logging, use the `RUST_LOG` environment variable. You can choose from `info`, `debug`, or `trace` log levels for increasing verbosity.

```bash
$ # Run the program with `trace` log-level
$ RUST_LOG=trace ./target/release/tomasulos < trace.dat > output.txt
```