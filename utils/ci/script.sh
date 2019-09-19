# Derived from https://github.com/japaric/trust

set -ex

# ----- Options -----

# TARGET enables cross-building
if [ -z $TARGET ]; then
    CARGO=cargo
elif [ "$TARGET" = "i686-unknown-linux-musl" ]; then
    CARGO=cargo
    TARGET="--target $TARGET"
else
    CARGO=cross
    TARGET="--target $TARGET"
fi

# NIGHTLY defaults off


# ----- Script -----

main() {
  if [ "0$NIGHTLY" -ge 1 ]; then
    $CARGO test $TARGET --benches
  fi

  $CARGO test $TARGET --manifest-path rand_isaac/Cargo.toml --features=serde1
  $CARGO test $TARGET --manifest-path rand_xorshift/Cargo.toml --features=serde1
  $CARGO test $TARGET --manifest-path rand_xoshiro/Cargo.toml
  $CARGO test $TARGET --manifest-path rand_jitter/Cargo.toml
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
