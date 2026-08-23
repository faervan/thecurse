run *FLAGS:
	RUST_LOG=info,thecurse=debug,mini_udp=debug,wgpu_hal=off cargo run -p thecurse_game -- {{FLAGS}}

edit:
	RUST_LOG=info,thecurse=debug,mini_udp=debug,wgpu_hal=off cargo run -p thecurse_editor

serve *FLAGS:
	RUST_LOG=info,thecurse=debug,mini_udp=debug cargo run -p thecurse_server -- {{FLAGS}}

ci-check:
	cargo +nightly fmt -- --config error_on_line_overflow=true --check && cargo clippy

test:
	cargo test
