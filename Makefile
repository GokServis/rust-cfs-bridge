# rust-cfs-bridge — convenience targets (Linux + Docker Compose with host network).
#
# up      — bridge-server + nginx only (no cFS). Uplink UDP to CI_LAB (:1234) fails until cFS runs.
# up-cfs  — same + core-cpu1 (CI_LAB / TO_LAB / bridge_reader).

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
