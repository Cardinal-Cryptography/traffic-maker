.PHONY: run build setup build-monitoring monitoring docker docker-stop

###############################################################################
# Local launching #############################################################
###############################################################################

run:
	cargo run --release

build:
	cargo build --release

setup:
	cd set_up; cargo run --release

build-monitoring:
	rustup target add wasm32-unknown-unknown
	cargo install --locked trunk
	cd monitoring; trunk build --release

monitoring: build-monitoring
	cd monitoring; trunk serve --open --release

###############################################################################
# Docker launching ############################################################
###############################################################################

docker: build
	cd monitoring; STATS_BASE_URL=http://backend:8080 make
	docker-compose -f docker/docker-compose.yml up -d

docker-stop:
	docker-compose -f docker/docker-compose.yml down -v
