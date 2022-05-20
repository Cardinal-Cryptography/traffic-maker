.PHONY: setup run monitoring

run:
	cargo run --release

setup:
	cd set_up; cargo run --release

monitoring:
	rustup target add wasm32-unknown-unknown
	cargo install --locked trunk
	cd monitoring; trunk serve --open
