FROM paritytech/ci-linux:production as builder
LABEL description="This is the build stage for Subsocial. Here we build the Subsocial node binary."

WORKDIR /subsocial

COPY . /subsocial

RUN cargo build --release

FROM debian:buster-slim
LABEL description="This is the 2nd stage. Here we assembly a very small docker image where we copy the Subsocial binary."
COPY --from=builder /subsocial/target/release/subsocial-node /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /subsocial subsocial && \
	mkdir -p /subsocial/.local/share && \
	mkdir /data && \
	chown -R subsocial:subsocial /data && \
	ln -s /data /subsocial/.local/share/subsocial-node && \
	rm -rf /usr/bin /usr/sbin

USER subsocial
EXPOSE 30333 9933 9944
VOLUME ["/data"]

CMD ["/usr/local/bin/subsocial-node"]
