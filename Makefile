.PHONY: run build setup build-docker run-docker build-monitoring monitoring build-monitoring-docker monitoring-docker

run:
	cargo run --release

build:
	cargo build --release

setup:
	cd set_up; cargo run --release

build-docker: build
	docker build --tag traffic-maker -f ./docker/backend/Dockerfile .

run-docker: build-docker
	docker run \
		--network host \
		--mount type=bind,src=`pwd`/Timetable.toml,dst=/traffic-maker/Timetable.toml \
		--name traffic-maker \
		traffic-maker

build-monitoring:
	rustup target add wasm32-unknown-unknown
	cargo install --locked trunk
	cd monitoring; STATS_BASE_URL=http://127.0.0.1:8080 trunk build --release

monitoring: build-monitoring
	cd monitoring; trunk serve --open --release

build-monitoring-docker: build-monitoring
	docker build --tag traffic-maker-monitoring -f ./docker/frontend/Dockerfile .

monitoring-docker: build-monitoring-docker
	docker run \
    		--name traffic-maker-monitoring \
    		-p 8081:80 \
    		-p 8080:8080 \
    		traffic-maker-monitoring
