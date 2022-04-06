# This image is currently available at `ghcr.io/cardinal-cryptography/rust-with-trunk`.
# Thanks to that, we do not need to install `trunk` every time (it lasts ~few minutes).

FROM rust:1.58

RUN apt-get update && apt-get install -y \
    libssl-dev ca-certificates \
    && rm -rf /tmp/* /var/{tmp,cache}/* /var/lib/{apt,dpkg}/

RUN cargo install --locked trunk && \
    rustup target add wasm32-unknown-unknown
