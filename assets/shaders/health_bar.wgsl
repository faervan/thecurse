#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP})
@binding(0)
var<uniform> health_percent: f32;

@fragment
fn fs_main(mesh: VertexOutput) -> @location(0) vec4f {
	if mesh.uv.x > health_percent {
		return vec4(0., 0., 0., 1.);
	} else {
		return vec4(1., 0., 0., 1.);
	}
}
