use super::atlas::TextureAtlas;
use super::data::{self, UnifiedVertex};
use super::pipeline::{AkselPipeline, LABEL_VERTEX_BUFFER};
use bytemuck::{Pod, Zeroable};
use iced_core::{Color, text::Shaping};
use iced_graphics::text::{
    self,
    cosmic_text::{Buffer, Metrics},
    font_system,
};
use iced_wgpu::wgpu;

use crate::render::Primitive;

type PackedColor = [f32; 4];

const TRANSPARENT: PackedColor = [0., 0., 0., 0.];
const INIT_CAPACITY: usize = 10_000;

fn pack_color(color: Color) -> PackedColor {
    color.into_linear()
}

#[derive(Debug)]
pub struct AkselMesh(Vec<UnifiedVertex>);

impl AkselMesh {
    pub fn new() -> Self {
        Self(Vec::with_capacity(INIT_CAPACITY))
    }

    pub fn push_vertex(&mut self, position: [f32; 2], color: PackedColor, uv: [f32; 2]) {
        self.0.push(UnifiedVertex {
            position,
            color,
            uv,
        })
    }

    pub fn push_primitive(&mut self, primitive: Primitive) {
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
                let len = (dx * dx + dy * dy).sqrt();

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
                // let mut lock = font_system().write().expect("Failed to get font_system");
                // let font_system = lock.raw();
                // let mut buffer = Buffer::new(
                //     font_system,
                //     Metrics::new(size.into(), line_height.to_absolute(size).into()),
                // );
                // buffer.set_size(font_system, Some(bounds.width), Some(bounds.height));
                // buffer.set_wrap(font_system, text::to_wrap(wrapping));
                // buffer.set_text(
                //     font_system,
                //     &content,
                //     &text::to_attributes(font),
                //     text::to_shaping(Shaping::Auto, &content),
                //     None, // TODO: ?
                // );
                // buffer.shape_until_scroll(font_system, false);
                //
                // for run in buffer.layout_runs() {
                //     for glyph in run.glyphs {
                //         // TODO: Offset and/or scale properly?
                //         let physical_glyph = glyph.physical((0.0, 0.0), 1.0);
                //
                //         let cache_key =
                //     }
                // }
            }

            _ => {}
        }
    }
}

impl iced_wgpu::Primitive for AkselMesh {
    type Pipeline = AkselPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &iced_core::Rectangle,
        viewport: &iced_graphics::Viewport,
    ) {
        let needed_capacity = self.0.len();
        if needed_capacity > pipeline.vertex_capacity {
            pipeline.vertex_buffer = data::create_vertex_buffer(device, needed_capacity);
            pipeline.vertex_capacity = needed_capacity;
        }

        queue.write_buffer(&pipeline.vertex_buffer, 0, bytemuck::cast_slice(&self.0));

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

    fn draw(&self, pipeline: &Self::Pipeline, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        render_pass.set_pipeline(&pipeline.pipeline);
        render_pass.set_bind_group(0, &pipeline.bind_group, &[]);
        render_pass.set_vertex_buffer(0, pipeline.vertex_buffer.slice(..));
        render_pass.draw(0..pipeline.vertex_count, 0..1);
        true // TODO: With caching, this should probably not return true always?
    }
}
