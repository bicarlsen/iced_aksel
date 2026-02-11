use iced_core::widget::shader;
use iced_graphics::text::cosmic_text::SwashCache;
use iced_wgpu::wgpu;

mod atlas;
mod data;

use atlas::FontAtlas;

const INTIIAL_CAPACITY: usize = 100;
const LABEL_MODULE: &str = "Aksel Shader Module";
const LABEL_VERTEX_BUFFER: &str = "Aksel Vertex Buffer";

pub struct AkselPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,

    pub sampler: wgpu::Sampler,

    pub vertex_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,

    pub swash_cache: SwashCache,
    pub atlas: FontAtlas,

    pub vertex_count: u32,
    pub vertex_capacity: usize,
}

impl AkselPipeline {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self
    where
        Self: Sized,
    {
        // Init module and pipeline
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(LABEL_MODULE),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "shader.wgsl"
            ))),
        });

        let (pipeline, bind_group_layout) = data::create_pipeline(device, format, &shader_module);
        let uniform_buffer = data::create_uniform_buffer(device);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(LABEL_VERTEX_BUFFER),
            size: (INTIIAL_CAPACITY * std::mem::size_of::<data::UnifiedVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
    }
}
