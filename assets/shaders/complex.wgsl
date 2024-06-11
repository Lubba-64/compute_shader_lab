// real is x imag is y

fn complex_mul(a: vec2<f64>, b: vec2<f64>) -> vec2<f64>{
    return vec2<f64>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);    
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
    return vec2<f64>((a.x * b.x + a.y * b.y) / norm_sqr, (a.y * b.x - a.x * b.y) / norm_sqr);
}

fn complex_add(a: vec2<f64>, b: vec2<f64>) -> vec2<f64>{
    return a + b;
}

fn complex_sub(a: vec2<f64>, b: vec2<f64>) -> vec2<f64>{
    return a - b;
}
