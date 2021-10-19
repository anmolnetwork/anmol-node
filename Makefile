.PHONY: run run-ibtida format-code benchmarks-build benchmarks-generate test

run:
	cargo run -- --dev --tmp

run-ibtida:
	docker-compose -f docker-compose.ibtida.yml down
	scripts/generate-keys.sh
	scripts/generate-chain-spec.sh
	scripts/import-keys.sh
	docker-compose -f docker-compose.ibtida.yml up

format-code:
	cargo fmt

benchmarks-build:
	cargo build --release --manifest-path node/Cargo.toml --features runtime-benchmarks

benchmarks-generate:
	./target/release/anmol benchmark \
		--extrinsic '*' \
		--pallet pallet_nft \
		--output ./pallets/nft/src/weights.rs \
		--execution wasm \
		--wasm-execution compiled \
		--template=./.maintain/frame-weight-template.hbs \
		--steps 50 \
		--repeat 20 \

test:
	cargo test -p pallet-nft --all-features
