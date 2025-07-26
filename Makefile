run:
	cargo run
r:
	cargo build --release
r-win:
	cargo build --release --target x86_64-pc-windows-gnu
r-linux:
	cargo build --release --target x86_64-unknown-linux-musl
fix:
	cargo fixit --clippy --allow-dirty