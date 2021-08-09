FROM anmolnetwork/anmol-node-build-test AS build_stage

FROM alpine:3.14.0 AS prod
WORKDIR /pkg
COPY --from=build_stage . .
CMD [ "./target/release/anmol"]