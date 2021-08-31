FROM rustlang/rust@sha256:ac9481b8dbb515bc957602843a1f9e8aa82fc90aef65c61b6448065575721254 as builder

# Enable cpu native optimizations
ENV RUSTFLAGS="-C target-cpu=native"

WORKDIR /usr/src/odysseus-identity-manager

COPY . .

RUN cargo install --path .

FROM debian:stable-slim as production

# RUN apt-get update && \
#   apt-get install -y  && \ 
#   rm -rf /var/lib/apt/lists/*

WORKDIR /odysseus-identity-manager

COPY --from=builder /usr/local/cargo/bin/odysseus-identity-manager /odysseus-identity-manager

COPY ./environments/ /odysseus-identity-manager/environments/

COPY ./src/templates/ /odysseus-identity-manager/templates/

ENV APP_TEMPLATE_PATH=/odysseus-identity-manager/templates/

CMD ["/odysseus-identity-manager/odysseus-identity-manager"]
