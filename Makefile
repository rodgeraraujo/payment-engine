.PHONY: build test lint run docker bigtest

build:
	cargo build --release

test:
	cargo test

lint:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings

run:
	cargo run --release -- tests/fixtures/basic.csv

docker:
	docker build -t payments-engine .
	docker run --rm -v "$(PWD)/tests/fixtures:/data:ro" payments-engine /data/basic.csv

bigtest: build
	python3 scripts/gen_large.py 5000000 > /tmp/big.csv
	/usr/bin/time -v ./target/release/payments-engine /tmp/big.csv > /dev/null
