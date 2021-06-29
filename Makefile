run:
	cargo run -- --dev --tmp --enable-offchain-indexing=true
run-debug:
	cargo run -- --dev --tmp --enable-offchain-indexing=true -l sc_offchain=trace
keystore-add:
	curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d "@scripts/keystore.json"