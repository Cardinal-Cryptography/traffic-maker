.PHONY: setup run monitoring build-backup build-docker run-docker

run:
	cargo run --release

setup:
	cd set_up; cargo run --release

monitoring:
	rustup target add wasm32-unknown-unknown
	cargo install --locked trunk
	cd monitoring; trunk serve --open --release

build-backup:
	cargo build --release

build-docker: build-backup
	docker build --tag traffic-maker -f ./docker/Dockerfile .

run-docker: build-docker
	docker run \
		--network host \
		--mount type=bind,src=`pwd`/Timetable.toml,dst=/traffic-maker/Timetable.toml \
		traffic-maker
