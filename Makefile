musl: musl-deps musl-amd64 musl-arm64

musl-deps:
	yay -S --needed --noconfirm musl aarch64-linux-musl-cross-bin

musl-amd64:
	cargo build --release --target x86_64-unknown-linux-musl
	file ./target/x86_64-unknown-linux-musl/release/substrate-benchmark-machine

musl-arm64:
	cargo build --release --target aarch64-unknown-linux-musl
	file ./target/aarch64-unknown-linux-musl/release/substrate-benchmark-machine

sync:
	curl -sL https://raw.githubusercontent.com/paritytech/polkadot-sdk/master/substrate/utils/frame/benchmarking-cli/src/machine/reference_hardware.json > reference_hardware.json
