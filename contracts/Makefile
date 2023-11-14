
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Build library dynamically linked to the rust runtime libraries
build:
	RUSTFLAGS='-C link-arg=-s' cargo +stable build --all --target wasm32-unknown-unknown --release
	cp target/wasm32-unknown-unknown/release/katherine_sale_contract.wasm res/
	cp target/wasm32-unknown-unknown/release/test_p_token.wasm res/

dev-deploy: build
	./scripts/dev-deploy.sh

test:
	RUST_BACKTRACE=1 cargo test -- --color always --nocapture

format:
	cargo fmt -- --check

doc:
	cargo doc

clean:
	cargo clean
