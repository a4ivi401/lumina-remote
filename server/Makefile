.PHONY: all deploy logs stop clean

# The default target
all: deploy

# Initialize Let's Encrypt certificates and start the entire stack
deploy:
	@echo "🚀 Deploying LuminaRemote Infrastructure..."
	@chmod +x ./init-letsencrypt.sh
	@./init-letsencrypt.sh

# View logs for the signal server
logs:
	docker-compose logs -f signal-server

# View logs for all services (Nginx, Certbot, Signal Server)
logs-all:
	docker-compose logs -f

# Stop all services gracefully
stop:
	docker-compose down

# Completely remove all containers, networks, and generated certificates
clean:
	docker-compose down -v
	sudo rm -rf ./certbot
