use bytemuck::{Pod, Zeroable};
use iced_wgpu::wgpu;
use image::GenericImageView;

use crate::render::buffer::shader::pipeline::LABEL_RENDERER_VERTEX_BUFFER;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct UnifiedVertex {
    // We use [f32; N] instead of vector to ensure memory layout explicitly
    pub position: [f32; 2],      // Vertex position (screen space for quads)
    pub color: [f32; 4],          // RGBA color
    pub uv: [f32; 2],             // UV coordinates for texture atlas
    pub primitive_type: u32,      // Type of primitive (0=MSDF text, 1=SDF line, 2=SDF circle, etc.)
    pub param0: [f32; 4],         // Shape-specific parameters
    pub param1: [f32; 4],         // Additional shape-specific parameters
}

// Primitive type constants
pub const PRIM_TYPE_MSDF_TEXT: u32 = 0;
pub const PRIM_TYPE_SDF_LINE: u32 = 1;
pub const PRIM_TYPE_SDF_CIRCLE: u32 = 2;
pub const PRIM_TYPE_SDF_ROUNDED_RECT: u32 = 3;
pub const PRIM_TYPE_SDF_ELLIPSE: u32 = 4;

impl UnifiedVertex {
    pub const fn new_shape(pos: [f32; 2], color: [f32; 4], white_pixel_uv: [f32; 2]) -> Self {
        Self {
            position: [pos[0], pos[1]],
            color,
            uv: white_pixel_uv,
            primitive_type: PRIM_TYPE_MSDF_TEXT, // Legacy - use white pixel as texture
            param0: [0.0, 0.0, 0.0, 0.0],
            param1: [0.0, 0.0, 0.0, 0.0],
        }
    }

    pub const fn new_msdf_text(pos: [f32; 2], color: [f32; 4], uv: [f32; 2]) -> Self {
        Self {
            position: pos,
            color,
            uv,
            primitive_type: PRIM_TYPE_MSDF_TEXT,
            param0: [0.0, 0.0, 0.0, 0.0],
            param1: [0.0, 0.0, 0.0, 0.0],
        }
    }

    pub fn new_sdf_line(
        pos: [f32; 2],
        color: [f32; 4],
        line_start: [f32; 2],
        line_end: [f32; 2],
        width: f32,
        rotation_radians: f32,
    ) -> Self {
        let cos_angle = rotation_radians.cos();
        let sin_angle = rotation_radians.sin();
        Self {
            position: pos,
            color,
            uv: [0.0, 0.0], // Not used for SDF shapes
            primitive_type: PRIM_TYPE_SDF_LINE,
            param0: [line_start[0], line_start[1], line_end[0], line_end[1]],
            param1: [width, cos_angle, sin_angle, 0.0],
        }
    }

    pub fn new_sdf_circle(
        pos: [f32; 2],
        color: [f32; 4],
        center: [f32; 2],
        radius: f32,
        rotation_radians: f32,
    ) -> Self {
        let cos_angle = rotation_radians.cos();
        let sin_angle = rotation_radians.sin();
        Self {
            position: pos,
            color,
            uv: [0.0, 0.0],
            primitive_type: PRIM_TYPE_SDF_CIRCLE,
            param0: [center[0], center[1], radius, 0.0],
            param1: [cos_angle, sin_angle, 0.0, 0.0],
        }
    }

    pub fn new_sdf_ellipse(
        pos: [f32; 2],
        color: [f32; 4],
        center: [f32; 2],
        radii: [f32; 2],
        rotation_radians: f32,
    ) -> Self {
        let cos_angle = rotation_radians.cos();
        let sin_angle = rotation_radians.sin();
        Self {
            position: pos,
            color,
            uv: [0.0, 0.0],
            primitive_type: PRIM_TYPE_SDF_ELLIPSE,
            param0: [center[0], center[1], radii[0], radii[1]],
            param1: [cos_angle, sin_angle, 0.0, 0.0],
        }
    }

