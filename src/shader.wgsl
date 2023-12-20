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
    @location(1) tex_pos: vec2<f32>,
};

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};
 

struct VertexOutput {
    @builtin(position) clip_space_target_eye: vec4<f32>,
    @location(0) tex_pos: vec2<f32>,
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
    out.tex_pos = model.tex_pos;
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

@group(4)@binding(0)
var t_model: texture_2d<f32>;
@group(4)@binding(1)
var s_model: sampler;


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

    let sampled_texture = textureSample(t_model, s_model, in.tex_pos);
    var glitch_mask_alpha = textureSample(t_glitch_area, s_glitch_area, vec2<f32>(u,v)).r;
    glitch_mask_alpha = glitch_mask_alpha * glitch_mask_alpha;
    if( glitch_mask_alpha > 0.95 ) {
        return sampled_texture;
    } else {
        // interpolate 
        return glitch_mask_alpha * sampled_texture + (1.0 - glitch_mask_alpha) * random_pattern(vec2<f32>(in.ndc_space_left_eye.x, in.ndc_space_left_eye.y));
    }
}

fn random_pattern(uv: vec2<f32>) -> vec4<f32> {
    let step_num = 24.0;
    let x = steps(uv.x, step_num) + glitch_area.time * 0.005;
    var y = steps(uv.y, step_num) + glitch_area.time * 0.005;
    let y_offset = noise(vec2f(x,y) * 10.0);
    y = y + y_offset * 0.1;
 
    var r = noise(vec2f(x,y) * 3.0) * 0.8;
    var g = noise(vec2f(x,y) * 3.0 + vec2f(0.1, 0.1) * glitch_area.visibility * 7.0) * 0.8;
    var b = noise(vec2f(x,y) * 3.0 + vec2f(0.2, 0.2) * glitch_area.visibility * 7.0) * 0.8;

    r = r + noise(vec2f(x,y) * 15.0) * 0.6;
    g = g + noise(vec2f(x,y) * 15.0 + vec2f(0.1, 0.1) * glitch_area.visibility * 7.0) * 0.6;
    b = b + noise(vec2f(x,y) * 15.0 + vec2f(0.2, 0.2) * glitch_area.visibility * 7.0) * 0.6;

    let r_visibility = max(glitch_area.visibility, 0.2);
    let g_visibility = glitch_area.visibility;
    let b_visibility = glitch_area.visibility;

    r = r * r_visibility;
    g = g * g_visibility;
    b = b * b_visibility;
    return vec4<f32>(r*r, g*g, b*b, 1.0);
}

// https://gist.github.com/munrocket/236ed5ba7e409b8bdf1ff6eca5dcdc39
fn noise(n: vec2f) -> f32 {
    let d = vec2f(0., 1.);
    let b = floor(n);
    let f = smoothstep(vec2f(0.), vec2f(1.), fract(n));
    return mix(mix(rand22(b), rand22(b + d.yx), f.x), mix(rand22(b + d.xy), rand22(b + d.yy), f.x), f.y) - 0.1;
}

fn rand22(n: vec2f) -> f32 { return fract(sin(dot(n, vec2f(12.9898, 4.1414))) * 43758.5453); }

fn steps(input: f32, steps: f32) -> f32 {
    return floor(input * steps) / steps;
}