build:
	cargo build

build-encoder:
	cargo build -p encoder-firmware --target thumbv6m-none-eabi

run:
	cd encoder-firmware && cargo run
