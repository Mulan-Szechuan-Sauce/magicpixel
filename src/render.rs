extern crate gl;

use sfml::graphics::{RenderStates, RenderTarget,RenderWindow, RectangleShape, Shader};
use sfml::system::{Vector2f};

use crate::grid::{ParticleGrid, ParticleType};
use crate::{RenderContext};

use std::ffi::CString;

pub trait Renderer {
    fn render(&mut self, grid: &ParticleGrid);
}

pub struct SfmlRenderer<'a> {
    shader: Shader<'a>,
    rect: RectangleShape<'a>,
    particles: Vec<f32>,
}

impl <'a> SfmlRenderer<'a> {
    pub fn new(shader_path: String, context: &RenderContext) -> SfmlRenderer<'a> {
        let mut shader = Shader::from_file(None, None, Some(shader_path.as_str())).unwrap();
        shader.set_uniform_int("win_height", context.win_height as i32);
        shader.set_uniform_float("scale", context.scale);
        shader.set_uniform_int("grid_width", context.grid_width as i32);

        SfmlRenderer {
            rect: RectangleShape::with_size(
                Vector2f::new(context.win_width as f32, context.win_height as f32)
            ),
            shader: shader,
            particles: vec![0.0; context.grid_width as usize * context.grid_height as usize],
        }
    }
}

impl <'a> Renderer for SfmlRenderer<'a> {
    fn render(&mut self, grid: &ParticleGrid) {
        for (i, p) in grid.grid.iter().enumerate() {
            let val = match p.p_type {
                ParticleType::Water => 1.0,
                _                   => 0.0,
            };

            self.particles[i] = val;
        }

        self.shader.set_uniform_array_float("particles", &self.particles);

        let mut states = RenderStates::default();
        states.set_shader(Some(&self.shader));
        //window.draw_with_renderstates(&self.rect, &states);
    }
}

pub struct GlslRenderer {
}

use gl::types::{GLfloat, GLenum, GLuint, GLint, GLchar, GLsizeiptr};

impl GlslRenderer {
    pub fn new(shader_path: String, context: &RenderContext) -> GlslRenderer {
        let shader_src = std::fs::read_to_string(shader_path).expect("shader not found!");

        compile_shader(&shader_src, gl::FRAGMENT_SHADER);

        GlslRenderer {
        }
    }
}

impl Renderer for GlslRenderer {
    fn render(&mut self, grid: &ParticleGrid) {
    }
}

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader;
    unsafe {
        shader = gl::CreateShader(ty);
        // Attempt to compile the shader
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);

            let mut buf = vec![32u8; len as usize];

            gl::GetShaderInfoLog(
                shader,
                len,
                std::ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );

            let message = String::from_utf8(buf.as_slice().to_vec())
                .ok()
                .expect("ShaderInfoLog not valid utf8");

            println!("Shader compile error:\n{}", message);

            panic!("Rubbish shader, rubbish programmer");
        }
    }
    shader
}
