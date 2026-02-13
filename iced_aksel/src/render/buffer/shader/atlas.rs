use std::{collections::HashMap, sync::OnceLock};

use etagere::AtlasAllocator;
use iced_graphics::text::cosmic_text::{
    self, CacheKey, FontSystem, Placement, SwashCache, SwashContent,
};
use iced_wgpu::wgpu;

const LABEL_ATLAS_TEXTURE: &str = "Aksel Font Atlas";
const ATLAS_SIZE: u32 = 1024;
const WHITE_PIXEL: [u8; 4] = [255, 255, 255, 255];
static WHITE_PIXEL_UV: OnceLock<[f32; 2]> = OnceLock::new();

#[derive(Clone, Copy)]
pub struct AtlasGlyph {
    placement: Placement,
    pub uv_tl: [f32; 2],
    pub uv_br: [f32; 2],
}

impl AtlasGlyph {
    pub fn get_logical_position(
        &mut self,
        scale_factor: f32,
        physical_position: &iced_core::Point,
        physical_glyph: &cosmic_text::PhysicalGlyph,
        run: &cosmic_text::LayoutRun,
    ) -> iced_core::Rectangle {
        let width = self.placement.width as f32 / scale_factor;
        let height = self.placement.height as f32 / scale_factor;
        let left = self.placement.left as f32 / scale_factor;
        let top = self.placement.top as f32 / scale_factor;
        let glyph_x = physical_glyph.x as f32 / scale_factor;
        let glyph_y = physical_glyph.y as f32 / scale_factor;
        let line_y = run.line_y / scale_factor;

        iced_core::Rectangle {
            x: physical_position.x + glyph_x + left,
            y: physical_position.y + line_y + glyph_y - top,
            width,
            height,
        }
    }
}

pub struct TextureAtlas {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub(crate) allocator: AtlasAllocator,
    cache: HashMap<CacheKey, AtlasGlyph>,
    swash_cache: SwashCache,
}

impl TextureAtlas {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let size = wgpu::Extent3d {
            width: ATLAS_SIZE,
            height: ATLAS_SIZE,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(LABEL_ATLAS_TEXTURE),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut allocator =
            AtlasAllocator::new(etagere::size2(ATLAS_SIZE as i32, ATLAS_SIZE as i32));

        // Write texture used for drawing shapes (A single white pixel)
        let _ = allocator.allocate(etagere::size2(1, 1)).unwrap();
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &WHITE_PIXEL,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        Self {
            texture,
            view,
            allocator,
            cache: HashMap::new(),
            swash_cache: SwashCache::new(),
        }
    }

    pub fn get_glyph(
        &mut self,
        queue: &wgpu::Queue,
        font_system: &mut FontSystem,
        key: CacheKey,
    ) -> Option<AtlasGlyph> {
        // Check for cached glyph first
        if let Some(cached) = self.cache.get(&key) {
            return Some(*cached);
        }

        // Render glyph pixels with cosmic-text
        let image = self.swash_cache.get_image_uncached(font_system, key)?;
        let width = image.placement.width;
        let height = image.placement.height;

        if width == 0 || height == 0 {
            return None; // Space character or invisible
        }

        let allocation = self
            .allocator
            .allocate(etagere::size2(width as i32, height as i32))?;

        // Upload pixels to GPU
        let mut rgba_pixels = Vec::with_capacity((width * height * 4) as usize);
        match image.content {
            SwashContent::Mask => {
                for alpha in image.data {
                    rgba_pixels.extend_from_slice(&[255, 255, 255, alpha]);
                }
            }
            SwashContent::Color => {
                rgba_pixels.extend_from_slice(&image.data);
            }
            SwashContent::SubpixelMask => return None, // TODO: Handle this?
        }

        let p_min = allocation.rectangle.min;
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: p_min.x as u32,
                    y: p_min.y as u32,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &rgba_pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        // Calculate UVs
        let u_min = p_min.x as f32 / ATLAS_SIZE as f32;
        let v_min = p_min.y as f32 / ATLAS_SIZE as f32;
        let u_max = (p_min.x as f32 + width as f32) / ATLAS_SIZE as f32;
        let v_max = (p_min.y as f32 + height as f32) / ATLAS_SIZE as f32;

        let result = AtlasGlyph {
            placement: image.placement,
            uv_tl: [u_min, v_min],
            uv_br: [u_max, v_max],
        };

        self.cache.insert(key, result);

        Some(result)
    }

    pub fn get_white_pixel_uv() -> [f32; 2] {
        *WHITE_PIXEL_UV.get_or_init(|| {
            let x = 0.5 / ATLAS_SIZE as f32;
            [x, x]
        })
    }
}
