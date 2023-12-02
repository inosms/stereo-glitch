use cgmath::InnerSpace;
use level_loader::{BlockType, ParsedLevel};
use mesh::InstanceRaw;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    iter,
    sync::Mutex,
};
use stereo_camera::StereoCamera;
use wgpu::util::DeviceExt;

use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

mod command;
mod game;
mod level_loader;
mod mesh;
mod physics;
mod stereo_camera;
mod texture;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

// cube vertices
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [1.0, 1.0, 0.0],
        color: [1.0, 1.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0, 0.0],
        color: [1.0, 1.0, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, 1.0],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 1.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0, 1.0],
        color: [0.0, 0.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, 1.0],
        color: [0.0, 0.0, 1.0],
    },
];

const INDICES: &[u16] = &[
    0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 0, 1, 5, 0, 5, 4, 3, 2, 6, 3, 6, 7, 0, 3, 7, 0, 7, 4, 1, 2,
    6, 1, 6, 5,
];

struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // 1.
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        // 2.
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        // 3.
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
    left: i32,
    _pad1: u32,
    _pad2: u32,
    _pad3: u32,
}

impl CameraUniform {
    fn new(left: bool) -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
            left: if left { -1 } else { 1 },
            _pad1: 0,
            _pad2: 0,
            _pad3: 0,
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: Window,

    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,

    stereo_camera: stereo_camera::StereoCamera,
    stereo_camera_uniform: stereo_camera::StereoCameraUniform,
    stereo_camera_buffer: wgpu::Buffer,
    stereo_camera_bind_group: wgpu::BindGroup,
    stereo_camera_left_target_buffer: wgpu::Buffer,
    stereo_camera_left_target_bind_group: wgpu::BindGroup,
    stereo_camera_right_target_buffer: wgpu::Buffer,
    stereo_camera_right_target_bind_group: wgpu::BindGroup,

    depth_texture: texture::Texture,

    glitch_area_texture_bind_group: wgpu::BindGroup,
    glitch_area_texture: texture::Texture,

    game_world: game::GameWorld,
    mesh_store: mesh::MeshStore,

    _clear_color: wgpu::Color,

    key_pressed: HashSet<KeyCode>,
}

