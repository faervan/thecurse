#import bevy_pbr::forward_io::VertexOutput;
#import bevy_pbr::mesh_view_bindings::globals;
#import bevy_render::maths::PI_2;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
	let points = 50.;
	let seed = 0.914282;

	let px = in.uv.x * points;
	let py = in.uv.y * points;

	let fx = fract(px);
	let fy = fract(py);

	let x0 = px - fx;
	let x1 = x0 + 1.;
	let y0 = py - fy;
	let y1 = y0 + 1.;

	let center = in.uv * points;
	let tl = gradient(vec2(x0, y0), center, seed);
	let tr = gradient(vec2(x1, y0), center, seed);
	let bl = gradient(vec2(x0, y1), center, seed);
	let br = gradient(vec2(x1, y1), center, seed);

	let top = mix(tl, tr, fx);
	let bottom = mix(bl, br, fx);

	if mix(top, bottom, fy) * 2. < sin(globals.time) {
		return vec4(0., 0., 0., 1.);
	} else {
		return vec4(1., 0., 0., 1.);
	}
}

fn gradient(corner: vec2f, center: vec2f, seed: f32) -> f32 {
	let angle = hash2(corner * seed) * PI_2;
	let random_direction = rotate_vec2(vec2(0., 1.), angle);
	return dot(random_direction, corner - center);
}

fn rotate_vec2(in: vec2f, angle: f32) -> vec2f {
	let s = sin(angle);
	let c = cos(angle);

	return vec2(
		in.x * c - in.y * s,
		in.x * s + in.y * c
	);
}

fn hash2(v: vec2f) -> f32 {
	return hash(dot(v, vec2(5742.948, 198.43)));
}

fn hash(v: f32) -> f32 {
	return fract(sin(v) * 3821.492173);
}
