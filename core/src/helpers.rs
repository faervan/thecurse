/// Compare two `[f32; 3]`, allowing minimal deviation of `0.05`.
pub fn approx_eq(left: [f32; 3], right: [f32; 3]) -> bool {
    left.into_iter()
        .zip(right)
        .all(|(l, r)| (l - r).abs() < 0.05)
}
