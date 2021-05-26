keystore-add:
	curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d "@scripts/keystore.json"