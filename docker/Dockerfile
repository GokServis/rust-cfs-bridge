FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
ENV SIMULATION=native
ENV OMIT_DEPRECATED=true
ENV BRIDGE_TLM_BIND=127.0.0.1:2234

RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    git \
    python3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

COPY . .

# NOTE: `sch_lab` schedule table is maintained in-tree (no patch step).

RUN chmod +x /app/docker/entrypoint.sh /app/docker/entrypoint-bridge.sh /app/docker/entrypoint-cfs.sh

# `make prep` regenerates cfs/sample_defs; keep mission cpu1_cfe_es_startup.scr in sync with cfe/cmake/sample_defs.
RUN cd cfs && \
    cp ./cfe/cmake/Makefile.sample Makefile && \
    cp -r ./cfe/cmake/sample_defs sample_defs && \
    make BUILDTYPE=release prep && \
    cp ./cfe/cmake/sample_defs/cpu1_cfe_es_startup.scr ./sample_defs/cpu1_cfe_es_startup.scr && \
    make && \
    make install && \
    cp ./cfe/cmake/sample_defs/cpu1_cfe_es_startup.scr ./build/exe/cpu1/cf/cfe_es_startup.scr

# Web UI (served by bridge-server from /app/bridge-ui/dist).
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
    apt-get install -y nodejs && \
    rm -rf /var/lib/apt/lists/* && \
    cd /app/bridge-ui && npm ci && npm run build

RUN cd rust-bridge && cargo build --release

ENTRYPOINT ["/app/docker/entrypoint.sh"]
