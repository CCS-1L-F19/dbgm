use imgui::{ImFontConfig, ImGui, ImVec4, Ui};
use imgui_gfx_renderer::{Renderer, Shaders};
use imgui_winit_support;
use std::time::Instant;

use crate::gui::Textures;

mod texture;

type ColorFormat = gfx::format::Rgba8;
type DepthFormat = gfx::format::DepthStencil;

pub struct GfxHost {
    clear_color: [f32; 4],
    events_loop: glutin::EventsLoop,
    window: glutin::WindowedContext,
    device: gfx_device_gl::Device,
    main_color: gfx::handle::RenderTargetView<gfx_device_gl::Resources, ColorFormat>,
    main_depth: gfx::handle::DepthStencilView<gfx_device_gl::Resources, DepthFormat>,
    factory: gfx_device_gl::Factory,
    encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
    imgui: ImGui,
    hidpi_factor: f64,
    renderer: imgui_gfx_renderer::Renderer<gfx_device_gl::Resources>,
}

impl GfxHost {
    pub fn init<F, T>(title: String, clear_color: [f32; 4], init_ui: F) -> (GfxHost, T)
        where F: FnOnce(&mut dyn Textures<CreationError=gfx::CombinedError>) -> T
    {
        let events_loop = glutin::EventsLoop::new();
        let context = glutin::ContextBuilder::new().with_vsync(true);
        let window = glutin::WindowBuilder::new()
            .with_title(title)
            .with_dimensions(glutin::dpi::LogicalSize::new(1024f64, 768f64));
        let (window, device, mut factory, main_color, main_depth) =
            gfx_window_glutin::init::<ColorFormat, DepthFormat>(window, context, &events_loop)
                .expect("Failed to initalize graphics");
        let encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
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

        let mut imgui = ImGui::init();
        {
            // Fix incorrect colors with sRGB framebuffer
            fn imgui_gamma_to_linear(col: ImVec4) -> ImVec4 {
                let x = col.x.powf(2.2);
                let y = col.y.powf(2.2);
                let z = col.z.powf(2.2);
                let w = 1.0 - (1.0 - col.w).powf(2.2);
                ImVec4::new(x, y, z, w)
            }

            let style = imgui.style_mut();
            for col in 0..style.colors.len() {
                style.colors[col] = imgui_gamma_to_linear(style.colors[col]);
            }
        }
        imgui.set_ini_filename(None);

        // In the examples we only use integer DPI factors, because the UI can get very blurry
        // otherwise. This might or might not be what you want in a real application.
        let hidpi_factor = window.get_hidpi_factor().round();

        let font_size = (13.0 * hidpi_factor) as f32;

        imgui.fonts().add_default_font_with_config(
            ImFontConfig::new()
                .oversample_h(1)
                .pixel_snap_h(true)
                .size_pixels(font_size),
        );

        imgui.set_font_global_scale((1.0 / hidpi_factor) as f32);

        let mut renderer = Renderer::init(&mut imgui, &mut factory, shaders, main_color.clone())
            .expect("Failed to initialize renderer");

        imgui_winit_support::configure_keys(&mut imgui);

        let mut textures = texture::GfxGlTextures {
            factory: &mut factory,
            textures: renderer.textures(),
        };

        let result = init_ui(&mut textures);

        (GfxHost {
            clear_color, events_loop, window, device, main_color, main_depth,
            factory, encoder, imgui, hidpi_factor, renderer
        }, result)
    }

