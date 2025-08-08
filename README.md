# viergewinnt-rs

A Rust implementation of the game _Vier gewinnt_, also kown as _Connect Four_ or
_Captain’s mistress_. The crate exposes a CLI where you can play against the
computer. The computer performs a minmax search to look for good moves.

## Run

`$ RUSTFLAGS="-C target-cpu native" cargo run --release`
