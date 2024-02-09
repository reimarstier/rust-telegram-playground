cargo build --release

BUILDKIT_PROGRESS=plain docker compose -f docker/addfromhost/docker-compose.yml build