    pub fn run<F>(self, mut run_ui: F)
        where F: FnMut(&Ui, &mut dyn Textures<CreationError=gfx::CombinedError>) -> bool,
    {
        use gfx::Device;

        let GfxHost {
            clear_color, mut events_loop, window, mut device, mut main_color, mut main_depth,
            mut factory, mut encoder, mut imgui, hidpi_factor, mut renderer
        } = self;

        let mut last_frame = Instant::now();
        let mut quit = false;

        loop {
            events_loop.poll_events(|event| {
                use glutin::{
                    Event,
                    WindowEvent::{CloseRequested, Resized},
                };

                imgui_winit_support::handle_event(
                    &mut imgui,
                    &event,
                    window.get_hidpi_factor(),
                    hidpi_factor,
                );

                if let Event::WindowEvent { event, .. } = event {
                    match event {
                        Resized(_) => {
                            gfx_window_glutin::update_views(&window, &mut main_color, &mut main_depth);
                            renderer.update_render_target(main_color.clone());
                        }
                        CloseRequested => quit = true,
                        _ => (),
                    }
                }
            });
            if quit {
                break;
            }

            let now = Instant::now();
            let delta = now - last_frame;
            let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
            last_frame = now;

            imgui_winit_support::update_mouse_cursor(&imgui, &window);

            let frame_size = imgui_winit_support::get_frame_size(&window, hidpi_factor).unwrap();

            let ui = imgui.frame(frame_size, delta_s);
            let mut textures = texture::GfxGlTextures {
                factory: &mut factory,
                textures: renderer.textures(),
            };

            if !run_ui(&ui, &mut textures) {
                break;
            }

            encoder.clear(&main_color, clear_color);
            renderer
                .render(ui, &mut factory, &mut encoder)
                .expect("Rendering failed");
            encoder.flush(&mut device);
            window.swap_buffers().unwrap();
            device.cleanup();
        }
    }
}

/*
pub fn run<F>(title: String, clear_color: [f32; 4], mut run_ui: F)
    where F: FnMut(&Ui, &mut dyn Textures<CreationError=gfx::CombinedError>) -> bool,
{
    use gfx::Device;

    type ColorFormat = gfx::format::Rgba8;
    type DepthFormat = gfx::format::DepthStencil;

    let mut events_loop = glutin::EventsLoop::new();
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let window = glutin::WindowBuilder::new()
        .with_title(title)
        .with_dimensions(glutin::dpi::LogicalSize::new(1024f64, 768f64));
    let (window, mut device, mut factory, mut main_color, mut main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(window, context, &events_loop)
            .expect("Failed to initalize graphics");
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
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

    let mut imgui = ImGui::init();
    {
        // Fix incorrect colors with sRGB framebuffer
        fn imgui_gamma_to_linear(col: ImVec4) -> ImVec4 {
            let x = col.x.powf(2.2);
            let y = col.y.powf(2.2);
            let z = col.z.powf(2.2);
            let w = 1.0 - (1.0 - col.w).powf(2.2);
            ImVec4::new(x, y, z, w)
        }

        let style = imgui.style_mut();
        for col in 0..style.colors.len() {
            style.colors[col] = imgui_gamma_to_linear(style.colors[col]);
        }
    }
    imgui.set_ini_filename(None);

    // In the examples we only use integer DPI factors, because the UI can get very blurry
    // otherwise. This might or might not be what you want in a real application.
    let hidpi_factor = window.get_hidpi_factor().round();

    let font_size = (13.0 * hidpi_factor) as f32;

    imgui.fonts().add_default_font_with_config(
        ImFontConfig::new()
            .oversample_h(1)
            .pixel_snap_h(true)
            .size_pixels(font_size),
    );

    imgui.set_font_global_scale((1.0 / hidpi_factor) as f32);

    let mut renderer = Renderer::init(&mut imgui, &mut factory, shaders, main_color.clone())
        .expect("Failed to initialize renderer");

    imgui_winit_support::configure_keys(&mut imgui);

    let mut last_frame = Instant::now();
    let mut quit = false;

    loop {
        events_loop.poll_events(|event| {
            use glutin::{
                Event,
                WindowEvent::{CloseRequested, Resized},
            };

            imgui_winit_support::handle_event(
                &mut imgui,
                &event,
                window.get_hidpi_factor(),
                hidpi_factor,
            );

            if let Event::WindowEvent { event, .. } = event {
                match event {
                    Resized(_) => {
                        gfx_window_glutin::update_views(&window, &mut main_color, &mut main_depth);
                        renderer.update_render_target(main_color.clone());
                    }
                    CloseRequested => quit = true,
                    _ => (),
                }
            }
        });
        if quit {
            break;
        }

        let now = Instant::now();
        let delta = now - last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        last_frame = now;

        imgui_winit_support::update_mouse_cursor(&imgui, &window);

        let frame_size = imgui_winit_support::get_frame_size(&window, hidpi_factor).unwrap();

        let ui = imgui.frame(frame_size, delta_s);
        let mut textures = texture::GfxGlTextures {
            factory: &mut factory,
            textures: renderer.textures(),
        };

        if !run_ui(&ui, &mut textures) {
            break;
        }

        encoder.clear(&main_color, clear_color);
        renderer
            .render(ui, &mut factory, &mut encoder)
            .expect("Rendering failed");
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
*/