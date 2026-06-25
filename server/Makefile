.PHONY: all deploy logs stop clean

# The default target
all: deploy

# Deploy without SSL
deploy:
	@echo "🚀 Deploying LuminaRemote Signal Server (HTTP only)..."
	docker-compose up -d --build

# View logs for the signal server
logs:
	docker-compose logs -f signal-server

# View logs for all services
logs-all:
	docker-compose logs -f

# Stop all services gracefully
stop:
	docker-compose down

# Completely remove all containers and volumes
clean:
	docker-compose down -v
