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

// MSDF generation parameters
const MSDF_SPREAD: f32 = 4.0; // Distance field spread in pixels
const USE_MSDF: bool = true;  // Enable MSDF rendering
const MSDF_REFERENCE_SIZE: f32 = 48.0; // Fixed size to render glyphs at for MSDF generation

/// Custom cache key that excludes font size for resolution-independent SDF caching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SdfCacheKey {
    font_id: cosmic_text::fontdb::ID,
    glyph_id: u16,
}

#[derive(Clone, Copy)]
pub struct AtlasGlyph {
    placement: Placement,
    pub uv_tl: [f32; 2],
    pub uv_br: [f32; 2],
    reference_size: f32, // The size this glyph was rendered at for SDF
}

impl AtlasGlyph {
    pub fn get_logical_position(
        &mut self,
        scale_factor: f32,
        physical_position: &iced_core::Point,
        physical_glyph: &cosmic_text::PhysicalGlyph,
        run: &cosmic_text::LayoutRun,
    ) -> iced_core::Rectangle {
        // Get the requested font size from the physical glyph's cache key
        let requested_size = f32::from_bits(physical_glyph.cache_key.font_size_bits);

        // Calculate the scale ratio between requested size and reference size
        let size_scale = requested_size / self.reference_size;

        // Scale the placement based on the size ratio
        let width = (self.placement.width as f32 * size_scale) / scale_factor;
        let height = (self.placement.height as f32 * size_scale) / scale_factor;
        let left = (self.placement.left as f32 * size_scale) / scale_factor;
        let top = (self.placement.top as f32 * size_scale) / scale_factor;
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
    cache: HashMap<SdfCacheKey, AtlasGlyph>,
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
        // Create our size-independent cache key
        let sdf_key = SdfCacheKey {
            font_id: key.font_id,
            glyph_id: key.glyph_id,
        };

        // Check for cached glyph first
        if let Some(cached) = self.cache.get(&sdf_key) {
            return Some(*cached);
        }

        // Try to generate MSDF from glyph shape first
        let (rgba_pixels, width, height, placement) = if USE_MSDF {
            match generate_msdf_from_shape(
                font_system,
                key.font_id,
                key.glyph_id,
                MSDF_REFERENCE_SIZE as u32,
            ) {
                Some((msdf_data, w, h, bbox)) => {
                    // Calculate placement with proper bearing
                    let placement = Placement {
                        left: bbox.x_min as i32,
                        top: bbox.y_max as i32,
                        width: w,
                        height: h,
                    };

                    (msdf_data, w, h, placement)
                }
                None => {
                    // Fallback to raster-based SDF
                    let reference_key = CacheKey {
                        font_id: key.font_id,
                        glyph_id: key.glyph_id,
                        font_size_bits: MSDF_REFERENCE_SIZE.to_bits(),
                        x_bin: key.x_bin,
                        y_bin: key.y_bin,
                        flags: key.flags,
                        font_weight: key.font_weight,
                    };

                    let image = self.swash_cache.get_image_uncached(font_system, reference_key)?;
                    let width = image.placement.width;
                    let height = image.placement.height;

                    if width == 0 || height == 0 {
                        return None;
                    }

                    let pixels = match image.content {
                        SwashContent::Mask => generate_sdf_from_raster(&image.data, width, height),
                        SwashContent::Color => image.data.to_vec(),
                        SwashContent::SubpixelMask => return None,
                    };

                    (pixels, width, height, image.placement)
                }
            }
        } else {
            // MSDF disabled, use raster fallback
            let reference_key = CacheKey {
                font_id: key.font_id,
                glyph_id: key.glyph_id,
                font_size_bits: MSDF_REFERENCE_SIZE.to_bits(),
                x_bin: key.x_bin,
                y_bin: key.y_bin,
                flags: key.flags,
                font_weight: key.font_weight,
            };

            let image = self.swash_cache.get_image_uncached(font_system, reference_key)?;
            let w = image.placement.width;
            let h = image.placement.height;

            if w == 0 || h == 0 {
                return None;
            }

            let pixels = match image.content {
                SwashContent::Mask => generate_sdf_from_raster(&image.data, w, h),
                SwashContent::Color => image.data.to_vec(),
                SwashContent::SubpixelMask => return None,
            };

            (pixels, w, h, image.placement)
        };

        let allocation = self
            .allocator
            .allocate(etagere::size2(width as i32, height as i32))?;

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
            placement,
            uv_tl: [u_min, v_min],
            uv_br: [u_max, v_max],
            reference_size: MSDF_REFERENCE_SIZE,
        };

