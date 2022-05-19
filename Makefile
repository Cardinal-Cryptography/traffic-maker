.PHONY: setup run

run:
	cargo build --release
	target/release/bin

setup:
	cd set_up; cargo build --release
	set_up/target/release/set-up --config-file set_up/Config.toml