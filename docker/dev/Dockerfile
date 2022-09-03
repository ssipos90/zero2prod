FROM rust:slim

RUN set -eu; \
    apt-get update; \
    apt-get install -y \
      ca-certificates \
      clang \
      libssl-dev \
      lld \
      openssl \
      ;

RUN apt-get -y install --no-install-recommends librust-openssl-sys-dev

ARG UID=1000
ARG GID=1000

RUN set -eu; \
    groupadd -g $GID app; \
    useradd -u $UID -g $GID -m app

COPY ./entrypoint.sh /entrypoint.sh

ENV CARGO_HOME=/home/app/.cargo
USER app
RUN mkdir /home/app/app /home/app/.cargo
WORKDIR /home/app/app
VOLUME /home/app/.cargo

ENTRYPOINT ["/entrypoint.sh"]

CMD ["serve"]
