# Order Book
![example workflow](https://github.com/dorublanzeanu/order-book/actions/workflows/blank.yml/badge.svg)

This project implements a solution for a working order book.

An order book is a collection of buy and sell orders(`asks` and `bids`) that a broker keeps to facillitate the transactions for users.

## Project structure
```
- .github/workflows/blank.yml           - Github Actions description file - For CI on Github
- input                                 - File with input example
- src                                   - Sources directory
    - orderbook                         - OrderBook module implementation
        - mod.rs
    - main.rs                           - Program entry point - this program reads from input file and prints to stdout the expected results
- Cargo.toml                            - Cargo build dependency description file
- Dockerfile                            - Docker image build file - used to test/build in a containerized manned
- Readme.md                             - This file
- run.sh                                - Script to facillitate build/run inside container
```
## How to run
### Run locally
Dependencies:
- `Rust version 1.57`

The unit tests for this project can be compiled an run locally with:
```
# Build
$ cargo build

# Test
$ cargo test
```

The program that takes input from `./input/input.csv` can be run with:
```
# Build
$ cargo build

# Run
$ cargo run
```
Note: The program will wait for incoming orders indefinetly, as a normal broker does.
### Run in Docker Container
Use the `run.sh` script that uses `docker` to run an `ubuntu-20.04` container.
Dependencies:
- `Docker version 20.10.11, build dea9396`

The following commands will build a container image named `orderbook` and run a `orderbook_cont` container:
```
# Build Docker image
$ ./run.sh -b

# Run unit tests in Docker container
$ ./run.sh -r

# Or simpler - only runs the container
$ ./run.sh
```
### Automatically run with Github actions
The builds can be seen at https://github.com/dorublanzeanu/order-book/actions/
## Github
This project can be found at https://github.com/dorublanzeanu/order-book
