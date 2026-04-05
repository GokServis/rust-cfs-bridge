FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
ENV SIMULATION=native
ENV OMIT_DEPRECATED=true

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

RUN chmod +x /app/docker/entrypoint.sh

RUN cd cfs && \
    cp ./cfe/cmake/Makefile.sample Makefile && \
    cp -r ./cfe/cmake/sample_defs sample_defs && \
    make BUILDTYPE=release prep && \
    make && \
    make install

RUN cd rust-bridge && cargo build --release

ENTRYPOINT ["/app/docker/entrypoint.sh"]
