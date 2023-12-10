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
    visibility: f32,

    _padding_0: f32,
    _padding_1: f32,
    _padding_2: f32,
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
        return glitch_area.visibility * random_pattern(vec2<f32>(in.ndc_space_left_eye.x, in.ndc_space_left_eye.y));
    }
}

fn random_pattern(uv: vec2<f32>) -> vec4<f32> {
    // let value = 0.1*sin(uv.x*0.01) + 0.3*cos(uv.y*0.001+2.1) + 0.01*sin(uv.x*3.0+0.76) + 0.01*sin(uv.y*0.04+1.0) + 0.1*sin(uv.y*8.0+1.0)
    // + 0.1*sin(uv.x*100.0) + 0.3*cos(uv.y*7.+8.1) + 0.01*sin(uv.x*2.0 + 5.0);

    let value = noise(uv * 90.0);
    return vec4<f32>(value, value, value, 1.0);
}



fn rand(n: vec2<f32>) -> f32 { 
    return fract(sin(dot(n, vec2<f32>(12.9898, 4.1414))) * 43758.5453);
}

fn noise(p: vec2<f32>) -> f32{
    let ip = floor(p);
    var u = fract(p);
    u = u*u*(3.0-(2.0*u));
    
    let res = mix(
        mix(rand(ip),rand(ip+vec2<f32>(1.0,0.0)),u.x),
        mix(rand(ip+vec2<f32>(0.0,1.0)),rand(ip+vec2<f32>(1.0,1.0)),u.x),u.y);
    return res*res;
}
