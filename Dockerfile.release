# FORK from: https://github.com/paritytech/polkadot/blob/master/docker/Dockerfile
FROM paritytech/ci-linux:production as builder
LABEL description="This is the build stage for Parallel. Here we create the binary."

ARG PROFILE=release
WORKDIR /parallel

COPY . /parallel

RUN cargo build --$PROFILE

# ===== SECOND STAGE ======

FROM debian:buster-slim
LABEL description="This is the 2nd stage: a very small image where we copy the Parallel binary."
ARG PROFILE=release
COPY --from=builder /parallel/target/$PROFILE/parallel /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /parallel parallel && \
	mkdir -p /parallel/.local/share && \
	mkdir /data && \
	chown -R parallel:parallel /data && \
	ln -s /data /parallel/.local/share/parallel && \
	rm -rf /usr/bin /usr/sbin

COPY --from=builder /parallel/resources/rococo_local.json /parallel

USER parallel
EXPOSE 30333 9933 9944
VOLUME ["/data"]

CMD ["/usr/local/bin/parallel"]
