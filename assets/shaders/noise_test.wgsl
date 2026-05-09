#import bevy_pbr::forward_io::VertexOutput;
#import bevy_pbr::mesh_view_bindings::globals;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
	let points = 50.;

	let x = sample_point_2d(in.uv.x * points);
	let y = sample_point_2d(in.uv.y * points);

	if mix(x, y, 0.5) < sin(globals.time) {
		return vec4(0., 0., 0., 1.);
	} else {
		return vec4(1., 0., 0., 1.);
	}
}

fn sample_point_2d(px: f32) -> f32 {
	let prev = px - fract(px);
	let next = prev + 1.;

	// Random value between -1 and 1
	let prev_v = hash(19.26 + prev) * 2. - 1.;
	let next_v = hash(19.26 + next) * 2. - 1.;

	return mix(prev_v, next_v, fract(px));
}

fn hash(v: f32) -> f32 {
	return fract(sin(v) * 3821.492173);
}
