fn lerp(a: f32, b: f32, t: f32) -> f32{
    return a + t * (b - a);
}

fn lerp_vec(a: vec4<f32>, b: vec4<f32>, t: f32) -> vec4<f32> {
    return vec4<f32>(lerp(a.x, b.x, t), lerp(a.y, b.y, t), lerp(a.z, b.z, t), lerp(a.w, b.w, t));
}
