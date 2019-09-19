set -ex

MIRI_NIGHTLY=nightly-$(curl -s https://rust-lang.github.io/rustup-components-history/x86_64-unknown-linux-gnu/miri)
echo "Installing latest nightly with Miri: $MIRI_NIGHTLY"
rustup default "$MIRI_NIGHTLY"

rustup component add miri
cargo miri setup

cargo miri test --manifest-path rand_isaac/Cargo.toml --features=serde1
cargo miri test --manifest-path rand_xorshift/Cargo.toml --features=serde1
cargo miri test --manifest-path rand_xoshiro/Cargo.toml --features=serde1
cargo miri test --manifest-path rand_jitter/Cargo.toml
