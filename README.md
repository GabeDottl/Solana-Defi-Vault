# Cove
Cove is a platform for Solana Vaults.

See src/instructions.rs for the API.

Within Yearn, the derivative of a X token would be yX - within Laguna Finance, we us lX. llX is used
for 2nd order derivatives.

## TODO
* Cleanup / merge the various Deposits & Withdraw logic
* Unit tests
* Security audit
* More example vaults
* Add fees
* Add governance? Or put this a level above.
* Allow multisig client wallets (i.e. support multiple signers)

### Environment Setup
1. Install Rust from https://rustup.rs/
2. Install Solana v1.6.2 or later from https://docs.solana.com/cli/install-solana-cli-tools#use-solanas-install-tool

### Build and test for program compiled natively
```
$ cargo build
$ cargo test
```

### Build and test the program compiled for BPF
```
$ cargo build-bpf
$ cargo test-bpf
```