        self.cache.insert(sdf_key, result);

        Some(result)
    }

    pub fn get_white_pixel_uv() -> [f32; 2] {
        *WHITE_PIXEL_UV.get_or_init(|| {
            let x = 0.5 / ATLAS_SIZE as f32;
            [x, x]
        })
    }
}

/// Generate MSDF from glyph shape using fdsm
/// Returns (rgba_data, width, height, scaled_bbox)
fn generate_msdf_from_shape(
    font_system: &mut FontSystem,
    font_id: cosmic_text::fontdb::ID,
    glyph_id: u16,
    size: u32,
) -> Option<(Vec<u8>, u32, u32, fdsm_ttf_parser::ttf_parser::Rect)> {
    use fdsm_ttf_parser::ttf_parser::{Face, GlyphId, Rect};

    font_system.db().with_face_data(font_id, |data, face_index| {
        // Parse the font with ttf-parser
        let face = Face::parse(data, face_index).ok()?;

        // Load the glyph shape using fdsm-ttf-parser
        let shape = fdsm_ttf_parser::load_shape_from_face(&face, GlyphId(glyph_id))?;

        // Get glyph bounding box to calculate dimensions
        let bbox = face.glyph_bounding_box(GlyphId(glyph_id))?;
        let units_per_em = face.units_per_em() as f32;

        // Calculate scale to fit glyph in the target size
        let scale = size as f32 / units_per_em;

        // Calculate actual pixel dimensions with padding for the distance field
        let padding = MSDF_SPREAD;
        let width = (((bbox.x_max - bbox.x_min) as f32 * scale).ceil() as i32 + (padding * 2.0) as i32).max(1) as u32;
        let height = (((bbox.y_max - bbox.y_min) as f32 * scale).ceil() as i32 + (padding * 2.0) as i32).max(1) as u32;

        // Transform shape to pixel coordinates
        // Don't flip Y in the transform (that inverts winding order)
        // We'll flip the image buffer afterwards instead
        use fdsm::transform::Transform;
        use nalgebra::{Affine2, Matrix3};

        let scale_f64 = scale as f64;

        let transform = Affine2::from_matrix_unchecked(Matrix3::new(
            scale_f64, 0.0, (-bbox.x_min as f64 * scale_f64) + padding as f64,
            0.0, scale_f64, (-bbox.y_min as f64 * scale_f64) + padding as f64,
            0.0, 0.0, 1.0,
        ));

        let mut transformed_shape = shape;
        transformed_shape.transform(&transform);

        // Color the transformed shape using edge coloring
        let sin_alpha = 3.0f64.to_radians().sin(); // Angle threshold for corners
        let seed = 0; // Random seed for coloring
        let colored_shape = fdsm::shape::Shape::<fdsm::shape::ColoredContour>::edge_coloring_simple(
            transformed_shape,
            sin_alpha,
            seed
        );

        // Prepare the shape for MSDF generation
        let prepared = colored_shape.prepare();

        // Create scaled bounding box for placement
        let scaled_bbox = Rect {
            x_min: (bbox.x_min as f32 * scale) as i16,
            y_min: (bbox.y_min as f32 * scale) as i16,
            x_max: (bbox.x_max as f32 * scale) as i16,
            y_max: (bbox.y_max as f32 * scale) as i16,
        };

        // Create an RGB image buffer for the MSDF
        let mut image = image::RgbImage::new(width, height);

        // Generate the MSDF
        let range = MSDF_SPREAD as f64; // Distance range in pixels
        fdsm::generate::generate_msdf(&prepared, range, &mut image);

        // Flip the image vertically (fonts use Y-up, images use Y-down)
        // This is simpler than flipping during transform which inverts winding order
        image::imageops::flip_vertical_in_place(&mut image);

        // Convert to RGBA format for the texture atlas
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for pixel in image.pixels() {
            rgba.push(pixel[0]); // R
            rgba.push(pixel[1]); // G
            rgba.push(pixel[2]); // B
            rgba.push(255);      // A (full alpha)
        }

        Some((rgba, width, height, scaled_bbox))
    })?
}

