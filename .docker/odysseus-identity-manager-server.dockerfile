FROM rustlang/rust@sha256:ad43faa521e7982d7786b43f7afd1bd469dbaf9d9c9148ec18417cb34b0d21b3 as builder

# Enable cpu native optimizations
ENV RUSTFLAGS="-C target-cpu=native"

WORKDIR /usr/src/odysseus-identity-manager

COPY . .

RUN cargo install --path .

FROM debian:stable-slim as production

# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*

WORKDIR /odysseus-identity-manager

COPY --from=builder /usr/local/cargo/bin/odysseus-identity-manager /odysseus-identity-manager

COPY ./environments/ /odysseus-identity-manager/environments/

CMD ["/odysseus-identity-manager/odysseus-identity-manager"]
