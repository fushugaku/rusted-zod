use bevy::{
    asset::{AssetId, Assets, Handle},
    image::Image,
    prelude::{Color, Component, Resource, Sprite},
    render::render_resource::TextureFormat,
};
use std::collections::HashMap;

/// CPU-side equivalents of the source `ZSDL_Surface::BlitHitSurface` output.
///
/// A source image has one cached white silhouette with the exact same dimensions,
/// atlas coordinates and sampler. Swapping the sprite image for one extracted
/// frame gives the source behavior without requiring a second unit-specific
/// render path or a platform-specific shader variant.
#[derive(Default, Resource)]
pub(crate) struct SourceHitSurfaceCache {
    images: HashMap<AssetId<Image>, Handle<Image>>,
}

#[derive(Clone, Component)]
pub(crate) struct ObjectHitFlash {
    original_image: Handle<Image>,
    hit_image: Handle<Image>,
    original_color: Color,
}

impl SourceHitSurfaceCache {
    fn image_for(
        &mut self,
        original: &Handle<Image>,
        images: &mut Assets<Image>,
    ) -> Option<Handle<Image>> {
        if let Some(hit_image) = self.images.get(&original.id()) {
            return Some(hit_image.clone());
        }

        let mut hit_image = images.get(original)?.clone();
        let pixels = hit_image.data.as_mut()?;
        if !apply_source_hit_pixels(pixels, hit_image.texture_descriptor.format) {
            return None;
        }

        let hit_image = images.add(hit_image);
        self.images.insert(original.id(), hit_image.clone());
        Some(hit_image)
    }
}

pub(crate) fn apply_source_hit_surface(
    sprite: &mut Sprite,
    active: Option<&ObjectHitFlash>,
    cache: &mut SourceHitSurfaceCache,
    images: &mut Assets<Image>,
) -> Option<ObjectHitFlash> {
    let (original_image, original_color) = active.map_or_else(
        || (sprite.image.clone(), sprite.color),
        |flash| (flash.original_image.clone(), flash.original_color),
    );
    let hit_image = cache.image_for(&original_image, images)?;

    sprite.image = hit_image.clone();
    sprite.color = Color::WHITE;
    Some(ObjectHitFlash {
        original_image,
        hit_image,
        original_color,
    })
}

pub(crate) fn restore_source_hit_surface(sprite: &mut Sprite, flash: &ObjectHitFlash) {
    // A unit animation may have advanced to another standalone image during the
    // hit frame. In that case its new image is already authoritative and must
    // not be replaced by the stale pre-hit frame.
    if sprite.image == flash.hit_image {
        sprite.image = flash.original_image.clone();
    }
    if sprite.color == Color::WHITE {
        sprite.color = flash.original_color;
    }
}

fn apply_source_hit_pixels(pixels: &mut [u8], format: TextureFormat) -> bool {
    match format {
        TextureFormat::Rgba8Unorm
        | TextureFormat::Rgba8UnormSrgb
        | TextureFormat::Rgba8Uint
        | TextureFormat::Bgra8Unorm
        | TextureFormat::Bgra8UnormSrgb => {
            if !pixels.len().is_multiple_of(4) {
                return false;
            }
            for pixel in pixels.chunks_exact_mut(4) {
                if pixel[3] == 0 {
                    pixel.copy_from_slice(&[0, 0, 0, 0]);
                } else {
                    pixel.copy_from_slice(&[255, 255, 255, 255]);
                }
            }
            true
        }
        TextureFormat::Rg8Unorm | TextureFormat::Rg8Uint => {
            if !pixels.len().is_multiple_of(2) {
                return false;
            }
            for pixel in pixels.chunks_exact_mut(2) {
                if pixel[1] == 0 {
                    pixel.copy_from_slice(&[0, 0]);
                } else {
                    pixel.copy_from_slice(&[255, 255]);
                }
            }
            true
        }
        TextureFormat::R8Unorm | TextureFormat::R8Uint => {
            for value in pixels {
                *value = 255;
            }
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgba_hit_surface_uses_alpha_as_a_binary_white_mask() {
        let mut pixels = vec![12, 34, 56, 0, 1, 2, 3, 1, 120, 130, 140, 254];

        assert!(apply_source_hit_pixels(
            &mut pixels,
            TextureFormat::Rgba8UnormSrgb
        ));
        assert_eq!(
            pixels,
            vec![0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255]
        );
    }

    #[test]
    fn luminance_alpha_hit_surface_uses_the_second_channel_as_alpha() {
        let mut pixels = vec![90, 0, 17, 128];

        assert!(apply_source_hit_pixels(
            &mut pixels,
            TextureFormat::Rg8Unorm
        ));
        assert_eq!(pixels, vec![0, 0, 255, 255]);
    }

    #[test]
    fn unsupported_pixel_layout_is_not_silently_reinterpreted() {
        let mut pixels = vec![0; 8];

        assert!(!apply_source_hit_pixels(
            &mut pixels,
            TextureFormat::Rgba16Float
        ));
        assert_eq!(pixels, vec![0; 8]);
    }
}
