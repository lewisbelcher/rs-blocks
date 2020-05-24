# Publish crates on crates.io

set -e
echo $CARGO_TOKEN
cargo login $CARGO_TOKEN
cd rs-blocks-derive
cargo publish || :  # Allow error if version exists (other errors should be caught below)
cd ..
cargo publish
