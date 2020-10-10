ARG BUILDER_DIR=/srv/rgb


FROM rustlang/rust:nightly-slim as builder

ARG SRC_DIR=/usr/local/src/rgb
ARG BUILDER_DIR

RUN apt-get update \
    && apt-get -y install --no-install-recommends \
        build-essential cmake libpq-dev libssl-dev libzmq3-dev pkg-config

WORKDIR "$SRC_DIR"

COPY src src
COPY Cargo.lock Cargo.toml README.md ./

RUN cargo install --path . --root "${BUILDER_DIR}"


FROM debian:buster-slim

ARG BUILDER_DIR
ARG BIN_DIR=/usr/local/bin
ARG DATA_DIR=/var/lib/rgb
ARG USER=rgbd

RUN apt-get update \
    && apt-get -y install --no-install-recommends \
       libssl1.1 \
       tini \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

RUN adduser --home "${DATA_DIR}" --shell /bin/bash --disabled-login \
        --gecos "${USER} user" ${USER}

COPY --from=builder --chown=${USER}:${USER} \
     "${BUILDER_DIR}/bin/" "${BIN_DIR}"

WORKDIR "${BIN_DIR}"
USER ${USER}

VOLUME "$DATA_DIR"

ENTRYPOINT ["/usr/bin/tini", "-g", "--", "/usr/local/bin/rgbd", \
			"--bin-dir", "/usr/local/bin", "--data-dir", "/var/lib/rgb"]

CMD ["-vvvv"]
