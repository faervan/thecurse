run *FLAGS:
	RUST_LOG=info,dreamgame=debug,mini_udp=debug,wgpu_hal=off cargo run -p dreamgame_game \
		-- {{FLAGS}}

run-trace *FLAGS:
	RUST_LOG=info,dreamgame=debug,mini_udp=trace,wgpu_hal=off cargo run -p dreamgame_game \
		-- {{FLAGS}}

run-on-server *FLAGS:
	just run --addr 72.61.104.16 --no-fake-unreliability {{FLAGS}}

serve *FLAGS:
	RUST_LOG=info,dreamgame=debug,mini_udp=debug cargo run -p dreamgame_server -- {{FLAGS}}

serve-trace *FLAGS:
	RUST_LOG=info,dreamgame=debug,mini_udp=trace cargo run -p dreamgame_server -- {{FLAGS}}

serve-release *FLAGS:
	RUST_LOG=info,dreamgame=debug,mini_udp=debug cargo run -p dreamgame_server \
		--release \
		--no-default-features \
		-- {{FLAGS}}

ci-check:
	cargo +nightly fmt -- --config error_on_line_overflow=true --check && cargo clippy

test:
	cargo test
