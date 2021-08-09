# Build initial stage with all dependencies from base image
FROM anmolnetwork/anmol-node-build-test AS build_base_stage

# Second stage to build prod image
FROM alpine:3.14.0
WORKDIR /pkg
COPY --from=build_base_stage . .
CMD [ "./target/release/anmol"]
