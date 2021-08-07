FROM anmolnetwork/anmol-node-build-test AS build_stage
RUN cargo build --release

FROM alpine:3.14.0 AS prod
COPY --from=build_stage target/release/ target/release/.
CMD [ "cargo", "build", "--release" ]