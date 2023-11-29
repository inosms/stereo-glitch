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
    @location(4) camera_space_pos: vec4<f32>,
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
    out.camera_space_pos = camera.view_proj * out.world_space_pos;
    out.clip_position = out.camera_space_pos;
    out.clip_position.x /= out.clip_position.w;
    out.clip_position.y /= out.clip_position.w;
    out.clip_position.z /= out.clip_position.w;
    out.clip_position.w = 1.0;

    out.clip_position.x /= 2.0;
    out.clip_position.x += 0.5 * f32(camera.left);

    out.clip_space_pos = vec3<f32>(out.clip_position.x, out.clip_position.y, out.clip_position.z);
    out.camera_left = f32(camera.left);
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

    let w = 3.0; // TODO
    let h = 3.0; // TODO

    let u = in.world_space_pos.x / w;
    let v = in.world_space_pos.y * -1.0 / h;

    let glitch_mask_color = textureSample(t_glitch_area, s_glitch_area, vec2<f32>(u,v));

    if( glitch_mask_color.r > 0.5 ) {
        return vec4<f32>(in.color, 1.0);
    } else {
        // map x: [-1, 0] to [0, 1]
        // keep [0, 1] as is
        let x = (in.clip_space_pos.x + 1.0) % 1.0;
        let y = in.clip_space_pos.y;

        // only displace the right eye
        var displacement = 0.0;
        let depth_factor = 0.17;
        if( in.clip_space_pos.x >= 0.0 ) {
            let far = 10.0;
            let near = 4.0;

            let depth_0_to_1 = clamp((in.camera_space_pos.z - near) / (far - near), 0.0, 1.0);
            displacement = (1.0 - depth_0_to_1) * depth_factor;
        }
        
       return random_pattern(vec2<f32>(x + displacement, y));
    }
}

fn linearize_depth(d: f32, zNear: f32, zFar: f32) -> f32 {
    return zNear * zFar / (zFar + d * (zNear - zFar));
}


fn random_pattern(uv: vec2<f32>) -> vec4<f32> {
    // let value = 0.1*sin(uv.x*0.01) + 0.3*cos(uv.y*0.001+2.1) + 0.01*sin(uv.x*3.0+0.76) + 0.01*sin(uv.y*0.04+1.0) + 0.1*sin(uv.y*8.0+1.0)
    // + 0.1*sin(uv.x*100.0) + 0.3*cos(uv.y*7.+8.1) + 0.01*sin(uv.x*2.0 + 5.0);

    let value = noise(uv * 100.0);
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
