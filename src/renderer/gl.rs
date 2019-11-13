use image::*;
use imgui::Context;
use imgui_gfx_renderer::{Renderer, Shaders};
use gfx::{texture as tex, Factory as _};

use super::{ColorFormat, Textures, Texture};

pub mod types {
    pub type Device = gfx_device_gl::Device;
    pub type Factory = gfx_device_gl::Factory;
    pub type Resources = gfx_device_gl::Resources;
    pub type Textures<'a> = super::GfxGlTextures<'a>;
}

pub struct RenderSystem {
    pub renderer: Renderer<ColorFormat, types::Resources>,
    pub windowed_context: glutin::WindowedContext<glutin::PossiblyCurrent>,
    pub device: types::Device,
    pub factory: types::Factory,
    pub main_color: Option<gfx::handle::RenderTargetView<types::Resources, ColorFormat>>,
    pub main_depth: gfx::handle::DepthStencilView<types::Resources, gfx::format::DepthStencil>,
}

impl RenderSystem {
    pub fn init(
        imgui: &mut Context,
        builder: glutin::WindowBuilder,
        events_loop: &glutin::EventsLoop,
    ) -> RenderSystem {
        {
            // Fix incorrect colors with sRGB framebuffer
            fn imgui_gamma_to_linear(col: [f32; 4]) -> [f32; 4] {
                let x = col[0].powf(2.2);
                let y = col[1].powf(2.2);
                let z = col[2].powf(2.2);
                let w = 1.0 - (1.0 - col[3]).powf(2.2);
                [x, y, z, w]
            }

            let style = imgui.style_mut();
            for col in 0..style.colors.len() {
                style.colors[col] = imgui_gamma_to_linear(style.colors[col]);
            }
            
        }

        let context = glutin::ContextBuilder::new().with_vsync(true);
        let (windowed_context, device, mut factory, main_color, main_depth) =
            gfx_window_glutin::init(builder, context, &events_loop)
                .expect("Failed to initialize graphics");
        let shaders = {
            let version = device.get_info().shading_language;
            if version.is_embedded {
                if version.major >= 3 {
                    Shaders::GlSlEs300
                } else {
                    Shaders::GlSlEs100
                }
            } else if version.major >= 4 {
                Shaders::GlSl400
            } else if version.major >= 3 {
                if version.minor >= 2 {
                    Shaders::GlSl150
                } else {
                    Shaders::GlSl130
                }
            } else {
                Shaders::GlSl110
            }
        };
        let renderer =
            Renderer::init(imgui, &mut factory, shaders).expect("Failed to initialize renderer");
        RenderSystem {
            renderer,
            windowed_context,
            device,
            factory,
            main_color: Some(main_color),
            main_depth,
        }
    }
    pub fn window(&self) -> &glutin::Window {
        self.windowed_context.window()
    }
    pub fn update_views(&mut self, _: glutin::dpi::LogicalSize) {
        if let Some(main_color) = self.main_color.as_mut() {
            gfx_window_glutin::update_views(
                &self.windowed_context,
                main_color,
                &mut self.main_depth,
            );
        }
    }
    pub fn swap_buffers(&mut self) {
        self.windowed_context.swap_buffers().unwrap();
    }
    pub fn textures(&mut self) -> GfxGlTextures {
        GfxGlTextures { 
            factory: &mut self.factory, 
            textures: self.renderer.textures(),
        }
    }
}

pub struct GfxGlTextures<'a> {
    pub(in super) factory: &'a mut types::Factory,
    pub(in super) textures: &'a mut imgui::Textures<imgui_gfx_renderer::Texture<types::Resources>>,
}

impl<'a> Textures for GfxGlTextures<'a> {
    type CreationError = gfx::CombinedError;
    fn create_texture(&mut self, image: &image::DynamicImage) -> Result<Texture, gfx::CombinedError> {
        let (width, height) = image.dimensions();
        
        let (_, srv) = self.factory.create_texture_immutable_u8::<ColorFormat>(
            tex::Kind::D2(width as u16, height as u16, tex::AaMode::Single),
            tex::Mipmap::Allocated,
            &[&image.to_rgba()],
        )?;

        let sampler = self.factory.create_sampler(
            tex::SamplerInfo::new(tex::FilterMethod::Bilinear, tex::WrapMode::Tile)
        );

        let id = self.textures.insert((srv, sampler));
        Ok(Texture { id, size: [width as f32, height as f32] })
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