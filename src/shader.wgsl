// Vertex shader
struct CameraUniform {
    // view projection for the left eye
    view_proj_left: mat4x4<f32>,
    // view projection for the right eye
    view_proj_right: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct RenderEyeTarget {
    // -1 for left eye, 1 for right eye
    eye_target: f32,

    _padding_0: f32,
    _padding_1: f32,
    _padding_2: f32,
};
@group(1) @binding(0)
var<uniform> render_eye_target: RenderEyeTarget;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};
 

struct VertexOutput {
    @builtin(position) clip_space_target_eye: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) ndc_space_target_eye: vec3<f32>,
    // -1 for left eye, 1 for right eye
    @location(2) eye_target: f32,
    @location(3) world_space_pos: vec4<f32>,
    @location(4) ndc_space_left_eye: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    // get the view projection matrix for the current eye
    // if the eye target is negative, we want to render the left eye
    var camera_view_proj_for_eye: mat4x4<f32>;
    if( render_eye_target.eye_target < 0.0 ) {
        camera_view_proj_for_eye = camera.view_proj_left;
    } else {
        camera_view_proj_for_eye = camera.view_proj_right;
    }

    var out: VertexOutput;
    // COPY ==============================
    out.color = model.color;
    out.eye_target = f32(render_eye_target.eye_target);

    // DO TRANSFORM ==============================
    out.world_space_pos = model_matrix * vec4<f32>(model.position, 1.0);
    out.clip_space_target_eye = camera_view_proj_for_eye * out.world_space_pos;
    // for the left eye we want to render the left half of the screen
    // for the right eye we want to render the right half of the screen
    out.clip_space_target_eye.x = out.clip_space_target_eye.x / 2.0 + 0.5 * f32(render_eye_target.eye_target) * out.clip_space_target_eye.w;

    // NDC SPACE ==============================
    out.ndc_space_target_eye = vec3<f32>(out.clip_space_target_eye.x, out.clip_space_target_eye.y, out.clip_space_target_eye.z) / out.clip_space_target_eye.w;
    
    // we need the left eye position for the glitch mask
    // we want to map the same surfaces of objects for the left and right eye to the same pixel in the glitch mask
    // for this we use the left eye position to index the glitch mask for both eyes
    let clip_space_left_eye = camera.view_proj_left * out.world_space_pos;
    out.ndc_space_left_eye = vec3<f32>(clip_space_left_eye.x, clip_space_left_eye.y, clip_space_left_eye.z) / clip_space_left_eye.w;

    return out;
}


@group(2)@binding(0)
var t_glitch_area: texture_2d<f32>;
@group(2)@binding(1)
var s_glitch_area: sampler;

struct GlitchAreaUniform {
    time: f32,
    visibility: f32,

    _padding_0: f32,
    _padding_1: f32,
};
@group(3)@binding(0)
var<uniform> glitch_area: GlitchAreaUniform;


// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // if out of bounds for target eye discard
    if ((in.ndc_space_target_eye.x * in.eye_target) < 0.0 || (in.ndc_space_target_eye.x * in.eye_target) > 1.0) {
        discard;
    }

    // Ugly hack: just fix the size of the glitch mask so we do not need to pass the size 
    // or reallocate the texture when the level changes
    let w = 256.0;
    let h = 256.0;

    let u = in.world_space_pos.x / w;
    let v = in.world_space_pos.y * -1.0 / h;

    let glitch_mask_color = textureSample(t_glitch_area, s_glitch_area, vec2<f32>(u,v));
    if( glitch_mask_color.r > 0.5 ) {
        return vec4<f32>(in.color, 1.0);
    } else {
        return random_pattern(vec2<f32>(in.ndc_space_left_eye.x, in.ndc_space_left_eye.y));
    }
}

