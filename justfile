run:
	RUST_LOG=info,thecurse=debug,wgpu_hal=off cargo run -p thecurse_game

edit:
	RUST_LOG=info,thecurse=debug,wgpu_hal=off cargo run -p thecurse_editor

serve:
	RUST_LOG=info,thecurse=debug cargo run -p thecurse_server

ci-check:
	cargo +nightly fmt -- --config error_on_line_overflow=true --check && cargo clippy
