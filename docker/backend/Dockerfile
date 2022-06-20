FROM ubuntu:impish-20220128

RUN apt-get update -y && \
    apt-get install -y -q libssl-dev ca-certificates curl && \
    apt-get clean && \
    rm -rf /tmp/* /var/{tmp,cache}/* /var/lib/{apt,dpkg}/

COPY target/release/bin /usr/local/bin/backend
RUN chmod +x /usr/local/bin/backend

EXPOSE 8080

ENTRYPOINT ["backend"]
