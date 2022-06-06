## Dockerized nginx config for traffic-maker monitoring frontend

Build WASM & other static content artifacts:

```bash
cd monitoring
STATS_BASE_URL=http://127.0.0.1:8080 trunk build --release
```

- where `STATS_BASE_URL` is where the backend server will be listening for API requests.

Build docker image:

```bash
docker build --tag monitoring:latest -f ./docker/nginx-frontend/Dockerfile .
```

Run it:

```
docker-compose -f docker/nginx-frontend/docker-compose.yml up
```

Visit `localhost:8040` where the content is being served.
