cargo publish --manifest-path core/Cargo.toml &&
sleep 10 &&
cargo publish --manifest-path macro/Cargo.toml &&
sleep 10 &&
cargo publish;