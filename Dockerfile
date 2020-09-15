FROM rustlang/rust:nightly-slim as builder

RUN apt-get update \
    && apt-get -y install --no-install-recommends \
        build-essential cmake libpq-dev libssl-dev libzmq3-dev pkg-config

WORKDIR /srv/app

COPY ffi ffi
COPY src src
COPY Cargo.lock Cargo.toml README.md ./

RUN cargo build --release


FROM debian:buster-slim

ENV APP_DIR=/srv/app USER=rgbnode

RUN adduser --home ${APP_DIR} --shell /bin/bash --disabled-login \
        --gecos "${USER} user" ${USER}

COPY --from=builder --chown=${USER}:${USER} \
        ${APP_DIR}/target/release/ /usr/local/bin/

RUN apt-get update \
    && apt-get -y install --no-install-recommends \
        libssl1.1 tini \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

WORKDIR ${APP_DIR}
USER ${USER}

RUN mkdir data

VOLUME ["${APP_DIR}/data"]
ENTRYPOINT ["/usr/bin/tini", "-g", "--", "/usr/local/bin/rgbd", \
			"--bin-dir", "/usr/local/bin", "--data-dir", "./data"]
CMD ["-vvvv"]
