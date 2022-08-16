.PHONY: run build setup build-monitoring monitoring docker docker-stop build-backend-docker build-frontend-docker

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
	cd monitoring; STATS_BASE_URL="https://traffic-maker.dev.azero.dev" trunk build --release -d dist-devnet
	cd monitoring; STATS_BASE_URL="https://traffic-maker.test.azero.dev" trunk build --release -d dist-testnet

monitoring: build-monitoring
	cd monitoring; trunk serve --open --release

###############################################################################
# Docker launching ############################################################
###############################################################################

docker: build build-monitoring
	docker-compose -f docker/docker-compose.yml up -d

docker-stop:
	docker-compose -f docker/docker-compose.yml down -v

build-backend-docker: build
	docker build --tag traffic-maker:latest -f docker/backend/Dockerfile .

build-frontend-docker: build-monitoring
	docker build --tag traffic-maker-monitoring:latest -f docker/frontend/Dockerfile .
