# Build initial stage with all dependencies from build image
FROM anmolnetwork/anmol-node-build-test AS build_stage

# Second stage to build prod image
FROM alpine:3.14.0 AS prod
WORKDIR /pkg
COPY --from=build_stage . .
CMD [ "./target/release/anmol"]