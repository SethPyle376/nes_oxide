use crate::Cpu;

mod debug;
mod frame;

pub use debug::DebugGui;
pub use frame::Frame;

use glow::HasContext;

pub struct Renderer {
    pub window: sdl2::video::Window,
    pub frame: Frame,
    platform: imgui_sdl2_support::SdlPlatform,
    gl: glow::Context,
    gl_context: sdl2::video::GLContext,
    imgui: imgui::Context,
    renderer: imgui_glow_renderer::Renderer,
    textures: imgui::Textures<glow::Texture>,
    texture_id: imgui::TextureId,
    ppu_texture: glow::NativeTexture,
    debug_gui: DebugGui,
}

impl Renderer {
    pub fn new(sdl: &sdl2::Sdl) -> Self {
        let subsystem = sdl.video().unwrap();
        let gl_attr = subsystem.gl_attr();

        gl_attr.set_context_version(3, 3);
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);

        let window = subsystem
            .window("nes_oxide", 1920, 1080)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let gl_context = window.gl_create_context().unwrap();
        window.gl_make_current(&gl_context).unwrap();
        window.subsystem().gl_set_swap_interval(1).unwrap();

        let gl = unsafe {
            glow::Context::from_loader_function(|s| window.subsystem().gl_get_proc_address(s) as _)
        };

        unsafe { gl.enable(glow::FRAMEBUFFER_SRGB) };

        let mut textures = imgui::Textures::<glow::Texture>::default();
        let ppu_texture = unsafe { gl.create_texture() }.unwrap();

        let frame = Frame::default();

        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(ppu_texture));
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as _,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as _,
            );
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB as _,
                256,
                240,
                0,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                Some(&frame.data),
            );
        }

        let texture_id = textures.insert(ppu_texture);

        let mut imgui = imgui::Context::create();

        let platform = imgui_sdl2_support::SdlPlatform::init(&mut imgui);
        let renderer =
            imgui_glow_renderer::Renderer::initialize(&gl, &mut imgui, &mut textures, false)
                .unwrap();

        Self {
            window,
            frame,
            platform,
            gl,
            gl_context,
            imgui,
            renderer,
            textures,
            texture_id,
            ppu_texture,
            debug_gui: DebugGui::default(),
        }
    }

    pub fn handle_event(&mut self, event: &sdl2::event::Event) {
        self.platform.handle_event(&mut self.imgui, event);
    }

    pub fn render(&mut self, cpu: &mut Cpu, event_pump: &sdl2::EventPump) {
        unsafe {
            self.gl
                .bind_texture(glow::TEXTURE_2D, Some(self.ppu_texture));
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as _,
            );
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as _,
            );
            self.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB as _,
                256,
                240,
                0,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                Some(&self.frame.data),
            );
        }

        self.platform
            .prepare_frame(&mut self.imgui, &self.window, event_pump);

        let ui = self.imgui.new_frame();

        self.debug_gui.draw_debug(cpu, ui);

        imgui::Image::new(self.texture_id, [1024.0, 960.0]).build(ui);

        let draw_data = self.imgui.render();

        unsafe {
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }

        self.renderer
            .render(&self.gl, &self.textures, draw_data)
            .unwrap();

        self.window.gl_swap_window();
    }
}
