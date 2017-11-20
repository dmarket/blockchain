
# Dmarket Cryptocurrency Tutorial

Dmarket Blockchain test version uses [Exonum](https://github.com/exonum/exonum) framework(v0.2).

## Prerequisites

To run this example you need to install [Rust](https://www.rust-lang.org/en-US/)
compiler and [third-party libraries](http://exonum.com/doc/get-started/install/).

## Build & Run

### Blockchain Node

To build and run a single node use:

```sh
# clone the repository with blockchain node
git clone git@github.com:suntechsoft/dmarket-blockchain.git
cd dmarket-blockchain

# build and run
cargo run
```

Now the node is listening HTTP requests on `localhost:8000`.

### Sample Transactions & Read Requests

When node is launched, you can use transaction examples to check that it works properly.
A simplest way to do this is launching the [`test.sh`](examples/test.sh)
script in the **examples** directory. This script creates two wallets, performs a transfer
among them, and then verifies that the wallet status was correctly updated.

Alternatively, you may use command-line utilities, such as `curl`, to manually POST transactions
on [the transaction endpoint](http://127.0.0.1:8000/api/services/cryptocurrency/v1/wallets/transaction).

## License

DMarket Cryptocurrency is licensed under the MIT License . See [LICENSE](LICENSE) for details.
