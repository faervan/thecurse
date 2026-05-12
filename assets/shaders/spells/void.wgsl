#import bevy_pbr::forward_io::VertexOutput;
#import bevy_pbr::decal::forward::get_forward_decal_info;

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> progress: f32;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
	let decal_info = get_forward_decal_info(in);
	let q = decal_info.uv - vec2(0.5);

	var color = vec4(0.5, 2., 4., 0.);

	let outer = 0.4;
	let inner = 0.4 * progress;
	let width = 0.01;

	let len = length(q);
	let in_outer = len > outer && outer + width > len;
	let in_inner = len > inner && inner + width > len;
	if in_outer || in_inner {
		color.a = decal_info.alpha;
	}

	return color;
}
