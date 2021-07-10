use glutin::dpi::{PhysicalSize, Size};
use glutin::event_loop::EventLoop;
use glutin::window::WindowBuilder;
use glutin::{Api, ContextBuilder, GlProfile, GlRequest, PossiblyCurrent, WindowedContext};

pub struct Display {
    context: WindowedContext<PossiblyCurrent>,
}

impl Display {
    #[allow(unused_must_use)]
    pub fn new(title: &'static str, event_loop: &EventLoop<()>) -> Self {
        let monitor = event_loop.primary_monitor().unwrap();
        let psize = monitor.size();
        let size = Size::Physical(psize);

        let window_builder = WindowBuilder::new()
            .with_title(title)
            .with_maximized(false)
            .with_resizable(true)
            .with_inner_size(size);
        let context_builder = ContextBuilder::new()
            .with_srgb(true)
            .with_vsync(true)
            .with_depth_buffer(24)
            .with_gl_profile(GlProfile::Core)
            .with_gl(GlRequest::Specific(Api::OpenGl, (4, 1)));

        let context = context_builder
            .build_windowed(window_builder, &event_loop)
            .unwrap();
        let context = unsafe { context.make_current().unwrap() };

        context.window().set_cursor_grab(true);
        context.window().set_cursor_visible(false);

        gl::load_with(|symbol| context.get_proc_address(symbol));

        Display { context }
    }

    pub fn size(&self) -> (usize, usize) {
        let size = self.context.window().inner_size();
        (size.width as usize, size.height as usize)
    }

    pub fn resize(&self, size: PhysicalSize<u32>) {
        self.context.resize(size);
    }

    pub fn swap_buffers(&self) {
        self.context.swap_buffers().unwrap();
    }

    pub fn request_redraw(&self) {
        self.context.window().request_redraw();
    }

    //pub fn set_fullscreen(&self, value: bool) {
    //    self.context.window().set_fullscreen(if value {
    //        Some(Fullscreen::Borderless(self.monitor.clone()))
    //    } else {
    //        None
    //    });
    //}
}
