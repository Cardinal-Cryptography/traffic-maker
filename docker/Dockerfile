FROM rust:1.58

RUN apt-get update && apt-get install -y \
    libssl-dev ca-certificates \
    && rm -rf /tmp/* /var/{tmp,cache}/* /var/lib/{apt,dpkg}/

RUN cargo install --locked trunk && \
    rustup target add wasm32-unknown-unknown

COPY target/release/bin /usr/local/bin/backend
RUN chmod +x /usr/local/bin/backend

COPY Timetable.toml /traffic-maker/
COPY monitoring/ /traffic-maker/

COPY docker/docker_entrypoint.sh /traffic-maker/docker_entrypoint.sh
RUN chmod +x /traffic-maker/docker_entrypoint.sh

EXPOSE 8081

WORKDIR traffic-maker
ENTRYPOINT ["./docker_entrypoint.sh"]
