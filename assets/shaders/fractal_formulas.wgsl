#import "shaders/complex.wgsl"::{complex_mul, complex_pow, complex_cosh, complex_add}

fn mandelbrot_classic(z: vec2<f64>, c: vec2<f64>) -> vec2<f64> {
    return complex_add(complex_pow(z, 2), c);
}

fn mandelbrot_classic_10(z: vec2<f64>, c: vec2<f64>) -> vec2<f64> {
    return complex_add(complex_pow(z, 10), c);
}

fn coshbrot_3(z: vec2<f64>, c: vec2<f64>) -> vec2<f64> {
    return complex_add(complex_cosh(complex_pow(z, 3)), c);
}

fn burning_ship(z: vec2<f64>, c: vec2<f64>) -> vec2<f64> {
    return complex_add(complex_pow(abs(z.x) + 1 * abs(z.y), 2), c);
}
