
/*mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}*/
extern crate gl;

use sdl2::video::{GLContext, Window, GLProfile, SwapInterval};
use sdl2::{Sdl, VideoSubsystem};

pub mod graphics;
pub mod camera;

pub struct Screen {
    pub sdl_context: Sdl,
    pub gl_context: GLContext,
    pub window: Window,
    pub video: VideoSubsystem,
}
impl Screen {
    pub fn new(title: &str, width: u32, height: u32, aa_buffers: Option<u8>, aa_samples: Option<u8>) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video = sdl_context.video().unwrap();
        let attr = video.gl_attr();
        attr.set_context_profile(GLProfile::Core);
        attr.set_context_version(4, 6);
        attr.set_multisample_buffers(aa_buffers.unwrap_or(0));
        attr.set_multisample_samples(aa_samples.unwrap_or(0));
        
        let window = video.window(title, width, height).opengl().build().unwrap();
        let gl_context = window.gl_create_context().unwrap();
        gl::load_with(|name| video.gl_get_proc_address(name) as *const _);
        
        video.gl_set_swap_interval(SwapInterval::VSync).unwrap();
        
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthMask(gl::TRUE);
            gl::DepthFunc(gl::LEQUAL);
            gl::DepthRange(0.0, 1.0);
            
            if attr.multisample_buffers() > 0 || attr.multisample_samples() > 0 {
                gl::Enable(gl::MULTISAMPLE);
            }
            
            gl::Viewport(0, 0, width as i32, height as i32);
        }
        
        Screen {
            sdl_context,
            gl_context,
            window,
            video,
        }
    }
    
    pub fn refresh(&self) {
        self.window.gl_swap_window();
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT); }
    }
}