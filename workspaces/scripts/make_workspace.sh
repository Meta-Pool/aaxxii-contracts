cd ../contracts
RUSTFLAGS='-C link-arg=-s' cargo +stable build --all --target wasm32-unknown-unknown --release
cp ./target/wasm32-unknown-unknown/release/katherine_sale_contract.wasm ../workspaces/res/
cp ./target/wasm32-unknown-unknown/release/test_p_token.wasm ../workspaces/res/
cd -
cargo run