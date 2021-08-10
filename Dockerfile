# Build initial stage with all dependencies from base image
FROM anmolnetwork/anmol-node-build-test AS build_base_stage

# Second stage to build prod image
FROM debian:10.10-slim
WORKDIR /pkg
COPY --from=build_base_stage ./app .
CMD [ "./target/release/anmol"]
