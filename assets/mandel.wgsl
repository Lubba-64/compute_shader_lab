@group(0) @binding(0) var texture: texture_storage_2d<rgba8unorm, read_write>;

const escape_dist: f64 = 10;
const iterations: u32 = 10000;
const view_scale: f64 = 10;
const view_pos: vec2<f64> = vec2<f64>(0, 0);

// real is x imag is y
fn complex_mul(a: vec2<f64>, b: vec2<f64>) -> vec2<f64>{
    let re = a.x * b.x - a.y * b.y;
    let im = a.x * b.y + a.y * b.x;
    return vec2<f64>(re, im);    
}

fn complex_pow(a: vec2<f64>, pow: i32) -> vec2<f64>{
    var res = a;
    for (var i: i32 = 0; i < pow - 1; i++){
        res = complex_mul(res, a);
    }
    return res;
}

fn complex_cosh(a: vec2<f64>) -> vec2<f64> {
    return vec2<f64>(sinh(a.x) * cos(a.y), sinh(a.x) * sin(a.y));
}

fn complex_norm_sqr(a: vec2<f64>) -> f64{
    return a.x * a.x + a.y * a.y;
}

fn complex_div(a: vec2<f64>, b: vec2<f64>) -> vec2<f64> {
    let norm_sqr = complex_norm_sqr(b);
    let re = a.x * b.x + a.y * b.y;
    let im = a.y * b.x - a.x * b.y;
    return vec2<f64>(re / norm_sqr, im / norm_sqr);
}

fn complex_add(a: vec2<f64>, b: vec2<f64>) -> vec2<f64>{
    return a + b;
}

fn complex_sub(a: vec2<f64>, b: vec2<f64>) -> vec2<f64>{
    return a - b;
}

fn lerp(a: f32, b: f32, t: f32) -> f32{
    return a + t * (b - a);
}

fn lerp_vec(a: vec4<f32>, b: vec4<f32>, t: f32) -> vec4<f32> {
    return vec4<f32>(lerp(a.x, b.x, t), lerp(a.y, b.y, t), lerp(a.z, b.z, t), lerp(a.w, b.w, t));
}

@compute @workgroup_size(8, 8, 1)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>) { 
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));  

    let c = ((vec2<f64>(location) - vec2<f64>(textureDimensions(texture).xy) / 2) / vec2<f64>(textureDimensions(texture).xy)) * view_scale + view_pos;
    var z = vec2<f64>(0);
    var escaped = false;
    for (var i: u32 = 0; i < iterations; i++){
        if (abs(z.x) + abs(z.y) > escape_dist){
            escaped = true;
            textureStore(texture, location, lerp_vec(vec4<f32>(1,0,0,1), vec4<f32>(1,1,1,1), f32(i) / f32(iterations)));
            return;
        }
        z = mandelbrot_classic_10(z, c);
    }
    textureStore(texture, location, vec4<f32>(0,0,0,1));
}

fn mandelbrot_classic(z: vec2<f64>, c: vec2<f64>) -> vec2<f64> {
    return complex_add(complex_pow(z, 2), c);
}

fn mandelbrot_classic_10(z: vec2<f64>, c: vec2<f64>) -> vec2<f64> {
    return complex_add(complex_pow(z, 10), c);
}

fn coshbrot_3(z: vec2<f64>, c: vec2<f64>) -> vec2<f64> {
    return complex_add(complex_cosh(complex_pow(z, 3)), c);
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));  

    let c = ((vec2<f64>(location) - vec2<f64>(textureDimensions(texture).xy) / 2) / vec2<f64>(textureDimensions(texture).xy)) * view_scale + view_pos;
    var z = vec2<f64>(0);
    var escaped = false;
    for (var i: u32 = 0; i < iterations; i++){
        if (abs(z.x) + abs(z.y) > escape_dist){
            escaped = true;
            textureStore(texture, location, lerp_vec(vec4<f32>(1,0,0,1), vec4<f32>(1,1,1,1), f32(i) / f32(iterations)));
            return;
        }
        z = mandelbrot_classic_10(z, c);
    }
    textureStore(texture, location, vec4<f32>(0,0,0,1));
}