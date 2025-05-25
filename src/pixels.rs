use bevy::{
    asset::RenderAssetUsages,
    image::{ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
};
use bevy_egui::{EguiUserTextures, egui};
use rand_core::RngCore;

use crate::Rng;

pub struct Rgba8 {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Rgba8 {
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub fn new_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha: std::u8::MAX,
        }
    }

    pub fn write(&self, image: &mut Image, x: u32, y: u32) {
        let i = ((y * image.width() + x) * 4) as usize;
        let Some(ref mut frame) = image.data else {
            error!("failed to get image data!");
            return;
        };
        frame[i] = self.red;
        frame[i + 1] = self.green;
        frame[i + 2] = self.blue;
        frame[i + 3] = self.alpha;
    }
}

#[derive(Resource)]
pub struct PixelBufferImageId(pub egui::TextureId);

#[derive(Resource)]
pub struct PixelBuffer {
    pub handle: Handle<Image>,
}

pub fn setup_pixel_buffer(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut user_textures: ResMut<EguiUserTextures>,
) {
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
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::nearest());

    let handle = images.add(image);
    let pixels = PixelBuffer { handle };

    commands.insert_resource(PixelBufferImageId(
        user_textures.add_image(pixels.handle.clone()),
    ));

    commands.insert_resource(pixels);
}

pub fn draw_random_pixels(
    pixels: Res<PixelBuffer>,
    mut images: ResMut<Assets<Image>>,
    mut rng: Rng,
) {
    let Some(image) = images.get_mut(&pixels.handle) else {
        error!("failed to get pixel buffer!");
        return;
    };

    let UVec2 {
        x: width,
        y: height,
    } = image.size();

    for x in 0..width {
        for y in 0..height {
            Rgba8::new_rgb(
                (rng.next_u32() % 255) as u8,
                (rng.next_u32() % 255) as u8,
                (rng.next_u32() % 255) as u8,
            )
            .write(image, x, y);
        }
    }
}
