zigbuild:
	cargo zigbuild --release --target x86_64-unknown-linux-musl
	cargo zigbuild --release --target aarch64-unknown-linux-musl
	# cargo zigbuild --release --target universal2-apple-darwin
	# cargo zigbuild --release --target aarch64-apple-darwin
	# cargo zigbuild --release --target x86_64-apple-darwin
	# cargo zigbuild --release --target x86_64-pc-windows-gnu

release:
	@ mkdir -p dist
	@ cp -v target/x86_64-unknown-linux-musl/release/substrate-benchmark-machine ./dist/substrate-benchmark-machine-linux-amd64-static
	@ cp -v target/aarch64-unknown-linux-musl/release/substrate-benchmark-machine ./dist/substrate-benchmark-machine-linux-aarch64-static
	@ file dist/*
	@ du -sh dist/*
	@ zip -r dist.zip dist

nix:
	# sudo nix daemon
	# sudo nix build

sync:
	curl -sL https://raw.githubusercontent.com/paritytech/polkadot-sdk/master/substrate/utils/frame/benchmarking-cli/src/machine/reference_hardware.json > reference_hardware.json
