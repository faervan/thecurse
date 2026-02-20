run:
	RUST_LOG=info,thecurse=debug,wgpu_hal=off cargo run -p thecurse_game
edit:
	RUST_LOG=info,thecurse=debug,wgpu_hal=off cargo run -p thecurse_editor
