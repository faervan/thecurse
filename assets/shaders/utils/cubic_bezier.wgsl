#define_import_path thecurse::utils

// https://www.desmos.com/calculator/ebdtbxgbq0
fn cubic_bezier(start: vec2f, end: vec2f, ctrl_start: vec2f, ctrl_end: vec2f, t: f32) -> vec2f {
	return vec2(
		cubic_bezier_1d(start.x, end.x, ctrl_start.x, ctrl_end.x, t),
		cubic_bezier_1d(start.y, end.y, ctrl_start.y, ctrl_end.y, t),
	);
}

fn cubic_bezier_1d(start: f32, end: f32, ctrl_start: f32, ctrl_end: f32, t: f32) -> f32 {
	return pow(1 - t, 3) * start
			+ 3 * t * pow(1 - t, 2) * ctrl_start
			+ 3 * pow(t, 2) * (1 - t) * ctrl_end
			+ pow(t, 3) * end;
}
