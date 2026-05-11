#import bevy_pbr::forward_io::{Vertex, VertexOutput};
#import bevy_pbr::mesh_functions::{
	mesh_position_local_to_clip,
	get_world_from_local,
	mesh_normal_local_to_world,
};

#import thecurse::gradient_noise::noise_2d;

@vertex
fn vs_main(in: Vertex) -> VertexOutput {
	var out: VertexOutput;

	let world_from_local = get_world_from_local(in.instance_index);
	let position = vec4(in.position, 1.0);

	out.world_position = world_from_local * position;
	out.world_normal = mesh_normal_local_to_world(in.normal, in.instance_index);
	out.instance_index = in.instance_index;
	out.uv = in.uv;

	out.position = mesh_position_local_to_clip(
        world_from_local,
        position
    );

	let grid_size = vec2(0.2);
	let seed = 0.914282;
	let noise = noise_2d(out.world_position.xz, grid_size, seed);
	let detail_noise = noise_2d(out.world_position.xz, grid_size * 20., seed * 0.98433);

	out.position.y += noise * 2.;
	out.position.y += detail_noise * 0.5;

	return out;
}
