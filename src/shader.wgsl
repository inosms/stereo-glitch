// Vertex shader

// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
    left: i32,
};
@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

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
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) clip_space_pos: vec3<f32>,
    // -1 for left eye, 1 for right eye
    @location(2) camera_left: f32,
    @location(3) world_space_pos: vec4<f32>,
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
    var out: VertexOutput;
    out.color = model.color;
    out.world_space_pos = model_matrix * vec4<f32>(model.position, 1.0);
    out.clip_position = camera.view_proj * out.world_space_pos;
    out.clip_position.x /= out.clip_position.w;
    out.clip_position.y /= out.clip_position.w;
    out.clip_position.z /= out.clip_position.w;
    out.clip_position.w = 1.0;

    out.clip_position.x /= 2.0;
    out.clip_position.x += 0.5 * f32(camera.left);
    out.camera_left = f32(camera.left);
    out.clip_space_pos = vec3<f32>(out.clip_position.x, out.clip_position.y, out.clip_position.z);
    return out;
}


@group(1)@binding(0)
var t_glitch_area: texture_2d<f32>;
@group(1)@binding(1)
var s_glitch_area: sampler;

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // if out of bounds for each eye discard
    if ((in.clip_space_pos.x * in.camera_left) < 0.0 || (in.clip_space_pos.x * in.camera_left) > 1.0) {
        discard;
    }

    let w = 3.0;
    let h = 3.0;

    let u = in.world_space_pos.x / w;
    let v = in.world_space_pos.y * -1.0 / h;

    let glitch_mask_color = textureSample(t_glitch_area, s_glitch_area, vec2<f32>(u,v));

    if( glitch_mask_color.r > 0.5 ) {
        return vec4<f32>(in.color, 1.0);
    } else {
        return vec4<f32>(0.0,0.0,0.0,1.0);
    }
}