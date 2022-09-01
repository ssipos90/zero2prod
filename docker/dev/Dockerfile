FROM rust:slim

RUN set -eu; \
    apt-get update; \
    apt-get install -y \
      clang \
      lld \
      ca-certificates \
      openssl;

ARG UID=1000
ARG GID=1000

RUN set -eu; \
    useradd -u $UID -g $GID -U -m app; \
    mkdir /app; \
    chown app:app /app

USER app
WORKDIR /app

CMD ["cargo", "watch", "--watch-when-idle", "-x", "test -- --nocapture"]
