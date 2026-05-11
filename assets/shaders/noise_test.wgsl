#import bevy_pbr::forward_io::VertexOutput;
#import bevy_pbr::mesh_view_bindings::globals;
#import thecurse::gradient_noise::noise_2d;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
	let grid_size = vec2(50.);
	let seed = 0.914282;

	let noise = noise_2d(in.uv, grid_size, seed);
	return vec4(noise, 0., 0., 1.);
}
