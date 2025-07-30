run:
	cargo run
win:
	chmod +x scripts/build_windows.sh && ./scripts/build_windows.sh
linux:
	cargo build --release --target x86_64-unknown-linux-musl
macos:
	chmod +x scripts/build_macos.sh && ./scripts/build_macos.sh
fix:
	cargo fixit --clippy --allow-dirty

svg2icon:
	chmod +x scripts/create_icon.sh && ./scripts/create_icon.sh