/// Generate a high-quality SDF from a rasterized glyph using distance transform (fallback)
/// This uses a two-pass algorithm for efficiency and accuracy
fn generate_sdf_from_raster(data: &[u8], width: u32, height: u32) -> Vec<u8> {
    if !USE_MSDF {
        // Just convert alpha to RGB
        let mut result = Vec::with_capacity((width * height * 4) as usize);
        for alpha in data {
            result.extend_from_slice(&[255, 255, 255, *alpha]);
        }
        return result;
    }

    let w = width as usize;
    let h = height as usize;
    let spread = MSDF_SPREAD;

    // Convert alpha to binary with anti-aliasing consideration
    // Use the alpha values directly for better gradients
    let values: Vec<f32> = data.iter().map(|&a| a as f32 / 255.0).collect();

    // Compute distance transform
    let mut dist_inside = vec![std::f32::INFINITY; w * h];
    let mut dist_outside = vec![std::f32::INFINITY; w * h];

    // Initialize distances
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let alpha = values[idx];

            if alpha > 0.5 {
                // Inside
                dist_inside[idx] = 0.0;
                // Calculate distance to nearest "outside" pixel
                let edge_dist = (alpha - 0.5) * 2.0; // 0.0 at edge, 1.0 at full opacity
                dist_outside[idx] = edge_dist * spread;
            } else {
                // Outside
                dist_outside[idx] = 0.0;
                // Calculate distance to nearest "inside" pixel
                let edge_dist = (0.5 - alpha) * 2.0; // 0.0 at edge, 1.0 at full transparency
                dist_inside[idx] = edge_dist * spread;
            }
        }
    }

    // Two-pass distance transform (horizontal then vertical)
    // Pass 1: Forward pass (top-left to bottom-right)
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;

            // Check left neighbor
            if x > 0 {
                let left_idx = idx - 1;
                dist_inside[idx] = dist_inside[idx].min(dist_inside[left_idx] + 1.0);
                dist_outside[idx] = dist_outside[idx].min(dist_outside[left_idx] + 1.0);
            }

            // Check top neighbor
            if y > 0 {
                let top_idx = idx - w;
                dist_inside[idx] = dist_inside[idx].min(dist_inside[top_idx] + 1.0);
                dist_outside[idx] = dist_outside[idx].min(dist_outside[top_idx] + 1.0);
            }
        }
    }

    // Pass 2: Backward pass (bottom-right to top-left)
    for y in (0..h).rev() {
        for x in (0..w).rev() {
            let idx = y * w + x;

            // Check right neighbor
            if x < w - 1 {
                let right_idx = idx + 1;
                dist_inside[idx] = dist_inside[idx].min(dist_inside[right_idx] + 1.0);
                dist_outside[idx] = dist_outside[idx].min(dist_outside[right_idx] + 1.0);
            }

            // Check bottom neighbor
            if y < h - 1 {
                let bottom_idx = idx + w;
                dist_inside[idx] = dist_inside[idx].min(dist_inside[bottom_idx] + 1.0);
                dist_outside[idx] = dist_outside[idx].min(dist_outside[bottom_idx] + 1.0);
            }
        }
    }

    // Generate final SDF
    let mut sdf = vec![0u8; w * h * 4];

    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;

            // Compute signed distance (positive inside, negative outside)
            // Fixed: was dist_inside - dist_outside, should be dist_outside - dist_inside
            let signed_dist = dist_outside[idx] - dist_inside[idx];

            // Normalize to 0-1 range
            let normalized = ((signed_dist / spread) + 1.0) * 0.5;
            let value = (normalized.clamp(0.0, 1.0) * 255.0) as u8;

            // Store in all RGB channels (pseudo-MSDF - same value in all channels)
            // The median filter in the shader will still work correctly
            let out_idx = idx * 4;
            sdf[out_idx] = value;
            sdf[out_idx + 1] = value;
            sdf[out_idx + 2] = value;
            sdf[out_idx + 3] = 255; // Full alpha
        }
    }

    sdf
}
