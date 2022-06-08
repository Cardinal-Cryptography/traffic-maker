.PHONY: build-dist

build-dist:
	rm -rf dist/
	cargo build --release --target wasm32-unknown-unknown
	cargo install wasm-bindgen-cli
	wasm-bindgen target/wasm32-unknown-unknown/release/traffic-maker-monitoring.wasm --out-dir dist --web
	cp aleph-style.scss dist/aleph-style.scss
	cp no-trunk-index.html dist/index.html
