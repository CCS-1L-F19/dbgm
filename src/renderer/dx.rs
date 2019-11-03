// DirectX backend. not finished

mod types {
    pub type Device = gfx_device_dx11::Device;
    pub type Factory = gfx_device_dx11::Factory;
    pub type Resources = gfx_device_dx11::Resources;
}

pub struct RenderSystem {
    pub renderer: Renderer<ColorFormat, types::Resources>,
    pub window: gfx_window_dxgi::Window,
    pub device: types::Device,
    pub factory: types::Factory,
    pub main_color: Option<gfx::handle::RenderTargetView<types::Resources, ColorFormat>>,
}

impl RenderSystem {
    pub fn init(
        imgui: &mut Context,
        builder: glutin::WindowBuilder,
        events_loop: &glutin::EventsLoop,
    ) -> RenderSystem {
        let (window, device, mut factory, main_color) =
            gfx_window_dxgi::init(builder, &events_loop).expect("Failed to initialize graphics");
        let renderer = Renderer::init(imgui, &mut factory, Shaders::HlslSm40)
            .expect("Failed to initialize renderer");
        RenderSystem {
            renderer,
            window,
            device,
            factory,
            main_color: Some(main_color),
        }
    }
    pub fn window(&self) -> &glutin::Window {
        &self.window.inner
    }
    pub fn update_views(&mut self, size: glutin::dpi::LogicalSize) {
        let physical = size.to_physical(self.window().get_hidpi_factor());
        let (width, height): (u32, u32) = physical.into();
        let _ = self.main_color.take(); // we need to drop main_color before calling update_views
        self.main_color = Some(
            gfx_window_dxgi::update_views(
                &mut self.window,
                &mut self.factory,
                &mut self.device,
                width as u16,
                height as u16,
            )
            .expect("Failed to update resize"),
        );
    }
    pub fn swap_buffers(&mut self) {
        self.window.swap_buffers(1);
    }
}