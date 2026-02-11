use std::collections::HashMap;

use etagere::AtlasAllocator;
use iced_graphics::text::cosmic_text::{CacheKey, FontSystem, Placement, SwashCache, SwashContent};
use iced_wgpu::wgpu;

#[derive(Clone, Copy)]
pub struct AtlasGlyph {
    placement: Placement,
    uv_tl: [f32; 2],
    uv_br: [f32; 2],
}

pub struct FontAtlas {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub width: u32,
    pub height: u32,

    pub(crate) allocator: AtlasAllocator,

    cache: HashMap<CacheKey, AtlasGlyph>,
}

impl FontAtlas {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Aksel font altas"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let allocator = AtlasAllocator::new(etagere::size2(width as i32, height as i32));

        Self {
            texture,
            view,
            width,
            height,
            allocator,
            cache: HashMap::new(),
        }
    }

    pub fn get_glyph(
        &mut self,
        queue: &wgpu::Queue,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
        key: CacheKey,
    ) -> Option<AtlasGlyph> {
        // Check for cached glyph first
        if let Some(cached) = self.cache.get(&key) {
            return Some(*cached);
        }

        // Render glyph pixels with cosmic-text
        let image = swash_cache.get_image_uncached(font_system, key)?;
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
        let u_min = p_min.x as f32 / self.width as f32;
        let v_min = p_min.y as f32 / self.height as f32;
        let u_max = (p_min.x as f32 + width as f32) / self.width as f32;
        let v_max = (p_min.y as f32 + height as f32) / self.height as f32;

        let result = AtlasGlyph {
            placement: image.placement,
            uv_tl: [u_min, v_min],
            uv_br: [u_max, v_max],
        };

        self.cache.insert(key, result);

        Some(result)
    }
}
