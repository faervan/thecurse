#import bevy_pbr::forward_io::VertexOutput;
#import bevy_pbr::mesh_view_bindings::globals;
#import bevy_render::maths::PI_2;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
	// https://darkeclipz.github.io/fractals/paper/Fractals%20&%20Rendering%20Techniques.html
	let n = 1024.;
	let b = 256.;
	let sample_count = 16.;

	var frag_color: vec3f;

	let centered = in.uv - 0.5;
	let animated_uv = rotate_vec2(centered, fract(globals.time) * PI_2) + 0.5;

	for (var i = 0.; i < sample_count; i += 1) {
		var uv = in.uv * 2. - 1.;
		// var uv = animated_uv * 2. - 1.;
		// jitter
		uv += hash2(uv) * 0.01;
		// zoom
		uv *= 0.001;
		// move
		uv -= vec2(.908, -0.23527);

		let sample = sample_point(uv, n, b);
		frag_color += color(sample);
	}

	return vec4(frag_color / sample_count, 1.);
}

fn sample_point(uv: vec2f, n: f32, b: f32) -> f32 {
	var z = vec2(0.);
	let c = uv;

	var i: f32;
	for (i = 0.; i < n; i += 1.) {
		z = mat2x2(z, vec2(-z.y, z.x)) * z + c;
		if dot(z, z) > b * b {
			break;
		}
	}

	// Smooth iteration count to get rid of color bands
	i -= log(log(dot(z, z)) / log(b)) / log(2.);

	return i / n;
}

fn palette1() -> mat4x3f {
	let a = vec3(0.938, 0.328, 0.718);
	// let b = vec3(0.659, 0.438, 0.328);
	let b = a;
	// let c = vec3(0.388, 0.388, 0.296);
	let c = a;
	let d = vec3(2.538, 2.478, 0.168);
	return mat4x3f(a, b, c, d);
}

// https://dev.thi.ng/gradients/
fn color(t: f32) -> vec3f {
	let palette = palette1();
	let a = palette[0];
	let b = palette[1];
	let c = palette[2];
	let d = palette[3];

	return vec3(a + b * cos(PI_2 * (c * t + d)));
}

fn hash(p: vec2f) -> f32 {
	return fract(sin(dot(p, vec2(9843.0218, 827.9137))) * 2874.042981);
}

fn hash2(p: vec2f) -> vec2f {
	return vec2(hash(p), fract(sin(hash(p)) * 134.10482));
}

fn rotate_vec2(in: vec2f, angle: f32) -> vec2f {
	let s = sin(angle);
	let c = cos(angle);

	return vec2(
		in.x * c - in.y * s,
		in.x * s + in.y * c
	);
}
