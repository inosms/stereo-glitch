use game_objects::{glitch_area::GlitchAreaVisibilityDTO, time_keeper::TimeKeeper};
use level_loader::ParsedLevel;
use mesh::{InstanceRaw, Vertex};
use object_types::BlockType;
use rapier3d::na::ComplexField;
use std::{
    collections::{HashMap, HashSet},
    iter,
};

use wgpu::util::DeviceExt;

use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

mod command;
mod game;
mod level_loader;
mod mesh;
mod object_types;
mod physics;
mod stereo_camera;
mod texture;
mod level_compressor;
mod game_objects;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

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

    stereo_camera_uniform: stereo_camera::StereoCameraUniform,
    stereo_camera_buffer: wgpu::Buffer,
    stereo_camera_bind_group: wgpu::BindGroup,
    stereo_camera_left_target_buffer: wgpu::Buffer,
    stereo_camera_left_target_bind_group: wgpu::BindGroup,
    stereo_camera_right_target_buffer: wgpu::Buffer,
    stereo_camera_right_target_bind_group: wgpu::BindGroup,

    glitch_fragment_data_buffer: wgpu::Buffer,
    glitch_fragment_data_bind_group: wgpu::BindGroup,

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

        let mut mesh_store = mesh::MeshStore::new();
        let initial_instance_buffer_size: i32 = 1;
        let wall_mesh = mesh_store.add_mesh(mesh::Mesh::new_cube_with_color(
            &device,
            [0.0, 0.5, 0.0],
            initial_instance_buffer_size as usize,
        ));
        let floor_mesh = mesh_store.add_mesh(mesh::Mesh::new_cube_with_color_and_scale(
            &device,
            [0.0, 1.0, 0.0],
            1.0,
            1.0,
            8.0,
            initial_instance_buffer_size as usize,
        ));
        let player_mesh = mesh_store.add_mesh(mesh::Mesh::new_cube_with_color_and_scale(
            &device,
            [0.0, 0.0, 1.0],
            0.8,
            0.8,
            1.0,
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
        let box_mesh = mesh_store.add_mesh(mesh::Mesh::new_cube_with_color(
            &device,
            [0.2, 0.2, 0.2],
            initial_instance_buffer_size as usize,
        ));
        let trigger_mesh = mesh_store.add_mesh(mesh::Mesh::new_cube_with_color_and_scale(
            &device,
            [0.3, 0.3, 0.3],
            1.0,
            1.0,
            0.1,
            initial_instance_buffer_size as usize,
        ));
        let charge_mesh = mesh_store.add_mesh(mesh::Mesh::new_cube_with_color_and_scale(
            &device,
            [1.0, 1.0, 1.0],
            1.0,
            1.0,
            1.0,
            initial_instance_buffer_size as usize,
        ));
        let static_enemy_mesh = mesh_store.add_mesh(mesh::Mesh::new_cube_with_color_and_scale(
            &device,
            [1.0, 0.0, 0.0],
            1.0,
            1.0,
            1.5,
            initial_instance_buffer_size as usize,
        ));
        let linear_enemy_mesh = mesh_store.add_mesh(mesh::Mesh::new_cube_with_color_and_scale(
            &device,
            [1.0, 0.5, 0.0],
            1.0,
            1.0,
            1.0,
            initial_instance_buffer_size as usize,
        ));

        let handle_store = HashMap::from_iter(vec![
            (BlockType::Wall, wall_mesh),
            (BlockType::FloorNormal, floor_mesh),
            (BlockType::Player, player_mesh),
            (BlockType::Goal, goal_mesh),
            (BlockType::Door, door_mesh),
            (BlockType::Box, box_mesh),
            (BlockType::Trigger, trigger_mesh),
            (BlockType::Charge, charge_mesh),
            (BlockType::StaticEnemy, static_enemy_mesh),
            (BlockType::LinearEnemy, linear_enemy_mesh),
        ]);

        let game_world = game::GameWorld::new(handle_store);

        let stereo_camera = game_world.get_camera();
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

        let glitch_fragment_data_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Glitch Fragment Data Buffer"),
                contents: bytemuck::cast_slice(&[GlitchAreaVisibilityDTO::new(0.0, 0.0)]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let glitch_fragment_data_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("glitch_fragment_data_bind_group_layout"),
            });

        let glitch_fragment_data_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &glitch_fragment_data_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: glitch_fragment_data_buffer.as_entire_binding(),
            }],
            label: Some("glitch_fragment_data_bind_group"),
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
                    * 4 * 4 * 4
            ],
            ParsedLevel::MAX_LEVEL_WIDTH_AND_HEIGHT as u32 * 4,
            ParsedLevel::MAX_LEVEL_WIDTH_AND_HEIGHT as u32 * 4,
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
                    &glitch_fragment_data_bind_group_layout,
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

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            stereo_camera_uniform,
            stereo_camera_buffer,
            stereo_camera_bind_group,
            stereo_camera_left_target_buffer,
            stereo_camera_left_target_bind_group,
            stereo_camera_right_target_buffer,
            stereo_camera_right_target_bind_group,
            glitch_fragment_data_buffer,
            glitch_fragment_data_bind_group,
            depth_texture,
            _clear_color: wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            glitch_area_texture_bind_group,
            glitch_area_texture,
            mesh_store,
            game_world,
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
            self.game_world
                .set_camera_aspect((self.config.width as f32 / 2.0) / self.config.height as f32);
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

    fn update(&mut self) {
        while let Some(command) = command::COMMANDS.pop() {
            log::debug!("Processing command: {:?}", command);
            match command {
                command::Command::LoadLevel(parsed_level) => {
                    self.glitch_area_texture.write_rgba8(
                        &self.queue,
                        &parsed_level.to_glitch_raw_rgba8(),
                        ParsedLevel::MAX_LEVEL_WIDTH_AND_HEIGHT as u32 * 4,
                        ParsedLevel::MAX_LEVEL_WIDTH_AND_HEIGHT as u32 * 4,
                    );
                    self.game_world.load_level(parsed_level);
                }
                command::Command::SetEyeDistance(distance) => {
                    self.game_world.set_eye_distance(distance);
                }
                command::Command::SetSize(width, height, scale_factor) => {
                    self.resize(winit::dpi::LogicalSize::new(width, height).to_physical(scale_factor as f64));
                }
                command::Command::JoystickInput(x, y) => {
                    self.game_world.move_player(cgmath::vec3(x, y, 0.0));
                }
                command::Command::ActionButtonPressed => {
                    self.game_world.player_pull_action();
                }
                command::Command::ActionButtonReleased => {
                    self.game_world.release_player_pull_action();
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

        self.stereo_camera_uniform
            .update_view_proj(&self.game_world.get_camera());
        self.queue.write_buffer(
            &self.stereo_camera_buffer,
            0,
            bytemuck::cast_slice(&[self.stereo_camera_uniform]),
        );

        let glitch_visibility_dto: GlitchAreaVisibilityDTO = self.game_world.ref_glitch_area_visibility().into();
        self.queue.write_buffer(
            &self.glitch_fragment_data_buffer,
            0,
            bytemuck::cast_slice(&[glitch_visibility_dto]),
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
                render_pass.set_bind_group(3, &self.glitch_fragment_data_bind_group, &[]);
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
                                if state.key_pressed.insert(*key_code)
                                    && key_code == &KeyCode::Enter
                                    || key_code == &KeyCode::Space
                                {
                                    state.game_world.player_pull_action()
                                }
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
                                if state.key_pressed.remove(key_code) && key_code == &KeyCode::Enter
                                    || key_code == &KeyCode::Space
                                {
                                    state.game_world.release_player_pull_action()
                                }
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
