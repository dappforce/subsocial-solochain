FROM dappforce/cargo-chef:latest  AS chef
WORKDIR /subsocial

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /subsocial/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release

FROM debian:buster-slim
COPY --from=builder /subsocial/target/release/subsocial-node /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /subsocial subsocial && \
        apt update && apt install curl -y && \
	mkdir -p /subsocial/.local/share && \
	mkdir /data && \
	chown -R subsocial:subsocial /data && \
	ln -s /data /subsocial/.local/share/subsocial-node

USER subsocial
EXPOSE 30333 9933 9944
VOLUME ["/data"]

CMD ["/usr/local/bin/subsocial-node"]
