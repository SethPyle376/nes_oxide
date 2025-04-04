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

        subsystem.gl_set_swap_interval(1).unwrap();

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
        imgui.set_ini_filename(None);

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

        imgui::Image::new(self.texture_id, [256.0, 240.0]).build(ui);

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


#[rustfmt::skip]
pub static SYSTEM_PALLETE: [(u8,u8,u8); 64] = [
   (0x80, 0x80, 0x80), (0x00, 0x3D, 0xA6), (0x00, 0x12, 0xB0), (0x44, 0x00, 0x96), (0xA1, 0x00, 0x5E),
   (0xC7, 0x00, 0x28), (0xBA, 0x06, 0x00), (0x8C, 0x17, 0x00), (0x5C, 0x2F, 0x00), (0x10, 0x45, 0x00),
   (0x05, 0x4A, 0x00), (0x00, 0x47, 0x2E), (0x00, 0x41, 0x66), (0x00, 0x00, 0x00), (0x05, 0x05, 0x05),
   (0x05, 0x05, 0x05), (0xC7, 0xC7, 0xC7), (0x00, 0x77, 0xFF), (0x21, 0x55, 0xFF), (0x82, 0x37, 0xFA),
   (0xEB, 0x2F, 0xB5), (0xFF, 0x29, 0x50), (0xFF, 0x22, 0x00), (0xD6, 0x32, 0x00), (0xC4, 0x62, 0x00),
   (0x35, 0x80, 0x00), (0x05, 0x8F, 0x00), (0x00, 0x8A, 0x55), (0x00, 0x99, 0xCC), (0x21, 0x21, 0x21),
   (0x09, 0x09, 0x09), (0x09, 0x09, 0x09), (0xFF, 0xFF, 0xFF), (0x0F, 0xD7, 0xFF), (0x69, 0xA2, 0xFF),
   (0xD4, 0x80, 0xFF), (0xFF, 0x45, 0xF3), (0xFF, 0x61, 0x8B), (0xFF, 0x88, 0x33), (0xFF, 0x9C, 0x12),
   (0xFA, 0xBC, 0x20), (0x9F, 0xE3, 0x0E), (0x2B, 0xF0, 0x35), (0x0C, 0xF0, 0xA4), (0x05, 0xFB, 0xFF),
   (0x5E, 0x5E, 0x5E), (0x0D, 0x0D, 0x0D), (0x0D, 0x0D, 0x0D), (0xFF, 0xFF, 0xFF), (0xA6, 0xFC, 0xFF),
   (0xB3, 0xEC, 0xFF), (0xDA, 0xAB, 0xEB), (0xFF, 0xA8, 0xF9), (0xFF, 0xAB, 0xB3), (0xFF, 0xD2, 0xB0),
   (0xFF, 0xEF, 0xA6), (0xFF, 0xF7, 0x9C), (0xD7, 0xE8, 0x95), (0xA6, 0xED, 0xAF), (0xA2, 0xF2, 0xDA),
   (0x99, 0xFF, 0xFC), (0xDD, 0xDD, 0xDD), (0x11, 0x11, 0x11), (0x11, 0x11, 0x11)
];