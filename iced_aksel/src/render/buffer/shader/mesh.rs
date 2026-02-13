use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use super::atlas::TextureAtlas;
use super::data::{self, UnifiedVertex};
use super::pipeline::AkselPipeline;
use iced_core::{Color, text::Shaping};
use iced_graphics::text::{self, cosmic_text, font_system};
use iced_wgpu::wgpu;

use crate::render::Primitive;

static MESH_VERSION: AtomicU64 = AtomicU64::new(0);

type PackedColor = [f32; 4];

const TRANSPARENT: PackedColor = [0., 0., 0., 0.];
const INIT_CAPACITY: usize = 10_000;
const LABEL_CACHE_RENDER: &str = "Aksel Cache Render";

const fn pack_color(color: Color) -> PackedColor {
    [color.r, color.g, color.b, color.a]
}

#[derive(Default)]
struct VertexBuffer(Vec<UnifiedVertex>);

impl VertexBuffer {
    fn as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.0)
    }

    const fn len(&self) -> usize {
        self.0.len()
    }

    fn push_vertex(&mut self, position: [f32; 2], color: PackedColor, uv: [f32; 2]) {
        self.0.push(UnifiedVertex {
            position,
            color,
            uv,
            primitive_type: data::PRIM_TYPE_MSDF_TEXT, // Default to legacy texture rendering
            param0: [0.0, 0.0, 0.0, 0.0],
            param1: [0.0, 0.0, 0.0, 0.0],
        })
    }

    fn draw_primitive(
        &mut self,
        primitive: &Primitive,
        pipeline: &mut AkselPipeline,
        queue: &wgpu::Queue,
        font_system: &mut cosmic_text::FontSystem,
        scale_factor: f32,
    ) {
        match primitive {
            Primitive::Rectangle {
                xy1,
                xy2,
                fill,
                stroke: _stroke,
            } => {
                let color = fill.map(pack_color).unwrap_or(TRANSPARENT);

                // Use SDF rounded rect rendering
                let center_x = (xy1.x + xy2.x) / 2.0;
                let center_y = (xy1.y + xy2.y) / 2.0;
                let half_width = (xy2.x - xy1.x).abs() / 2.0;
                let half_height = (xy2.y - xy1.y).abs() / 2.0;
                let corner_radius = 0.0; // No rounding for basic rectangles

                // Add small margin for antialiasing
                let margin = 2.0;
                let x1 = xy1.x - margin;
                let y1 = xy1.y - margin;
                let x2 = xy2.x + margin;
                let y2 = xy2.y + margin;

                // Create 6 vertices for the bounding quad (2 triangles)
                // T1
                self.0.push(data::UnifiedVertex {
                    position: [x1, y1],
                    color,
                    uv: [0.0, 0.0],
                    primitive_type: data::PRIM_TYPE_SDF_ROUNDED_RECT,
                    param0: [center_x, center_y, half_width, half_height],
                    param1: [corner_radius, 1.0, 0.0, 0.0], // cos=1, sin=0 (no rotation)
                });
                self.0.push(data::UnifiedVertex {
                    position: [x2, y1],
                    color,
                    uv: [0.0, 0.0],
                    primitive_type: data::PRIM_TYPE_SDF_ROUNDED_RECT,
                    param0: [center_x, center_y, half_width, half_height],
                    param1: [corner_radius, 1.0, 0.0, 0.0],
                });
                self.0.push(data::UnifiedVertex {
                    position: [x1, y2],
                    color,
                    uv: [0.0, 0.0],
                    primitive_type: data::PRIM_TYPE_SDF_ROUNDED_RECT,
                    param0: [center_x, center_y, half_width, half_height],
                    param1: [corner_radius, 1.0, 0.0, 0.0],
                });
                // T2
                self.0.push(data::UnifiedVertex {
                    position: [x2, y1],
                    color,
                    uv: [0.0, 0.0],
                    primitive_type: data::PRIM_TYPE_SDF_ROUNDED_RECT,
                    param0: [center_x, center_y, half_width, half_height],
                    param1: [corner_radius, 1.0, 0.0, 0.0],
                });
                self.0.push(data::UnifiedVertex {
                    position: [x2, y2],
                    color,
                    uv: [0.0, 0.0],
                    primitive_type: data::PRIM_TYPE_SDF_ROUNDED_RECT,
                    param0: [center_x, center_y, half_width, half_height],
                    param1: [corner_radius, 1.0, 0.0, 0.0],
                });
                self.0.push(data::UnifiedVertex {
                    position: [x1, y2],
                    color,
                    uv: [0.0, 0.0],
                    primitive_type: data::PRIM_TYPE_SDF_ROUNDED_RECT,
                    param0: [center_x, center_y, half_width, half_height],
                    param1: [corner_radius, 1.0, 0.0, 0.0],
                });
            }

            Primitive::Line {
                start,
                end,
                stroke,
                clip_bounds: _clip_bounds,
                extensions: _extensions,
                arrows: _arrows,
            } => {
                let color = pack_color(stroke.fill);
                let thickness = stroke.thickness;

                // Calculate bounding box for the line with margin for antialiasing
                let margin = thickness / 2.0 + 2.0;
                let min_x = start.x.min(end.x) - margin;
                let max_x = start.x.max(end.x) + margin;
                let min_y = start.y.min(end.y) - margin;
                let max_y = start.y.max(end.y) + margin;

                // Calculate rotation (0 for now, line is defined by start/end directly)
                let rotation = 0.0_f32;

                // Create 6 vertices for the bounding quad (2 triangles)
                // T1
                self.0.push(data::UnifiedVertex {
                    position: [min_x, min_y],
                    color,
                    uv: [0.0, 0.0],
                    primitive_type: data::PRIM_TYPE_SDF_LINE,
                    param0: [start.x, start.y, end.x, end.y],
                    param1: [thickness, rotation.cos(), rotation.sin(), 0.0],
                });
                self.0.push(data::UnifiedVertex {
                    position: [max_x, min_y],
                    color,
                    uv: [0.0, 0.0],
                    primitive_type: data::PRIM_TYPE_SDF_LINE,
                    param0: [start.x, start.y, end.x, end.y],
                    param1: [thickness, rotation.cos(), rotation.sin(), 0.0],
                });
                self.0.push(data::UnifiedVertex {
                    position: [min_x, max_y],
                    color,
                    uv: [0.0, 0.0],
                    primitive_type: data::PRIM_TYPE_SDF_LINE,
                    param0: [start.x, start.y, end.x, end.y],
                    param1: [thickness, rotation.cos(), rotation.sin(), 0.0],
                });
                // T2
                self.0.push(data::UnifiedVertex {
                    position: [max_x, min_y],
                    color,
                    uv: [0.0, 0.0],
                    primitive_type: data::PRIM_TYPE_SDF_LINE,
                    param0: [start.x, start.y, end.x, end.y],
                    param1: [thickness, rotation.cos(), rotation.sin(), 0.0],
                });
                self.0.push(data::UnifiedVertex {
                    position: [max_x, max_y],
                    color,
                    uv: [0.0, 0.0],
                    primitive_type: data::PRIM_TYPE_SDF_LINE,
                    param0: [start.x, start.y, end.x, end.y],
                    param1: [thickness, rotation.cos(), rotation.sin(), 0.0],
                });
                self.0.push(data::UnifiedVertex {
                    position: [min_x, max_y],
                    color,
                    uv: [0.0, 0.0],
                    primitive_type: data::PRIM_TYPE_SDF_LINE,
                    param0: [start.x, start.y, end.x, end.y],
                    param1: [thickness, rotation.cos(), rotation.sin(), 0.0],
                });
            }
            Primitive::Text {
                font,
                content,
                position,
                size,
                rotation,
                horizontal_alignment,
                vertical_alignment,
                fill,
                quality,
                line_height,
                bounds,
                wrapping,
            } => {
                let color = pack_color(*fill);
                pipeline.text_buffer.set_metrics_and_size(
                    font_system,
                    cosmic_text::Metrics::new(
                        (*size).into(),
                        line_height.to_absolute(*size).into(),
                    ),
                    Some(bounds.width),
                    Some(bounds.height),
                );
                pipeline
                    .text_buffer
                    .set_wrap(font_system, text::to_wrap(*wrapping));
                pipeline.text_buffer.shape_until_scroll(font_system, false);
                pipeline.text_buffer.set_text(
                    font_system,
                    content,
                    &text::to_attributes(*font),
                    text::to_shaping(Shaping::Auto, content),
                    None, // TODO: ?
                );

                for run in pipeline.text_buffer.layout_runs() {
                    for glyph in run.glyphs {
                        // TODO: Offset and/or scale properly?
                        let physical_glyph = glyph.physical((0.0, 0.0), 1.0);
                        let cache_key = physical_glyph.cache_key;

                        if let Some(mut atlas_glyph) =
                            pipeline.atlas.get_glyph(queue, font_system, cache_key)
                        {
                            let logical_position = atlas_glyph.get_logical_position(
                                scale_factor,
                                position,
                                &physical_glyph,
                                &run,
                            );

                            // T1
                            self.push_vertex(
                                [logical_position.x, logical_position.y],
                                color,
                                [atlas_glyph.uv_tl[0], atlas_glyph.uv_tl[1]],
                            );
                            self.push_vertex(
                                [
                                    logical_position.x + logical_position.width,
                                    logical_position.y,
                                ],
                                color,
                                [atlas_glyph.uv_br[0], atlas_glyph.uv_tl[1]],
                            );
                            self.push_vertex(
                                [
                                    logical_position.x,
                                    logical_position.y + logical_position.height,
                                ],
                                color,
                                [atlas_glyph.uv_tl[0], atlas_glyph.uv_br[1]],
                            );
                            // T2
                            self.push_vertex(
                                [
                                    logical_position.x + logical_position.width,
                                    logical_position.y,
                                ],
                                color,
                                [atlas_glyph.uv_br[0], atlas_glyph.uv_tl[1]],
                            );
                            self.push_vertex(
                                [
                                    logical_position.x + logical_position.width,
                                    logical_position.y + logical_position.height,
                                ],
                                color,
                                [atlas_glyph.uv_br[0], atlas_glyph.uv_br[1]],
                            );
                            self.push_vertex(
                                [
                                    logical_position.x,
                                    logical_position.y + logical_position.height,
                                ],
                                color,
                                [atlas_glyph.uv_tl[0], atlas_glyph.uv_br[1]],
                            );
                        }
                    }
                }
            }

            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShaderCache {
    primitives: Arc<[Primitive]>,
    version: u64,
}

impl ShaderCache {
    pub fn new() -> Self {
        Self {
            primitives: Arc::new([]),
            version: MESH_VERSION.fetch_add(1, Ordering::Relaxed),
        }
    }

    pub fn update(&mut self, data: Arc<[Primitive]>) {
        self.version = self.version.wrapping_add(1);
        self.primitives = data;
    }
}

impl iced_wgpu::Primitive for ShaderCache {
    type Pipeline = AkselPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &iced_core::Rectangle,
        viewport: &iced_graphics::Viewport,
    ) {
        let cached_version = pipeline.cache_version.load(Ordering::Relaxed);
        let needs_prepare = self.version != cached_version;

        // Only tessellate and upload if version changed
        if needs_prepare {
            let mut vertices = VertexBuffer::default();

            let mut lock = font_system().write().expect("Failed to get font_system");
            let font_system = lock.raw();
            self.primitives.iter().for_each(|primitive| {
                vertices.draw_primitive(
                    primitive,
                    pipeline,
                    queue,
                    font_system,
                    viewport.scale_factor(),
                )
            });
            drop(lock);

            let needed_capacity = vertices.len();
            if needed_capacity > pipeline.vertex_capacity {
                pipeline.vertex_buffer =
                    data::create_renderer_vertex_buffer(device, needed_capacity);
                pipeline.vertex_capacity = needed_capacity;
            }

            queue.write_buffer(&pipeline.vertex_buffer, 0, vertices.as_bytes());

            // Update uniform buffer with screen dimensions
            let uniforms = data::Uniforms {
                screen_width: viewport.physical_width() as f32,
                screen_height: viewport.physical_height() as f32,
                _padding1: 0.0,
                _padding2: 0.0,
            };
            queue.write_buffer(&pipeline.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

            pipeline.vertex_count = needed_capacity as u32;
        }

        // Always check texture sizes (handles window resize)
        let width = viewport.physical_width();
        let height = viewport.physical_height();

        let size_mismatch = pipeline.cache_size != (width, height);

        if size_mismatch || pipeline.msaa_texture.is_none() {
            let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Aksel MSAA Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: pipeline.sample_count,
                dimension: wgpu::TextureDimension::D2,
                format: pipeline.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            pipeline.msaa_view =
                Some(msaa_texture.create_view(&wgpu::TextureViewDescriptor::default()));
            pipeline.msaa_texture = Some(msaa_texture);
        }

        if size_mismatch || pipeline.cache_texture.is_none() {
            let cache_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Aksel Cache Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: pipeline.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            let cache_view = cache_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let cache_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Aksel Cache Bind Group"),
                layout: &pipeline.blit_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&cache_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&pipeline.sampler),
                    },
                ],
            });

            pipeline.cache_texture = Some(cache_texture);
            pipeline.cache_view = Some(cache_view);
            pipeline.cache_bind_group = Some(cache_bind_group);
            pipeline.cache_size = (width, height);
            pipeline.cache_version.store(0, Ordering::Relaxed);

            // Clear the internal vertices buffer
        }
    }

    fn render(
        &self,
        pipeline: &Self::Pipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        _clip_bounds: &iced_core::Rectangle<u32>,
    ) {
        if pipeline.vertex_count == 0 {
            return;
        }

        // Safety: Always initialized in `prepare` method
        let cache_view = pipeline.cache_view.as_ref().unwrap();
        let cache_bind_group = pipeline.cache_bind_group.as_ref().unwrap();

        let cached_version = pipeline.cache_version.load(Ordering::Relaxed);
        let needs_render = self.version != cached_version;

        if needs_render {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(LABEL_CACHE_RENDER),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: pipeline.msaa_view.as_ref().unwrap_or(cache_view),
                    resolve_target: pipeline.msaa_view.as_ref().map(|_| cache_view),
                    ops: wgpu::Operations {
                        // Clear and store the new texture
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&pipeline.pipeline);
            render_pass.set_bind_group(0, &pipeline.bind_group, &[]);
            render_pass.set_vertex_buffer(0, pipeline.vertex_buffer.slice(..));
            render_pass.draw(0..pipeline.vertex_count, 0..1);

            pipeline
                .cache_version
                .store(self.version, Ordering::Relaxed);
        }

        // Blit cached texture
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Askel Cache Blit"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load, // Preserve the pre-rendered content
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        render_pass.set_pipeline(&pipeline.blit_pipeline);
        render_pass.set_bind_group(0, cache_bind_group, &[]);
        render_pass.draw(0..3, 0..1); // Draw full-screen triangle (3 vertices)
    }

    fn draw(&self, pipeline: &Self::Pipeline, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        render_pass.set_pipeline(&pipeline.pipeline);
        render_pass.set_bind_group(0, &pipeline.bind_group, &[]);
        render_pass.set_vertex_buffer(0, pipeline.vertex_buffer.slice(..));
        render_pass.draw(0..pipeline.vertex_count, 0..1);
        // false
        true
    }
}
