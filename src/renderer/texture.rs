use image::*;
use imgui::TextureId;
use gfx::{format, texture as tex, Factory as _};
use gfx_device_gl::{Resources, Factory};
use crate::gui::Textures;

type ColorFormat = format::Srgba8;

pub struct GfxGlTextures<'a> {
    pub(in super) factory: &'a mut Factory,
    pub(in super) textures: &'a mut imgui::Textures<imgui_gfx_renderer::Texture<Resources>>,
}

impl<'a> Textures for GfxGlTextures<'a> {
    type CreationError = gfx::CombinedError;
    fn create_texture(&mut self, image: &image::DynamicImage) -> Result<TextureId, gfx::CombinedError> {
        let (width, height) = image.dimensions();
        
        let (_, srv) = self.factory.create_texture_immutable_u8::<ColorFormat>(
            tex::Kind::D2(width as u16, height as u16, tex::AaMode::Single),
            tex::Mipmap::Allocated,
            &[&image.to_rgba()],
        )?;

        let sampler = self.factory.create_sampler(
            tex::SamplerInfo::new(tex::FilterMethod::Bilinear, tex::WrapMode::Tile)
        );

        Ok(self.textures.insert((srv, sampler)))
    }

    /*
    pub fn create_texture(&mut self, image: image::DynamicImage) -> Result<TextureId, TextureCreationError> {
        use tex::*;

        let size = (image.width() as u16, image.height() as u16);
        let texture = self.factory.create_texture::<Bgra>(
            Kind::D2(size.0, size.1, AaMode::Single), 0, mem::Bind::TRANSFER_DST, mem::Usage::Upload, None
        )?;

        let srv = self.factory.view_texture_as_shader_resource(&texture, (0, 0), format::Swizzle::new())?;

        let sampler = self.factory.create_sampler(
            SamplerInfo::new(tex::FilterMethod::Bilinear, tex::WrapMode::Tile)
        );

        self.to_upload.push((image, texture));
        Ok(self.textures.insert((srv, sampler)))
    }

    pub(in super) fn upload_all(&mut self, encoder: &mut gfx::Encoder<Resources, CommandBuffer>) -> Result<(), TextureUploadError> {
        let data_size = self.to_upload.iter()
            .map(|(i, _)| (i.width() * i.height()) as usize)
            .fold(0, |size, s| size + s) * 4;

        let staging_buffer = self.factory.create_upload_buffer(data_size)?;
        let mut writer = self.factory.write_mapping(&staging_buffer)?;

        for (image, _) in self.to_upload {
            let data = image.to_bgra().into_raw();
            std::io::copy(&mut (&*data), &mut &mut *writer); // yikes
        }

        let mut offset = 0usize;
        for (image, texture) in self.to_upload {
            offset += (image.width() * image.height()) as usize;
        }

        self.to_upload.clear();
        Ok(())
    }
    */
}