> **Note**:
> Development continues in [subsocial-parachain](https://github.com/dappforce/subsocial-parachain) repo.

# Subsocial Node by [DappForce](https://github.com/dappforce)

Subsocial is a set of Substrate pallets with web UI that allows anyone to launch their own decentralized censorship-resistant social network aka community. Every community can be a separate Substrate chain and connect with other communities via a Polkadot-based relay chain.

You can think of this as decentralized versions of Reddit, Stack Exchange or Medium, where subreddits or communities of Stack Exchange or blogs on Medium run on their own chain. At the same time, users of these decentralized communities should be able to share their reputation or transfer coins and other values from one community to another via Polkadot relay chain.

To learn more about Subsocial, please visit [Subsocial Network](http://subsocial.network).

## Build

### Build from scratch

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Initialize your Wasm Build environment:

```bash
./scripts/init.sh
```

Build Wasm and native code:

```bash
cargo build --release
```

### Build runtime WASM with SRTool

You need to have docker installed for this type of build.

Add SRTool alias:

```sh
export RUSTC_VERSION=nightly-2021-03-15; export PACKAGE=subsocial-runtime; alias srtool='docker run --rm -it -e RUNTIME_DIR=runtime -e PACKAGE=$PACKAGE -v $PWD:/build -v "$TMPDIR"/cargo:/cargo-home chevdor/srtool:$RUSTC_VERSION'
```

Run build:

```sh
srtool build
```

## Run

### Single Node Development Chain

Purge any existing developer chain state:

```bash
./target/release/subsocial-node purge-chain --dev
```

Start a development chain with:

```bash
./target/release/subsocial-node --dev
```

Detailed logs may be shown by running the node with the following environment variables set: `RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --dev`.

### Multi-Node Local Testnet

If you want to see the multi-node consensus algorithm in action locally, then you can create a local testnet with two validator nodes for Alice and Bob, who are the initial authorities of the genesis chain that have been endowed with testnet units.

Optionally, give each node a name and expose them so they are listed on the Polkadot [telemetry site](https://telemetry.polkadot.io/#/Local%20Testnet).

You'll need two terminal windows open.

We'll start Alice's substrate node first on default TCP port 30333 with her chain database stored locally at `/tmp/alice`. The bootnode ID of her node is `QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR`, which is generated from the `--node-key` value that we specify below:

```bash
cargo run -- \
  --base-path /tmp/alice \
  --chain=local \
  --alice \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --validator
```

In the second terminal, we'll start Bob's substrate node on a different TCP port of 30334, and with his chain database stored locally at `/tmp/bob`. We'll specify a value for the `--bootnodes` option that will connect his node to Alice's bootnode ID on TCP port 30333:

```bash
cargo run -- \
  --base-path /tmp/bob \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR \
  --chain=local \
  --bob \
  --port 30334 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --validator
```

Additional CLI usage options are available and may be shown by running `cargo run -- --help`.

## License

Subsocial is [GPL 3.0](./LICENSE) licensed.
