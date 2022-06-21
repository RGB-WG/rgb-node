ARG BUILDER_DIR=/srv/rgb


FROM rust:1.59.0-slim-bullseye as builder

ARG SRC_DIR=/usr/local/src/rgb
ARG BUILDER_DIR

WORKDIR "$SRC_DIR"

COPY doc ${SRC_DIR}/doc
COPY shell ${SRC_DIR}/shell
COPY src ${SRC_DIR}/src
COPY build.rs Cargo.lock Cargo.toml codecov.yml config_spec.toml \
     LICENSE license_header README.md ${SRC_DIR}/

WORKDIR ${SRC_DIR}

RUN mkdir "${BUILDER_DIR}"

RUN cargo install --path . --root "${BUILDER_DIR}" --bins --all-features


FROM debian:bullseye-slim

ARG BUILDER_DIR
ARG BIN_DIR=/usr/local/bin
ARG DATA_DIR=/var/lib/rgb
ARG USER=rgb

RUN adduser --home "${DATA_DIR}" --shell /bin/bash --disabled-login \
        --gecos "${USER} user" ${USER}

COPY --from=builder --chown=${USER}:${USER} \
     "${BUILDER_DIR}/bin/" "${BIN_DIR}"

WORKDIR "${BIN_DIR}"
USER ${USER}

VOLUME "$DATA_DIR"

EXPOSE 63963

ENTRYPOINT ["rgbd"]

CMD ["-vvv", "--data-dir", "/var/lib/rgbd"]