fn random_pattern(uv: vec2<f32>) -> vec4<f32> {
    let x = steps(uv.x, 40.0);
    let y = steps(uv.y, 40.0);

    let xy_offset = simplexNoise2(vec2f(95.5498 + x, y + 95.5498) * 0.5 + vec2f(glitch_area.time * 0.05, glitch_area.time * 0.05));
    
    var r = 0.0;
    var g = 0.0;
    var b = 0.0;

    // layered noise 
    let steps = 2;
    for (var i = 1; i <= steps; i = i + 1) {
        r = r + (steps(simplexNoise2(vec2f(glitch_area.time * 0.05, glitch_area.time * 0.05) + vec2f(95.5498 + x, y + 95.5498 + xy_offset * 0.1) * pow(3.5, f32(i))) + 0.2, 64.0)) * pow(2.0, f32(-i) * 0.5) * 1.1;
        g = g + (steps(simplexNoise2(vec2f(glitch_area.time * 0.05, glitch_area.time * 0.05) + vec2f(95.5498 + x, y + 95.5498 + xy_offset * 0.1) * pow(3.5, f32(i)) - vec2f(1.0,1.0) * glitch_area.visibility * 0.15) + 0.2, 64.0))* pow(2.0, f32(-i) * 0.5) * 1.1;
        b = b + (steps(simplexNoise2(vec2f(glitch_area.time * 0.05, glitch_area.time * 0.05) + vec2f(95.5498 + x, y + 95.5498 + xy_offset * 0.1) * pow(3.5, f32(i)) - vec2f(1.0,1.0) * glitch_area.visibility * 0.3) + 0.2, 64.0))* pow(2.0, f32(-i) * 0.5) * 1.1;
    }

    return vec4<f32>(r, g, b, 1.0);
}

fn steps(input: f32, steps: f32) -> f32 {
    return floor(input * steps) / steps;
}

// https://gist.github.com/munrocket/236ed5ba7e409b8bdf1ff6eca5dcdc39
//  MIT License. © Ian McEwan, Stefan Gustavson, Munrocket, Johan Helsing
//
fn mod289(x: vec2f) -> vec2f {
    return x - floor(x * (1. / 289.)) * 289.;
}

fn mod289_3(x: vec3f) -> vec3f {
    return x - floor(x * (1. / 289.)) * 289.;
}

fn permute3(x: vec3f) -> vec3f {
    return mod289_3(((x * 34.) + 1.) * x);
}

//  MIT License. © Ian McEwan, Stefan Gustavson, Munrocket
fn simplexNoise2(v: vec2f) -> f32 {
    let C = vec4(
        0.211324865405187, // (3.0-sqrt(3.0))/6.0
        0.366025403784439, // 0.5*(sqrt(3.0)-1.0)
        -0.577350269189626, // -1.0 + 2.0 * C.x
        0.024390243902439 // 1.0 / 41.0
    );

    // First corner
    var i = floor(v + dot(v, C.yy));
    let x0 = v - i + dot(i, C.xx);

    // Other corners
    var i1 = select(vec2(0., 1.), vec2(1., 0.), x0.x > x0.y);

    // x0 = x0 - 0.0 + 0.0 * C.xx ;
    // x1 = x0 - i1 + 1.0 * C.xx ;
    // x2 = x0 - 1.0 + 2.0 * C.xx ;
    var x12 = x0.xyxy + C.xxzz;
    x12.x = x12.x - i1.x;
    x12.y = x12.y - i1.y;

    // Permutations
    i = mod289(i); // Avoid truncation effects in permutation

    var p = permute3(permute3(i.y + vec3(0., i1.y, 1.)) + i.x + vec3(0., i1.x, 1.));
    var m = max(0.5 - vec3(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw)), vec3(0.));
    m *= m;
    m *= m;

    // Gradients: 41 points uniformly over a line, mapped onto a diamond.
    // The ring size 17*17 = 289 is close to a multiple of 41 (41*7 = 287)
    let x = 2. * fract(p * C.www) - 1.;
    let h = abs(x) - 0.5;
    let ox = floor(x + 0.5);
    let a0 = x - ox;

    // Normalize gradients implicitly by scaling m
    // Approximation of: m *= inversesqrt( a0*a0 + h*h );
    m *= 1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h);

    // Compute final noise value at P
    let g = vec3(a0.x * x0.x + h.x * x0.y, a0.yz * x12.xz + h.yz * x12.yw);
    return 130. * dot(m, g);
}