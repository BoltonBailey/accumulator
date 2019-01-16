# Set up the Rust toolchain and modify PATH.
curl https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env
rustup update

# Install useful extensions.
cargo install cargo-watch
cargo install cargo-edit
