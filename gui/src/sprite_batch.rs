use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SpriteVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}

impl SpriteVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SpriteInstance {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub uv_offset: [f32; 2],
    pub uv_size: [f32; 2],
    pub color: [f32; 4],
}

impl SpriteInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        2 => Float32x2,  // position
        3 => Float32x2,  // size
        4 => Float32x2,  // uv_offset
        5 => Float32x2,  // uv_size
        6 => Float32x4   // color
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct SpriteBatch {
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instances: Vec<SpriteInstance>,
    max_instances: usize,
}

impl SpriteBatch {
    const QUAD_VERTICES: &[SpriteVertex] = &[
        SpriteVertex {
            position: [0.0, 0.0],
            tex_coords: [0.0, 0.0],
        },
        SpriteVertex {
            position: [1.0, 0.0],
            tex_coords: [1.0, 0.0],
        },
        SpriteVertex {
            position: [0.0, 1.0],
            tex_coords: [0.0, 1.0],
        },
        SpriteVertex {
            position: [1.0, 1.0],
            tex_coords: [1.0, 1.0],
        },
    ];

    const QUAD_INDICES: &[u16] = &[0, 1, 2, 1, 3, 2];

    pub fn new(device: &wgpu::Device, max_instances: usize) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Vertex Buffer"),
            contents: bytemuck::cast_slice(Self::QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite Instance Buffer"),
            size: (std::mem::size_of::<SpriteInstance>() * max_instances) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            vertex_buffer,
            instance_buffer,
            instances: Vec::with_capacity(max_instances),
            max_instances,
        }
    }

    pub fn add_sprite(
        &mut self,
        position: [f32; 2],
        size: [f32; 2],
        uv_offset: [f32; 2],
        uv_size: [f32; 2],
        color: [f32; 4],
    ) {
        if self.instances.len() < self.max_instances {
            self.instances.push(SpriteInstance {
                position,
                size,
                uv_offset,
                uv_size,
                color,
            });
        }
    }

    pub fn flush(&mut self, queue: &wgpu::Queue) {
        if !self.instances.is_empty() {
            queue.write_buffer(
                &self.instance_buffer,
                0,
                bytemuck::cast_slice(&self.instances),
            );
        }
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if !self.instances.is_empty() {
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.draw(0..6, 0..self.instances.len() as u32);
        }
    }

    pub fn clear(&mut self) {
        self.instances.clear();
    }

    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }
}
