use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
};

use crate::Rng;

#[derive(Resource)]
pub struct PixelBuffer {
    pub handle: Handle<Image>,
}

pub fn setup_pixel_buffer(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    const WIDTH: u32 = 28;
    const HEIGHT: u32 = 28;

    let size = Extent3d {
        width: WIDTH,
        height: HEIGHT,
        depth_or_array_layers: 1,
    };

    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    image.texture_descriptor.usage |=
        TextureUsages::COPY_DST | TextureUsages::COPY_SRC | TextureUsages::TEXTURE_BINDING;

    let handle = images.add(image);
    commands.insert_resource(PixelBuffer { handle })
}

pub fn draw_random_pixels(
    pixels: Res<PixelBuffer>,
    mut images: ResMut<Assets<Image>>,
    mut _rng: Rng,
) {
    let Some(image) = images.get_mut(&pixels.handle) else {
        error!("failed to get pixel buffer!");
        return;
    };

    let UVec2 {
        x: width,
        y: height,
    } = image.size();

    let Some(ref mut frame) = image.data else {
        error!("failed to get image data!");
        return;
    };
    for x in 0..width {
        for y in 0..height {
            let i = ((y * width + x) * 4) as usize;
            frame[i] = x as u8; // Red
            frame[i + 1] = y as u8; // Green
            frame[i + 2] = 255; // Blue
            frame[i + 3] = 255; // Alpha
        }
    }
}
