January 2022, Author's note: This code was created initially concieved as an SSI platform on Solana (similar to Civic.com) - see the older commits. It evolved into a defi Vault application for the Solana Season hackathon Summer 2021 before being abandoned. It's now open-source, closed license to serve as an example (and perhaps platform) for creating new Solana Defi projects.

Hack & Fork away :) - if you want to use the code commercially, I'm happy to adapt/rewrite it to meet your Solana defi product's needs - let me know what you want built, and I'll give you a quote for code + license agreement. Email: gabe@hearttoken.com

Tags: Yearn Ribbon Finance Maker Maple Finance Curve Finance Aave Compound lending protocol solana rust Instadapp Uniswap etc.

# Cove
Cove was a platform for Solana Yearn-like Vaults.

See src/instructions.rs for the API.

Within Yearn, the derivative of a X token would be yX - within Laguna Finance, we us lX. llX is used
for 2nd order derivatives.

Laguna Vaults are intended to be combined to form directed, acyclic investment graphs of arbitrary
complexity. Fees may be charged and routed at any level of the graph. In the future, the graph
will be able to react to arbitrary events.

The core benefit of Laguna Vaults are that they mint & distribute a derivative token to users when
depositing proportional to a best-estimate of their contribution to the current underlying value.
This makes it trivial, for example, to create arbitrary wrapper-tokens (like stETH, wETH).

This system is implemented with both functional and unit-tests and those are the best mechanism for understanding and verifying functionality.

The frontend was partially hacked together from another Solana project but was never completed and is still(?) private source.

## TODO
* TODO(003): Implement strategy withdraw/deposit
* TODO(004): Add functional test for Vault using another Vault as a Strategy.
* Add Peek function to strategy to see underlying value.
* TODO(001): Grant prportional lX tokens when depositing
* TODO(002): Charge lX tokens when withdrawing
* Add Multplexer for splitting tokens across multiple strategies (e.g. hodl & other)
* Add fee support
* Allow multisig client wallets (i.e. support multiple signers)
* Add reporting for calculating yield
* Add support for governance? Might implement above & separate
* Add Tend API for triggering harvesting (or other logic) across the graph on a periodic basis
* Unit tests
* Expand functional tests to include bad cases
* Security audit
* More example vaults
* Cleanup / merge the various Deposits & Withdraw logic


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
