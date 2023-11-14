
ifndef NEAR_ACCOUNT
NEAR_ACCOUNT="kate_tester3.testnet"
endif

lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Build library dynamically linked to the rust runtime libraries
build:
	RUSTFLAGS='-C link-arg=-s' cargo +stable build --all --target wasm32-unknown-unknown --release
	cp target/wasm32-unknown-unknown/release/katherine_sale_contract.wasm res/

publish-dev: build
	NEAR_ENV=testnet near dev-deploy --wasmFile res/katherine_fundraising_contract.wasm

publish-dev-init: build
	rm -rf neardev/
	NEAR_ENV=testnet near dev-deploy --wasmFile res/katherine_fundraising_contract.wasm --initFunction new --initArgs '{"owner_id": ${NEAR_ACCOUNT}, "min_deposit_amount": "2000000000000", "metapool_contract_address": "meta-v2.pool.testnet", "katherine_fee_percent": 100 }'

integration-meta-pool: build
	./scripts/integration_meta_pool.sh

integration: build
	./scripts/integration.sh

market: build
	./scripts/test_pre_release_market.sh

test-operator: build
	./scripts/test_operator.sh

install:
	cp target/release/libcfdi.so /usr/local/lib64/

test:
	RUST_BACKTRACE=1 cargo test -- --color always --nocapture

format:
	cargo fmt -- --check

doc:
	cargo doc

clean:
	cargo clean