// lumen: one shader for every primitive. See lumen-core/src/paint.rs.

struct Uniforms {
    screen: vec2<f32>,   // logical size
    scale: f32,          // physical px per logical px
    _pad: f32,
};

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var atlas: texture_2d<f32>;

struct VIn {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) local: vec2<f32>,
    @location(3) half_size: vec2<f32>,
    @location(4) radii: vec4<f32>,
    @location(5) param: f32,
    @location(6) mode: u32,
    @location(7) color: vec4<f32>,
    @location(8) border: vec4<f32>,
};

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) local: vec2<f32>,
    @location(2) @interpolate(flat) half_size: vec2<f32>,
    @location(3) @interpolate(flat) radii: vec4<f32>,
    @location(4) @interpolate(flat) param: f32,
    @location(5) @interpolate(flat) mode: u32,
    @location(6) color: vec4<f32>,
    @location(7) @interpolate(flat) border: vec4<f32>,
};

@vertex
fn vs_main(v: VIn) -> VOut {
    var o: VOut;
    let ndc = v.pos / u.screen * 2.0 - 1.0;
    o.pos = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);
    o.uv = v.uv;
    o.local = v.local;
    o.half_size = v.half_size;
    o.radii = v.radii;
    o.param = v.param;
    o.mode = v.mode;
    o.color = v.color;
    o.border = v.border;
    return o;
}

fn corner_radius(p: vec2<f32>, r: vec4<f32>) -> f32 {
    // r = (top-left, top-right, bottom-right, bottom-left); y grows down.
    if (p.x < 0.0) {
        return select(r.w, r.x, p.y < 0.0);
    }
    return select(r.z, r.y, p.y < 0.0);
}

fn sd_round_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + vec2<f32>(r, r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0, 0.0))) - r;
}

fn premul(c: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(c.rgb * c.a, c.a);
}

// Evan Wallace's closed-form blurred rounded rectangle.
fn gaussian(x: f32, sigma: f32) -> f32 {
    return exp(-(x * x) / (2.0 * sigma * sigma)) / (2.5066282746 * sigma);
}

fn erf2(x: vec2<f32>) -> vec2<f32> {
    let s = sign(x);
    let a = abs(x);
    var y = 1.0 + (0.278393 + (0.230389 + 0.078108 * (a * a)) * a) * a;
    y = y * y;
    return s - s / (y * y);
}

fn shadow_x(x: f32, y: f32, sigma: f32, corner: f32, half_size: vec2<f32>) -> f32 {
    let delta = min(half_size.y - corner - abs(y), 0.0);
    let curved = half_size.x - corner + sqrt(max(0.0, corner * corner - delta * delta));
    let integral = 0.5 + 0.5 * erf2((x + vec2<f32>(-curved, curved)) * (0.7071067811 / sigma));
    return integral.y - integral.x;
}

fn rounded_box_shadow(p: vec2<f32>, half_size: vec2<f32>, sigma: f32, corner: f32) -> f32 {
    let low = p.y - half_size.y;
    let high = p.y + half_size.y;
    let start = clamp(-3.0 * sigma, low, high);
    let end = clamp(3.0 * sigma, low, high);
    let step = (end - start) / 4.0;
    var y = start + step * 0.5;
    var value = 0.0;
    for (var i = 0; i < 4; i = i + 1) {
        value = value + shadow_x(p.x, p.y - y, sigma, corner, half_size) * gaussian(y, sigma) * step;
        y = y + step;
    }
    return value;
}

// Text contrast correction. lumen blends in gamma space, which renders
// light text on dark backgrounds thin and washed out (and dark on light a
// little heavy). Boost partial coverage more for light text, and more at
// 1x where stems are only a pixel or two wide, the way DirectWrite and
// browsers do.
fn text_coverage(cov: f32, rgb: vec3<f32>, scale: f32) -> f32 {
    let brightness = dot(rgb, vec3<f32>(0.30, 0.59, 0.11));
    let light = smoothstep(0.25, 0.85, brightness);
    let density = select(0.55, 1.0, scale < 1.5);
    let k = (0.25 + 1.15 * light) * density;
    return cov * (k + 1.0) / (cov * k + 1.0);
}

@fragment
fn fs_main(i: VOut) -> @location(0) vec4<f32> {
    switch i.mode {
        case 0u: {
            return premul(i.color);
        }
        case 1u: {
            let r = corner_radius(i.local, i.radii);
            let d = sd_round_box(i.local, i.half_size, r);
            let outer = clamp(0.5 - d * u.scale, 0.0, 1.0);
            let fill = premul(i.color);
            if (i.param <= 0.0) {
                return fill * outer;
            }
            // Inside border: the inner edge is the same SDF offset by the
            // border width, which keeps inner corners concentric.
            let inner = clamp(0.5 - (d + i.param) * u.scale, 0.0, 1.0);
            let b = premul(i.border);
            return b * (outer - inner) + fill * inner;
        }
        case 2u: {
            let r = corner_radius(i.local, i.radii);
            let a = rounded_box_shadow(i.local, i.half_size, i.param, r);
            return premul(i.color) * a;
        }
        default: {
            let raw = textureLoad(atlas, vec2<i32>(floor(i.uv)), 0).r;
            return premul(i.color) * text_coverage(raw, i.color.rgb, u.scale);
        }
    }
}
