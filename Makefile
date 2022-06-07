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
	cd monitoring; ${STATS_BASE_URL:-http://127.0.0.1:8080} trunk build --release

monitoring: build-monitoring
	cd monitoring; trunk serve --open --release

###############################################################################
# Docker launching ############################################################
###############################################################################

docker: build
	@STATS_BASE_URL=backend:8080 make build-monitoring
	docker-compose -f docker/docker-compose.yml up -d

docker-stop:
	docker-compose -f docker/docker-compose.yml down -v
