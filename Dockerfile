# syntax=docker/dockerfile:1
#
# Enable BuildKit before building this image:
# > export DOCKER_BUILDKIT=1
#
# To build the production image:
# > docker build \
#     --build-arg BUILD_DATE="$(date -u +'%Y-%m-%dT%H:%M:%SZ')" \
#     --build-arg GIT_SHORT_SHA="$(git rev-parse --short --verify HEAD)" \
#     --build-arg AWS_ACCESS_KEY_ID=$AWS_ACCESS_KEY_ID \
#     --build-arg AWS_SECRET_ACCESS_KEY=$AWS_SECRET_ACCESS_KEY \
#     -t anmolnetwork/anmol-node .

# Build the initial stage with all dependencies from base image
FROM anmolnetwork/anmol-node-build AS build

ARG AWS_ACCESS_KEY_ID
ENV AWS_ACCESS_KEY_ID=$AWS_ACCESS_KEY_ID

ARG AWS_SECRET_ACCESS_KEY
ENV AWS_SECRET_ACCESS_KEY=$AWS_SECRET_ACCESS_KEY

COPY common ./common
COPY node ./node
COPY pallets ./pallets
COPY runtime ./runtime
COPY Cargo.* .

ARG BUILD_ARGS=

RUN set -eux \
  && cargo build --release --locked $BUILD_ARGS \
  && mkdir -p /build/data

# Builds the final production image
FROM gcr.io/distroless/cc-debian10:nonroot AS production

LABEL maintainer="hello@anmol.network"
LABEL org.opencontainers.image.title="Anmol Node"
LABEL org.opencontainers.image.vendor="Anmol Network"
LABEL org.opencontainers.image.url="https://anmol.network"
LABEL org.opencontainers.image.source="https://github.com/anmolnetwork/anmol-node"
LABEL org.opencontainers.image.licenses="AGPL-3.0"

EXPOSE 9615 9933 9944 30333

VOLUME [ "/var/local/anmol" ]

ENTRYPOINT [ "/usr/local/bin/anmol", "--base-path", "/var/local/anmol" ]
CMD []

COPY --from=build --chown=nonroot:nonroot /build/data /var/local/anmol
COPY --from=build --chown=root:root /build/target/release/anmol /usr/local/bin/anmol

# --build-arg BUILD_DATE="$(date -u +'%Y-%m-%dT%H:%M:%SZ')"
ARG BUILD_DATE
LABEL org.opencontainers.image.created=$BUILD_DATE

# --build-arg GIT_SHORT_SHA="$(git rev-parse --short --verify HEAD)"
ARG GIT_SHORT_SHA
LABEL org.opencontainers.image.revision=$GIT_SHORT_SHA
