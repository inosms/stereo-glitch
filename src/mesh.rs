use bevy_ecs::system::ExclusiveSystemParam;
use cgmath::prelude::*;
use wgpu::util::DeviceExt;

use crate::game::Position;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
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
// 1x1 cube that is centered around (0, 0, 0.5)
const CUBE_VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.5, 0.5, 0.0],
        color: [0.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.0],
        color: [0.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [0.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [0.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [0.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [0.0, 0.0, 0.0],
    },
];

const CUBE_INDICES: &[u16] = &[
    0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 0, 1, 5, 0, 5, 4, 3, 2, 6, 3, 6, 7, 0, 3, 7, 0, 7, 4, 1, 2,
    6, 1, 6, 5,
];

pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    instance_buffer_size: usize,
    instances_used_num: usize,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
}

impl Mesh {
    pub fn new(
        vertices: Vec<Vertex>,
        indices: Vec<u16>,
        device: &wgpu::Device,
        instance_buffer_size: usize,
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let instances: Vec<Position> = vec![Position::default(); instance_buffer_size];
        let instance_data = instances.iter().map(InstanceRaw::from).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            vertices,
            indices,
            instance_buffer_size: instance_buffer_size as usize,
            instances_used_num: 0,
            vertex_buffer,
            index_buffer,
            instance_buffer,
        }
    }

    pub fn new_cube(device: &wgpu::Device, instance_size: usize) -> Self {
        Self::new(
            CUBE_VERTICES.to_vec(),
            CUBE_INDICES.to_vec(),
            device,
            instance_size,
        )
    }

    pub fn new_cube_with_color(device: &wgpu::Device, color: [f32; 3], instance_size: usize) -> Self {
        let mut vertices = CUBE_VERTICES.to_vec();
        for vertex in vertices.iter_mut() {
            vertex.color = color;
        }
        Self::new(vertices, CUBE_INDICES.to_vec(), device, instance_size)
    }

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
            log::info!("Will recreate buffer. Current buffer of size {} is too small for {} instances", self.instance_buffer_size, instances.len());
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
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.draw_indexed(
            0..self.indices.len() as u32,
            0,
            0..self.instances_used_num as u32,
        );
    }
}

pub struct MeshStore {
    meshes: Vec<Mesh>,
}

impl MeshStore {
    pub fn new() -> Self {
        Self { meshes: Vec::new() }
    }

    pub fn add_mesh(&mut self, mesh: Mesh) -> Handle {
        let handle = Handle::from(self.meshes.len());
        self.meshes.push(mesh);
        handle
    }

    pub fn get(&self, handle: Handle) -> Option<&Mesh> {
        self.meshes.get(handle.index())
    }

    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut Mesh> {
        self.meshes.get_mut(handle.index())
    }

    pub fn iter_handles(&self) -> impl Iterator<Item = Handle> {
        (0..self.meshes.len()).map(Handle::from)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Handle {
    index: usize,
}

impl Handle {
    pub fn from(index: usize) -> Self {
        Self { index }
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl From<&Position> for InstanceRaw {
    fn from(position: &Position) -> Self {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(position.position)
                * cgmath::Matrix4::from(position.rotation))
            .into(),
        }
    }
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
