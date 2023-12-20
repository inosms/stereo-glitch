use std::io::{BufReader, Cursor};

use wgpu::util::DeviceExt;

use crate::{game_objects::position::Position, mesh::InstanceRaw, texture::Texture};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl ModelVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct Material {
    pub diffuse_texture: Texture,
    pub bind_group: wgpu::BindGroup,
}

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

pub struct Model {
    pub mesh: Mesh,
    pub material: Material,

    instance_buffer_size: usize,
    instances_used_num: usize,
    instance_buffer: wgpu::Buffer,
}

impl Model {
    pub fn update_instance_buffer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[&Position],
    ) {
        let instance_data = instances
            .iter()
            .map(|&pos| InstanceRaw::from(pos))
            .collect::<Vec<_>>();

        self.instances_used_num = instances.len();

        if self.instance_buffer_size < instances.len() {
            log::info!(
                "Will recreate buffer. Current buffer of size {} is too small for {} instances",
                self.instance_buffer_size,
                instances.len()
            );
            self.instance_buffer.destroy();
            self.instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
            self.instance_buffer_size = instances.len();
            log::info!("Recreated index buffer to size {}", instances.len());
        } else {
            queue.write_buffer(
                &self.instance_buffer,
                0,
                bytemuck::cast_slice(&instance_data),
            );
        }
    }

    pub fn render_instances<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_bind_group(4, &self.material.bind_group, &[]);
        render_pass.draw_indexed(
            0..self.mesh.num_elements,
            0,
            0..self.instances_used_num as u32,
        );
    }
}

pub fn load_model(
    model_obj_file_raw: &[u8],
    model_texture_file_raw: &[u8],
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<Model> {
    let obj_cursor = Cursor::new(model_obj_file_raw);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, _) = tobj::load_obj_buf(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |_| {
            // we pass the raw bytes of the texture file to the loader function so no loading is needed
            Ok((Default::default(), Default::default()))
        },
    )?;

    let model_texture =
        Texture::from_raw(device, queue, model_texture_file_raw, Some("model texture"));

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&model_texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&model_texture.sampler),
            },
        ],
        label: None,
    });

    let material = Material {
        diffuse_texture: model_texture,
        bind_group,
    };

    let m = models.get(0).expect("No model loaded");

    let vertices = (0..m.mesh.positions.len() / 3)
        .map(|i| ModelVertex {
            position: [
                m.mesh.positions[i * 3],
                // rotate the model by 90 degrees around the x axis
                -m.mesh.positions[i * 3 + 2],
                m.mesh.positions[i * 3 + 1], 
            ],
            tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
        })
        .collect::<Vec<_>>();

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&m.mesh.indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    let mesh = Mesh {
        vertex_buffer,
        index_buffer,
        num_elements: m.mesh.indices.len() as u32,
        material: m.mesh.material_id.unwrap_or(0),
    };

    let instances: Vec<Position> = vec![Position::default(); 1];
    let instance_data = instances.iter().map(InstanceRaw::from).collect::<Vec<_>>();
    let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Instance Buffer"),
        contents: bytemuck::cast_slice(&instance_data),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });

    Ok(Model {
        mesh,
        material,
        instance_buffer_size: 1,
        instances_used_num: 0,
        instance_buffer,
    })
}

pub struct ModelStore {
    models: Vec<Model>,
}

impl ModelStore {
    pub fn new() -> Self {
        Self { models: Vec::new() }
    }

    pub fn add_model(&mut self, model: Model) -> ModelHandle {
        let handle = ModelHandle::from(self.models.len());
        self.models.push(model);
        handle
    }

    pub fn get(&self, handle: ModelHandle) -> Option<&Model> {
        self.models.get(handle.index())
    }

    pub fn get_mut(&mut self, handle: ModelHandle) -> Option<&mut Model> {
        self.models.get_mut(handle.index())
    }

    pub fn iter_handles(&self) -> impl Iterator<Item = ModelHandle> {
        (0..self.models.len()).map(ModelHandle::from)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ModelHandle {
    index: usize,
}

impl ModelHandle {
    pub fn from(index: usize) -> Self {
        Self { index }
    }

    pub fn index(&self) -> usize {
        self.index
    }
}
