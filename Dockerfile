FROM ubuntu:18.04

COPY .cargo/parallel /usr/local/bin

RUN apt update --fix-missing \
    && apt install -y \
        sudo

RUN useradd -m -u 1000 -U -s /bin/sh -d /parallel parallel && \
    usermod -aG sudo parallel && \
	mkdir -p /parallel/.local/share && \
	mkdir /data && \
	chown -R parallel:parallel /data && \
	ln -s /data /parallel/.local/share/parallel

RUN echo '%sudo   ALL=(ALL:ALL) NOPASSWD:ALL' >> /etc/sudoers

COPY ./resources/rococo_local.json /parallel

USER parallel
EXPOSE 30333 9933 9944
VOLUME ["/data"]

CMD ["/usr/local/bin/parallel"]
