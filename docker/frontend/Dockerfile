FROM rust:1.58

RUN apt-get update && \
    apt-get install -y libssl-dev ca-certificates curl wget && \
    apt-get clean && \
    rm -rf /tmp/* /var/{tmp,cache}/* /var/lib/{apt,dpkg}/

EXPOSE 8081

RUN wget -qO- https://github.com/thedodd/trunk/releases/download/v0.15.0/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf-

COPY monitoring/ ./monitoring
COPY common/ ./common

WORKDIR ./monitoring/
RUN rustup target add wasm32-unknown-unknown && \
    ../trunk build --release

ENTRYPOINT ["../trunk", "serve", "--release"]
