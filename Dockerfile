# syntax=docker/dockerfile:1
#
# Enable BuildKit before building this image:
# > export DOCKER_BUILDKIT=1
#
# To build the production image:
# > docker build -t anmolnetwork/anmol-node .

# Build the initial stage with all dependencies from base image
FROM anmolnetwork/anmol-node-build AS build

COPY chains ./chains
COPY common ./common
COPY node ./node
COPY pallets ./pallets
COPY runtime ./runtime
COPY Cargo.* .

ARG SCCACHE_BUCKET=
ENV SCCACHE_BUCKET=$SCCACHE_BUCKET

ARG AWS_ACCESS_KEY_ID=
ENV AWS_ACCESS_KEY_ID=$AWS_ACCESS_KEY_ID

ARG AWS_SECRET_ACCESS_KEY=
ENV AWS_SECRET_ACCESS_KEY=$AWS_SECRET_ACCESS_KEY

ARG PROFILE=release
ARG BUILD_ARGS=

RUN set -eux \
  && cargo build --$PROFILE --locked $BUILD_ARGS

# Builds the final production image
FROM debian:buster-slim AS production

EXPOSE 9615 9933 9944 30333

ARG USERNAME=anmol
ARG USER_UID=1000
ARG USER_GID=$USER_UID

RUN set -eux \
  && useradd -u ${USER_UID} -U -m -d /${USERNAME} -s /bin/sh ${USERNAME} \
  && mkdir -p /data \
  && chown -R ${USERNAME}:${USERNAME} /data \
	&& mkdir -p /${USERNAME}/.local/share \
	&& ln -s /data /${USERNAME}/.local/share/${USERNAME} \
	&& rm -rf /usr/bin /usr/sbin

USER ${USERNAME}

VOLUME [ "/data" ]

ENTRYPOINT [ "/usr/local/bin/anmol" ]

ARG PROFILE=release

COPY --from=build --chown=root:root /build/target/$PROFILE/anmol /usr/local/bin/anmol
