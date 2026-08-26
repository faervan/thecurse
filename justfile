run *FLAGS:
	RUST_LOG=info,dreamgame=debug,mini_udp=debug,wgpu_hal=off cargo run -p dreamgame_game -- {{FLAGS}}

serve *FLAGS:
	RUST_LOG=info,dreamgame=debug,mini_udp=debug cargo run -p dreamgame_server -- {{FLAGS}}

ci-check:
	cargo +nightly fmt -- --config error_on_line_overflow=true --check && cargo clippy

test:
	cargo test
