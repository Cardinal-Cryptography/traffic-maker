FROM ubuntu:impish-20220128

RUN apt-get update -y && \
    apt-get install --no-install-recommends -y -q wget nginx libssl-dev ca-certificates curl && \
    apt-get clean && \
    rm -rf /tmp/* /var/{tmp,cache}/* /var/lib/{apt,dpkg}/

# replace nginx config
COPY docker/frontend/nginx.conf /etc/nginx/nginx.conf

# replace default server
COPY docker/frontend/default /etc/nginx/sites-available/default

# site config
COPY docker/frontend/traffic-maker.site /etc/nginx/sites-available/traffic-maker.site

# setup error page
COPY docker/frontend/X0X.html /usr/share/nginx/html/X0X.html

# setup site
RUN ln -s -f /etc/nginx/sites-available/traffic-maker.site /etc/nginx/sites-enabled/traffic-maker.site

# copy static content
COPY monitoring/dist/ /monitoring

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
