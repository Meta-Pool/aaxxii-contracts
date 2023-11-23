# aaxxii-contracts

<img src="media/mp-aaxxii.png" width="400">

## Introduction

This repository contains multiple contracts. Click on the contract to get more information:

- 💹 [Katherine Sale Contract](contracts/katherine-sale-contract/README.md)


## Build the contracts

All the contracts are located in the `./contracts/` directory. If you're going to work with the contracts, we recommend you to make it your home 🏠.

```sh
cd contracts/
```

A test build could be easily run with the following command:

```sh
cargo build
```

We facilitated some Makefile with the most common commands to help you. For example, a production ready release could be generated as follow:

```sh
make build
```

## Unit testing

Inside the `./contracts/` directory you could easily run some unit testing for the contracts.

```sh
cargo test
```

## Integration test

The integration tests are located outside of the `./contracts/` in the `./workspaces` directory, that makes reference to the `near_workspaces` framework used to test deployment and function call inside a local network.

Integration test could be run like this:

```sh
# Make sure to be under the ./workspace directory
cd workspace/

cargo run
```
