FROM rust:1.88 AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release

FROM ubuntu:22.04

RUN apt-get update && apt-get upgrade -y && apt-get install -y libssl3 python3 lua5.4 clang openjdk-17-jdk scala groovy ruby-full perl ghc curl gpg && rm -rf /var/lib/apt/lists/*

# crystal
RUN curl -fsSL https://crystal-lang.org/install.sh | bash

# dart
RUN apt-get update && apt-get install apt-transport-https && wget -qO- https://dl-ssl.google.com/linux/linux_signing_key.pub | gpg  --dearmor -o /usr/share/keyrings/dart.gpg
RUN echo 'deb [signed-by=/usr/share/keyrings/dart.gpg arch=amd64] https://storage.googleapis.com/download.dartlang.org/linux/debian stable main' | tee /etc/apt/sources.list.d/dart_stable.list
RUN apt-get update && apt-get install dart

# julia
RUN curl -fsSL https://install.julialang.org | sh -s -- -y

# bun
RUN curl -fsSL https://bun.com/install | bash

# zig
ENV ZIG_VERSION=0.13.0
RUN curl -L https://ziglang.org/download/${ZIG_VERSION}/zig-linux-x86_64-${ZIG_VERSION}.tar.xz \
    | tar -xJ && \
    mv zig-linux-x86_64-${ZIG_VERSION} /opt/zig

# Add Zig to PATH
ENV PATH="/opt/zig:${PATH}"

COPY --from=builder /app/target/release/comphub /usr/local/bin/comphub
CMD ["comphub"]
