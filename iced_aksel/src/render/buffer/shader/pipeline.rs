use std::sync::atomic::AtomicU64;

use iced_graphics::text::cosmic_text;
use iced_wgpu::{primitive::Pipeline, wgpu};

use super::{atlas::TextureAtlas, data};

pub const RENDERER_FILE: &str = include_str!("shader_renderer.wgsl");
pub const RENDERER_SOURCE: wgpu::ShaderSource =
    wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(RENDERER_FILE));

pub const BLIT_FILE: &str = include_str!("shader_blit.wgsl");
pub const BLIT_SOURCE: wgpu::ShaderSource =
    wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(BLIT_FILE));

pub const LABEL_RENDERER_MODULE: &str = "Aksel Renderer Shader Module";
pub const LABEL_RENDERER_BIND_GROUP: &str = "Aksel Renderer Bind Group";
pub const LABEL_RENDERER_VERTEX_BUFFER: &str = "Aksel Renderer Vertex Buffer";
pub const LABEL_BLIT_MODULE: &str = "Aksel Blit Shader Module";
pub const LABEL_BLIT_BIND_GROUP_LAYOUT: &str = "Aksel Blit Bind Group Layout";
pub const LABEL_BLIT_PIPELINE_LAYOUT: &str = "Aksel Blit Pipeline Layout";
pub const LABEL_BLIT_PIPELINE: &str = "Aksel Blit Pipeline";
pub const LABEL_MSAA_TEXTURE: &str = "Aksel MSAA Texture";

pub const VERTEX_BUFFER_INIT_CAPACITY: usize = 100;
pub const VERTEX_BUFFER_SIZE: usize =
    VERTEX_BUFFER_INIT_CAPACITY * std::mem::size_of::<data::UnifiedVertex>();

pub const MSAA_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
pub const MSAA_SAMPLE_COUNT: u32 = 4;

pub struct AkselPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,

    pub sampler: wgpu::Sampler,

    pub text_buffer: cosmic_text::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,

    pub atlas: TextureAtlas,

    pub vertex_count: u32,
    pub vertex_capacity: usize,

    // MSAA
    pub sample_count: u32,
    pub format: wgpu::TextureFormat,
    pub msaa_view: Option<wgpu::TextureView>,
    pub msaa_texture: Option<wgpu::Texture>,

    // Caching
    pub cache_texture: Option<wgpu::Texture>,
    pub cache_view: Option<wgpu::TextureView>,
    pub cache_bind_group: Option<wgpu::BindGroup>,
    pub cache_size: (u32, u32),
    pub cache_version: AtomicU64,

    // Blit pipeline for cache
    pub blit_pipeline: wgpu::RenderPipeline,
    pub blit_bind_group_layout: wgpu::BindGroupLayout,
}

impl Pipeline for AkselPipeline {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self
    where
        Self: Sized,
    {
        // Init module and buffers
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(LABEL_RENDERER_MODULE),
            source: RENDERER_SOURCE,
        });
        let uniform_buffer = data::create_uniform_buffer(device);
        let vertex_buffer =
            data::create_renderer_vertex_buffer(device, VERTEX_BUFFER_INIT_CAPACITY);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Init text caches
        let atlas = TextureAtlas::new(device, queue);

        // Use the iced-provided format for all rendering to avoid color space conversion issues
        let (pipeline, bind_group_layout) =
            data::create_renderer_pipeline(device, format, &shader_module, MSAA_SAMPLE_COUNT);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(LABEL_RENDERER_BIND_GROUP),
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

        let blit_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(LABEL_BLIT_MODULE),
            source: BLIT_SOURCE,
        });

        let blit_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(LABEL_BLIT_BIND_GROUP_LAYOUT),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let blit_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(LABEL_BLIT_PIPELINE_LAYOUT),
            bind_group_layouts: &[&blit_bind_group_layout],
            push_constant_ranges: &[],
        });

        let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(LABEL_BLIT_PIPELINE),
            layout: Some(&blit_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &blit_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &blit_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(), // No MSAA for blit (count: 1)
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group,
            sampler,
            // Init empty text buffer
            text_buffer: cosmic_text::Buffer::new_empty(cosmic_text::Metrics {
                font_size: 1.0,
                line_height: 1.0,
            }),
            vertex_buffer,
            uniform_buffer,
            atlas,
            vertex_count: 0,
            vertex_capacity: VERTEX_BUFFER_INIT_CAPACITY,

            sample_count: MSAA_SAMPLE_COUNT,
            format, // Use iced format for all textures to avoid color space issues
            msaa_view: None,
            msaa_texture: None,

            cache_texture: None,
            cache_view: None,
            cache_bind_group: None,
            cache_size: (0, 0),
            cache_version: AtomicU64::new(0),

            blit_pipeline, // Uses same format as renderer for consistent color space
            blit_bind_group_layout,
        }
    }
}
