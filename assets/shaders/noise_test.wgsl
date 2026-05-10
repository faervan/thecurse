#import bevy_pbr::forward_io::VertexOutput;
#import bevy_pbr::mesh_view_bindings::globals;
#import bevy_render::maths::PI_2;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
	// https://darkeclipz.github.io/fractals/paper/Fractals%20&%20Rendering%20Techniques.html
	let n = 32.;
	let b = 4.;
	var uv = in.uv * 2. - 1.;
	uv.x -= 0.5;

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

	if i == n {
		return vec4(vec3(0.), 1.);
		return color(0.);
	} else {
		return color(i / n);
	}
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
fn color(t: f32) -> vec4f {
	let palette = palette1();
	let a = palette[0];
	let b = palette[1];
	let c = palette[2];
	let d = palette[3];

	return vec4(a + b * cos(PI_2 * (c * t + d)), 1.);
}
