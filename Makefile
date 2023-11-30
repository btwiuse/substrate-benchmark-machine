musl:
	cargo build --release --target x86_64-unknown-linux-musl
	file ./target/x86_64-unknown-linux-musl/release/substrate-benchmark-machine

sync:
	curl -sL https://raw.githubusercontent.com/paritytech/polkadot-sdk/master/substrate/utils/frame/benchmarking-cli/src/machine/reference_hardware.json > reference_hardware.json
