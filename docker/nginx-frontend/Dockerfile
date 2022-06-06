FROM debian:stretch-slim
MAINTAINER "Filip Bielejec" <filip.bielejec@cardinals.cc>

RUN apt-get update -y \
        && apt-get install --no-install-recommends -y \
        -q wget nginx

RUN apt-get clean && \
        rm -rf /var/lib/apt/lists/*

# replace nginx config
COPY docker/nginx-frontend/nginx.conf /etc/nginx/nginx.conf

# replace default server
COPY docker/nginx-frontend/default /etc/nginx/sites-available/default

# site config
COPY docker/nginx-frontend/traffic-maker.site /etc/nginx/sites-available/traffic-maker.site

# setup error page
COPY docker/nginx-frontend/X0X.html /usr/share/nginx/html/X0X.html

# setup site
RUN ln -s -f /etc/nginx/sites-available/traffic-maker.site /etc/nginx/sites-enabled/traffic-maker.site

# copy static content
COPY monitoring/dist/ /monitoring

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
