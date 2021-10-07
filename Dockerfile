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

ARG BUILD_ARGS=

RUN set -eux \
  && cargo build --release --locked $BUILD_ARGS \
  && mkdir -p /tmp/anmol-data

# Builds the final production image
FROM gcr.io/distroless/cc-debian10:nonroot AS production

EXPOSE 9615 9933 9944 30333

VOLUME [ "/var/local/anmol" ]

ENTRYPOINT [ "/usr/local/bin/anmol", "--base-path", "/var/local/anmol" ]
CMD []

COPY --from=build --chown=nonroot:nonroot /tmp/anmol-data /var/local/anmol
COPY --from=build --chown=nonroot:nonroot /build/chains /var/local/anmol/specs
COPY --from=build --chown=root:root /build/target/release/anmol /usr/local/bin/anmol