impl State {
    async fn new(window: Window) -> Self {
        // dummy size for init. will be resized later
        let size = PhysicalSize::new(400, 200);

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                // Some(&std::path::Path::new("trace")), // Trace path
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let stereo_camera = StereoCamera::new(
            (0.0, 1.0, 2.0).into(),
            (0.0, 0.0, 0.0).into(),
            cgmath::Vector3::unit_z(),
            (config.width as f32 / 2.0) / config.height as f32,
            45.0,
            0.1,
            20.0,
            0.9,
        );
        let mut stereo_camera_uniform = stereo_camera::StereoCameraUniform::new();
        stereo_camera_uniform.update_view_proj(&stereo_camera);
        let stereo_camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Stereo Camera Buffer"),
            contents: bytemuck::cast_slice(&[stereo_camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let stereo_camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("stereo_camera_bind_group_layout"),
            });

        let stereo_camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &stereo_camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: stereo_camera_buffer.as_entire_binding(),
            }],
            label: Some("stereo_camera_bind_group"),
        });

        let stereo_camera_target_left =
            stereo_camera::RenderEyeTarget::new(stereo_camera::EyeTarget::Left);
        let stereo_camera_target_right =
            stereo_camera::RenderEyeTarget::new(stereo_camera::EyeTarget::Right);

        let stereo_camera_left_target_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Stereo Camera Left Target Buffer"),
                contents: bytemuck::cast_slice(&[stereo_camera_target_left]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let stereo_camera_right_target_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Stereo Camera Right Target Buffer"),
                contents: bytemuck::cast_slice(&[stereo_camera_target_right]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let stereo_camera_target_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("stereo_camera_target_bind_group_layout"),
            });

        let stereo_camera_left_target_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &stereo_camera_target_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: stereo_camera_left_target_buffer.as_entire_binding(),
                }],
                label: Some("stereo_camera_left_target_bind_group"),
            });

        let stereo_camera_right_target_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &stereo_camera_target_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: stereo_camera_right_target_buffer.as_entire_binding(),
                }],
                label: Some("stereo_camera_right_target_bind_group"),
            });

        let glitch_area_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("glitch_area_texture_bind_group_layout"),
            });

        // Initialize the texture with empty data
        let glitch_area_texture = texture::Texture::from_raw_rgba8(
            &device,
            &queue,
            &vec![
                0;
                ParsedLevel::MAX_LEVEL_WIDTH_AND_HEIGHT
                    * ParsedLevel::MAX_LEVEL_WIDTH_AND_HEIGHT
                    * 4
            ],
            ParsedLevel::MAX_LEVEL_WIDTH_AND_HEIGHT as u32,
            ParsedLevel::MAX_LEVEL_WIDTH_AND_HEIGHT as u32,
            None,
        );

        let glitch_area_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &glitch_area_texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&glitch_area_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&glitch_area_texture.sampler),
                },
            ],
            label: Some("glitch_area_texture_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &stereo_camera_bind_group_layout,
                    &stereo_camera_target_group_layout,
                    &glitch_area_texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, //Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = INDICES.len() as u32;

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let mut mesh_store = mesh::MeshStore::new();
        let initial_instance_buffer_size: i32 = 1;
        let wall_mesh = mesh_store.add_mesh(mesh::Mesh::new_cube_with_color(
            &device,
            [1.0, 0.0, 0.0],
            initial_instance_buffer_size as usize,
        ));
        let floor_mesh = mesh_store.add_mesh(mesh::Mesh::new_cube_with_color(
            &device,
            [0.0, 1.0, 0.0],
            initial_instance_buffer_size as usize,
        ));
        let player_mesh = mesh_store.add_mesh(mesh::Mesh::new_cube_with_color(
            &device,
            [0.0, 0.0, 1.0],
            initial_instance_buffer_size as usize,
        ));
        let goal_mesh = mesh_store.add_mesh(mesh::Mesh::new_cube_with_color(
            &device,
            [1.0, 1.0, 0.0],
            initial_instance_buffer_size as usize,
        ));
        let door_mesh = mesh_store.add_mesh(mesh::Mesh::new_cube_with_color(
            &device,
            [1.0, 0.0, 1.0],
            initial_instance_buffer_size as usize,
        ));

        let handle_store = HashMap::from_iter(vec![
            (BlockType::Wall, wall_mesh),
            (BlockType::FloorNormal, floor_mesh),
            (BlockType::Player, player_mesh),
            (BlockType::Goal, goal_mesh),
            (BlockType::Door, door_mesh),
        ]);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            stereo_camera,
            stereo_camera_uniform,
            stereo_camera_buffer,
            stereo_camera_bind_group,
            stereo_camera_left_target_buffer,
            stereo_camera_left_target_bind_group,
            stereo_camera_right_target_buffer,
            stereo_camera_right_target_bind_group,
            depth_texture,
            glitch_area_texture_bind_group,
            glitch_area_texture,
            mesh_store,
            game_world: game::GameWorld::new(handle_store),
            _clear_color: wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            key_pressed: Default::default(),
        }
    }

    fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.stereo_camera
                .set_aspect((self.config.width as f32 / 2.0) / self.config.height as f32);
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

    fn update(&mut self) {
        while let Some(command) = command::COMMANDS.pop() {
            log::info!("Processing command: {:?}", command);
            match command {
                command::Command::LoadLevel(parsed_level) => {
                    self.game_world.clear();
                    for ((x, y), cell) in parsed_level.iter_cells() {
                        self.game_world.add_cell(x, y, cell);
                    }
                    self.glitch_area_texture.write_rgba8(
                        &self.queue,
                        &parsed_level.to_glitch_raw_rgba8(),
                        ParsedLevel::MAX_LEVEL_WIDTH_AND_HEIGHT as u32,
                        ParsedLevel::MAX_LEVEL_WIDTH_AND_HEIGHT as u32,
                    );
                }
                command::Command::SetEyeDistance(distance) => {
                    self.stereo_camera.set_eye_distance(distance);
                }
                command::Command::SetSize(width, height) => {
                    self.resize(winit::dpi::PhysicalSize::new(width, height));
                }
                command::Command::JoystickInput(x, y) => {
                    self.game_world.move_player(cgmath::vec3(x, y, 0.0));
                }
            }
        }

        let mut direction = cgmath::vec3(0.0, 0.0, 0.0);
        let mut button_pressed = false;
        if self.key_pressed.contains(&KeyCode::KeyW) {
            direction += cgmath::vec3(0.0, 1.0, 0.0);
            button_pressed = true;
        }
        if self.key_pressed.contains(&KeyCode::KeyA) {
            direction += cgmath::vec3(-1.0, 0.0, 0.0);
            button_pressed = true;
        }
        if self.key_pressed.contains(&KeyCode::KeyS) {
            direction += cgmath::vec3(0.0, -1.0, 0.0);
            button_pressed = true;
        }
        if self.key_pressed.contains(&KeyCode::KeyD) {
            direction += cgmath::vec3(1.0, 0.0, 0.0);
            button_pressed = true;
        }
        if button_pressed {
            self.game_world.move_player(direction);
        }
        self.game_world.update();

        for mesh_handle in self.mesh_store.iter_handles() {
            let instances = self.game_world.iter_instances(mesh_handle);
            self.mesh_store.get_mut(mesh_handle).map(|mesh| {
                mesh.update_instance_buffer(&self.device, &self.queue, &instances);
            });
        }

        let time = 0.05 * (instant::now() / 1000.0) as f32;
        let radius = 10.0 as f32;
        self.stereo_camera.set_eye(cgmath::Point3::new(
            0.0 + time.sin() * radius,
            0.0 + time.cos() * radius,
            7.0,
        ));
        self.stereo_camera
            .set_target(cgmath::Point3::new(0.0, 0.0, 0.0));

        self.stereo_camera_uniform
            .update_view_proj(&self.stereo_camera);
        self.queue.write_buffer(
            &self.stereo_camera_buffer,
            0,
            bytemuck::cast_slice(&[self.stereo_camera_uniform]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass Left"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self._clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            for stereo_camera_target in vec![
                &self.stereo_camera_left_target_bind_group,
                &self.stereo_camera_right_target_bind_group,
            ] {
                render_pass.set_bind_group(0, &self.stereo_camera_bind_group, &[]);
                render_pass.set_bind_group(1, stereo_camera_target, &[]);
                render_pass.set_bind_group(2, &self.glitch_area_texture_bind_group, &[]);
                for mesh_handle in self.mesh_store.iter_handles() {
                    self.mesh_store.get(mesh_handle).map(|mesh| {
                        mesh.render_instances(&mut render_pass);
                    });
                }
            }
        }
        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("game-container")?;
                let canvas = web_sys::Element::from(window.canvas()?);
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = State::new(window).await;

    event_loop
        .run(move |event, elwt| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if window_id == state.window.id() => elwt.exit(),
                Event::AboutToWait => {
                    state.window.request_redraw();
                }
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == state.window().id() => {
                    // if !state.input(event) {
                    // UPDATED!
                    match event {
                        // WindowEvent::Resized(physical_size) => {
                        //     state.resize(*physical_size);
                        // }
                        // TODO
                        // WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        //     // new_inner_size is &&mut so w have to dereference it twice
                        //     state.resize(**new_inner_size);
                        // }
                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    physical_key,
                                    state: ElementState::Pressed,
                                    ..
                                },
                            ..
                        } => match &physical_key {
                            PhysicalKey::Code(key_code) => {
                                state.key_pressed.insert(*key_code);
                            }
                            _ => {}
                        },
                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    physical_key,
                                    state: ElementState::Released,
                                    ..
                                },
                            ..
                        } => match &physical_key {
                            PhysicalKey::Code(key_code) => {
                                state.key_pressed.remove(key_code);
                            }
                            _ => {}
                        },
                        WindowEvent::RedrawRequested => {
                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                // Reconfigure the surface if it's lost or outdated
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    state.resize(state.size)
                                }
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    panic!("Out of memory");
                                }

                                Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                            }
                        }
                        _ => {}
                    }
                    // }
                }
                _ => {}
            }
        })
        .expect("Failed to run event loop");
}
