use std::time::Instant;

use gfx::Device;
use glutin::{Event, WindowEvent};
use imgui::{Context, FontConfig, FontSource, Ui, TextureId};
use imgui_winit_support::{HiDpiMode, WinitPlatform};

#[cfg_attr(feature = "opengl", path = "gl.rs")]
#[cfg_attr(feature = "directx", path = "dx.rs")]
mod backend;

use backend::*;

type ColorFormat = gfx::format::Srgba8;

pub struct System {
    pub events_loop: glutin::EventsLoop,
    pub imgui: Context,
    pub platform: WinitPlatform,
    pub render_sys: RenderSystem,
    pub font_size: f32,
}

pub fn init(title: &str) -> System {
    let title = match title.rfind('/') {
        Some(idx) => title.split_at(idx + 1).1,
        None => title,
    };
    let events_loop = glutin::EventsLoop::new();
    let builder = glutin::WindowBuilder::new()
        .with_title(title.to_owned())
        .with_dimensions(glutin::dpi::LogicalSize::new(1024f64, 768f64));

    let mut imgui = Context::create();
    imgui.set_ini_filename(None);

    let mut platform = WinitPlatform::init(&mut imgui);

    let hidpi_factor = platform.hidpi_factor();
    let font_size = (13.0 * hidpi_factor) as f32;
    imgui.fonts().add_font(&[
        FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: font_size,
                ..FontConfig::default()
            }),
        },
    ]);

    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    let render_sys = RenderSystem::init(&mut imgui, builder, &events_loop);
    platform.attach_window(imgui.io_mut(), render_sys.window(), HiDpiMode::Rounded);
    System {
        events_loop,
        imgui,
        platform,
        render_sys,
        font_size,
    }
}

impl System {
    pub fn main_loop<F: FnMut(&mut bool, &mut Ui, &mut types::Textures)>(self, mut run_ui: F) {
        let System {
            mut events_loop,
            mut imgui,
            mut platform,
            mut render_sys,
            ..
        } = self;
        let mut encoder: gfx::Encoder<_, _> = render_sys.factory.create_command_buffer().into();

        let mut last_frame = Instant::now();
        let mut run = true;

        while run {
            events_loop.poll_events(|event| {
                platform.handle_event(imgui.io_mut(), render_sys.window(), &event);

                if let Event::WindowEvent { event, .. } = event {
                    match event {
                        WindowEvent::Resized(size) => render_sys.update_views(size),
                        WindowEvent::CloseRequested => run = false,
                        _ => (),
                    }
                }
            });            

            let io = imgui.io_mut();
            platform
                .prepare_frame(io, render_sys.window())
                .expect("Failed to start frame");
            last_frame = io.update_delta_time(last_frame);
            
            let mut textures = render_sys.textures();

            let mut ui = imgui.frame();
            run_ui(&mut run, &mut ui, &mut textures);

            if let Some(main_color) = render_sys.main_color.as_mut() {
                encoder.clear(main_color, [1.0, 1.0, 1.0, 1.0]);
            }
            platform.prepare_render(&ui, render_sys.window());
            let draw_data = ui.render();
            if let Some(main_color) = render_sys.main_color.as_mut() {
                render_sys
                    .renderer
                    .render(&mut render_sys.factory, &mut encoder, main_color, draw_data)
                    .expect("Rendering failed");
            }
            encoder.flush(&mut render_sys.device);
            render_sys.swap_buffers();
            render_sys.device.cleanup();
        }
    }
}

#[derive(Copy, Clone)]
pub struct Texture {
    pub id: TextureId,
    pub size: [f32; 2],
}

pub trait Textures {
    type CreationError: std::fmt::Debug;
    fn create_texture(&mut self, image: &image::DynamicImage) -> Result<Texture, Self::CreationError>;
}