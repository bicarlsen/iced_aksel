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

fn pack_color(color: Color) -> PackedColor {
    color.into_linear()
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
                stroke,
            } => {
                let color = fill.map(pack_color).unwrap_or(TRANSPARENT);
                let uv = TextureAtlas::get_white_pixel_uv();

                // T1
                self.push_vertex([xy1.x, xy1.y], color, uv);
                self.push_vertex([xy2.x, xy1.y], color, uv);
                self.push_vertex([xy1.x, xy2.y], color, uv);
                // T2
                self.push_vertex([xy2.x, xy1.y], color, uv);
                self.push_vertex([xy2.x, xy2.y], color, uv);
                self.push_vertex([xy1.x, xy2.y], color, uv);
            }

            Primitive::Line {
                start,
                end,
                stroke,
                clip_bounds,
                extensions,
                arrows,
            } => {
                let color = pack_color(stroke.fill);
                let uv = TextureAtlas::get_white_pixel_uv();

                // Calculate direction vector
                let dx = end.x - start.x;
                let dy = end.y - start.y;

                // Calculate length
                let len = dx.hypot(dy);

                if len == 0.0 {
                    return; // No 0-division!
                }

                // Calculate normalized direction
                let u_x = dx / len;
                let u_y = dy / len;

                // Calculate perpendicular vector
                let n_x = -u_y;
                let n_y = u_x;

                // Calculate offset
                let half_width = stroke.thickness / 2.0;
                let off_x = n_x * half_width;
                let off_y = n_y * half_width;

                // Calculate the 4 corners
                let x1 = start.x + off_x;
                let y1 = start.y + off_y;
                let x2 = start.x - off_x;
                let y2 = start.y - off_y;
                let x3 = end.x - off_x;
                let y3 = end.y - off_y;
                let x4 = end.x + off_x;
                let y4 = end.y + off_y;

                // T1
                self.push_vertex([x1, y1], color, uv);
                self.push_vertex([x2, y2], color, uv);
                self.push_vertex([x4, y4], color, uv);
                // T2
                self.push_vertex([x2, y2], color, uv);
                self.push_vertex([x3, y3], color, uv);
                self.push_vertex([x4, y4], color, uv);
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
            pipeline.vertex_buffer = data::create_renderer_vertex_buffer(device, needed_capacity);
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
        // render_pass.set_pipeline(&pipeline.pipeline);
        // render_pass.set_bind_group(0, &pipeline.bind_group, &[]);
        // render_pass.set_vertex_buffer(0, pipeline.vertex_buffer.slice(..));
        // render_pass.draw(0..pipeline.vertex_count, 0..1);
        false
    }
}
