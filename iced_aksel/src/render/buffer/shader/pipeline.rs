use iced_graphics::text::cosmic_text::SwashCache;
use iced_wgpu::{primitive::Pipeline, wgpu};

use super::{atlas::TextureAtlas, data};

pub const WGSL_FILE: &str = include_str!("shader.wgsl");
pub const WGSL_SOURCE: wgpu::ShaderSource =
    wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(WGSL_FILE));

pub const LABEL_MODULE: &str = "Aksel Shader Module";
pub const LABEL_VERTEX_BUFFER: &str = "Aksel Vertex Buffer";
pub const LABEL_BIND_GROUP: &str = "Aksel Bind Group";

pub const VERTEX_BUFFER_INIT_CAPACITY: usize = 100;
pub const VERTEX_BUFFER_SIZE: usize =
    VERTEX_BUFFER_INIT_CAPACITY * std::mem::size_of::<data::UnifiedVertex>();

pub struct AkselPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,

    pub sampler: wgpu::Sampler,

    pub vertex_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,

    pub swash_cache: SwashCache,
    pub atlas: TextureAtlas,

    pub vertex_count: u32,
    pub vertex_capacity: usize,
}

impl Pipeline for AkselPipeline {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self
    where
        Self: Sized,
    {
        // Init module and buffers
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(LABEL_MODULE),
            source: WGSL_SOURCE,
        });
        let uniform_buffer = data::create_uniform_buffer(device);
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(LABEL_VERTEX_BUFFER),
            size: VERTEX_BUFFER_SIZE as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Init text caches
        let swash_cache = SwashCache::new();
        let atlas = TextureAtlas::new(device, queue);

        let (pipeline, bind_group_layout) = data::create_pipeline(device, format, &shader_module);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(LABEL_BIND_GROUP),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            pipeline,
            bind_group,
            sampler,
            vertex_buffer,
            uniform_buffer,
            swash_cache,
            atlas,
            vertex_count: 0,
            vertex_capacity: VERTEX_BUFFER_INIT_CAPACITY,
        }
    }
}
