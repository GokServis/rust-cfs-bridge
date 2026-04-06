# rust-cfs-bridge — convenience targets (Linux + Docker Compose with host network).

.PHONY: up up-cfs down logs-bridge logs-ui logs-cfs

up:
	docker compose up --build

up-cfs:
	docker compose --profile cfs up --build

down:
	docker compose --profile cfs down 2>/dev/null || docker compose down

logs-bridge:
	docker compose logs -f bridge-server

logs-ui:
	docker compose logs -f bridge-ui

logs-cfs:
	docker compose --profile cfs logs -f cfs
