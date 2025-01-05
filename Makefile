build:
	cross build --release --target aarch64-unknown-linux-gnu

test:
	cargo test --verbose

clean:
	cargo clean

scp: build
	scp ./target/aarch64-unknown-linux-gnu/release/rpi-plant-moisture cole@pi-callie.local:~/rpi-plant-moisture

.PHONY: build test clean scp