    pub fn new_sdf_rounded_rect(
        pos: [f32; 2],
        color: [f32; 4],
        center: [f32; 2],
        half_size: [f32; 2],
        corner_radius: f32,
        rotation_radians: f32,
    ) -> Self {
        let cos_angle = rotation_radians.cos();
        let sin_angle = rotation_radians.sin();
        Self {
            position: pos,
            color,
            uv: [0.0, 0.0],
            primitive_type: PRIM_TYPE_SDF_ROUNDED_RECT,
            param0: [center[0], center[1], half_size[0], half_size[1]],
            param1: [corner_radius, cos_angle, sin_angle, 0.0],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Uniforms {
    pub screen_width: f32,
    pub screen_height: f32,
    // Padding to align to 16 bytes
    pub _padding1: f32,
    pub _padding2: f32,
}

pub fn create_renderer_vertex_buffer(
    device: &wgpu::Device,
    needed_capacity: usize,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(LABEL_RENDERER_VERTEX_BUFFER),
        size: (needed_capacity * std::mem::size_of::<UnifiedVertex>()) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

pub fn create_mini_atlas(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::TextureView {
    let size = wgpu::Extent3d {
        width: 2,
        height: 2,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Mini Atlas"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    // The Data: 4 Pixels (R, G, B, A)
    // 1. Top-Left: WHITE (255, 255, 255, 255)
    // 2. Top-Right: RED   (255, 0, 0, 255)
    // 3. Bottom-Left: BLUE (0, 0, 255, 255)
    // 4. Bottom-Right: GREEN (0, 255, 0, 255)
    let data: [u8; 16] = [
        255, 255, 255, 255, 255, 0, 0, 255, 0, 0, 255, 255, 0, 255, 0, 255,
    ];

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(8), // 2 pixels * 4 bytes = 8 bytes per row
            rows_per_image: None,
        },
        size,
    );

    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

pub fn create_uniform_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Aksel Uniform Buffer"),
        size: std::mem::size_of::<Uniforms>() as u64, // 8 bytes (two f32s)
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

pub fn create_renderer_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    shader_module: &wgpu::ShaderModule,
    sample_count: u32,
) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Aksel Renderer Bind Group Layout"),
        entries: &[
            // Binding 0: The Uniform Buffer (Screen Size)
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX, // Only used in Vertex Shader
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Binding 1: The Texture (Atlas)
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // Binding 2: The Sampler
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Aksel Renderer Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Aksel Renderer Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader_module,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<UnifiedVertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    // Location 0: Position (Float32x2) - offset 0
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    },
                    // Location 1: Color (Float32x4) - offset 8
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 8,
                        shader_location: 1,
                    },
                    // Location 2: UV (Float32x2) - offset 24
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 24,
                        shader_location: 2,
                    },
                    // Location 3: Primitive Type (Uint32) - offset 32
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Uint32,
                        offset: 32,
                        shader_location: 3,
                    },
                    // Location 4: Param0 (Float32x4) - offset 36
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 36,
                        shader_location: 4,
                    },
                    // Location 5: Param1 (Float32x4) - offset 52
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 52,
                        shader_location: 5,
                    },
                ],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader_module,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                // 3. ENABLE BLENDING (Crucial for Text and Glass)
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    (pipeline, bind_group_layout)
}

// Modified helper for internal use
pub fn create_depth_texture_with_size(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> wgpu::TextureView {
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT, // Only needs to be an attachment
        view_formats: &[],
    });
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

pub fn create_texture_from_bytes(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bytes: &[u8],
    label: &str,
) -> wgpu::TextureView {
    // 1. Decode the image using the 'image' crate
    let img = image::load_from_memory(bytes).expect("Failed to load image");
    let rgba = img.to_rgba8();
    let (width, height) = img.dimensions();

    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    // 2. Create the GPU Texture
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    // 3. Upload the pixels
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width), // 4 bytes per pixel * width
            rows_per_image: None,
        },
        size,
    );

    texture.create_view(&wgpu::TextureViewDescriptor::default())
}
