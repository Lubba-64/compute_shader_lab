#import "shaders/lerp.wgsl"::lerp_vec
#import "shaders/fractal_formulas.wgsl"::mandelbrot_classic

struct GameOfLifeData {
    view_scale: f64,
    view_pos: vec2<f64>,
};


@group(0) @binding(0) var texture: texture_storage_2d<rgba8unorm, read_write>;
@group(0) @binding(1) var<uniform> game_of_life_data: GameOfLifeData;

const escape_dist: f64 = 10;
const iterations: u32 = 100;

@compute @workgroup_size(8, 8, 1)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let c = ((vec2<f64>(location) - vec2<f64>(textureDimensions(texture).xy) / 2) / vec2<f64>(textureDimensions(texture).xy)) * game_of_life_data.view_scale + game_of_life_data.view_pos;
    var z = vec2<f64>(0);
    var escaped = false;
    for (var i: u32 = 0; i < iterations; i++){
        if (abs(z.x) + abs(z.y) > escape_dist){
            escaped = true;
            textureStore(texture, location, lerp_vec(vec4<f32>(1,0,0,1), vec4<f32>(1,1,1,1), f32(i) / f32(iterations) * 100000 % 1));
            return;
        }
        z = mandelbrot_classic(z, c);
    }
    textureStore(texture, location, vec4<f32>(0,0,0,1));
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let c = ((vec2<f64>(location) - vec2<f64>(textureDimensions(texture).xy) / 2) / vec2<f64>(textureDimensions(texture).xy)) * game_of_life_data.view_scale + game_of_life_data.view_pos;
    var z = vec2<f64>(0);
    var escaped = false;
    for (var i: u32 = 0; i < iterations; i++){
        if (abs(z.x) + abs(z.y) > escape_dist){
            escaped = true;
            textureStore(texture, location, lerp_vec(vec4<f32>(1,0,0,1), vec4<f32>(1,1,1,1), f32(i) / f32(iterations) * 100000 % 1));
            return;
        }
        z = mandelbrot_classic(z, c);
    }
    textureStore(texture, location, vec4<f32>(0,0,0,1));
}
