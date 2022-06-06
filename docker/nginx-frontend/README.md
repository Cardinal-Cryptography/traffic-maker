## Dockerized nginx config for traffic-maker monitoring frontend

Build docker image:

```bash
docker build --tag monitoring:latest -f ./docker/nginx-frontend/Dockerfile .
```

Run it:

```
docker-compose -f docker/nginx-frontend/docker-compose.yml up
```
