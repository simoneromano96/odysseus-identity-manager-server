FROM rustlang/rust@sha256:51a79952f457741d24adb6b3918349f7b5723930e16358635a5a185a628991cb as builder

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
