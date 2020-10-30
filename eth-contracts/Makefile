test:
	docker run --rm -p 8545:8545 tockb/geth-priv:v1.9.23 &
	./wait-for-it.sh -t 0 127.0.0.1:8545 -- echo "node is up"
	node scripts/waitForMiner.js
	npx hardhat test --network geth

.PHONY: test