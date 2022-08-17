FROM nginx:1.23

ENV BACKEND_HTTP_HOSTPORT "backend:8080"
ENV MONITORING_ENV "default"

COPY docker/frontend/nginx.conf /etc/nginx/nginx.conf
COPY docker/frontend/traffic-maker.conf.template /etc/nginx/templates/traffic-make.conf.template
COPY docker/frontend/X0X.html /usr/share/nginx/html/X0X.html

COPY monitoring/dist/ /monitoring-default
COPY monitoring/dist-testnet/ /monitoring-testnet
COPY monitoring/dist-devnet/ /monitoring-devnet

