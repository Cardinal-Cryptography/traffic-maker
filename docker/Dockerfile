# For 99.99% current image version is final, thus no tag (`latest`, by default).
FROM ghcr.io/cardinal-cryptography/rust-with-trunk

RUN apt-get update && apt-get install -y \
    libssl-dev ca-certificates \
    && rm -rf /tmp/* /var/{tmp,cache}/* /var/lib/{apt,dpkg}/

COPY target/release/bin /usr/local/bin/backend
RUN chmod +x /usr/local/bin/backend

COPY Timetable.toml /traffic-maker/
COPY monitoring/ /traffic-maker/
COPY frontend.tar.gz /traffic-maker/

COPY docker/docker_entrypoint.sh /traffic-maker/docker_entrypoint.sh
RUN chmod +x /traffic-maker/docker_entrypoint.sh

EXPOSE 8081

WORKDIR traffic-maker
ENTRYPOINT ["./docker_entrypoint.sh"]
