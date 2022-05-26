FROM rust:1.58

RUN apt-get update && apt-get install -y \
    libssl-dev ca-certificates \
    && rm -rf /tmp/* /var/{tmp,cache}/* /var/lib/{apt,dpkg}/

COPY target/release/bin /usr/local/bin/backend
RUN chmod +x /usr/local/bin/backend

COPY docker/docker_entrypoint.sh /traffic-maker/docker_entrypoint.sh
RUN chmod +x /traffic-maker/docker_entrypoint.sh

EXPOSE 8080

WORKDIR traffic-maker
ENTRYPOINT ["./docker_entrypoint.sh"]
