extern crate gl;

use sfml::graphics::{RenderStates, RenderTarget,RenderWindow, RectangleShape, Shader};
use sfml::system::{Vector2f};

use crate::grid::{ParticleGrid, ParticleType};
use crate::{RenderContext};

use std::ffi::CString;

use gl::types::{GLfloat, GLenum, GLuint, GLint, GLchar, GLsizeiptr, GLboolean};

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
    program_id: GLuint,
    frag_shader_id: GLuint,
    vert_shader_id: GLuint,
}

impl GlslRenderer {
    pub fn new(
        vert_shader_path: String,
        frag_shader_path: String,
        context: &RenderContext
    ) -> GlslRenderer {
        let vert_shader_src = std::fs::read_to_string(vert_shader_path).expect("shader not found!");
        let vert_shader_id = compile_shader(&vert_shader_src, gl::VERTEX_SHADER);

        let frag_shader_src = std::fs::read_to_string(frag_shader_path).expect("shader not found!");
        let frag_shader_id = compile_shader(&frag_shader_src, gl::FRAGMENT_SHADER);

        let program_id = link_program(vert_shader_id, frag_shader_id);

        GlslRenderer {
            frag_shader_id: frag_shader_id,
            vert_shader_id: vert_shader_id,
            program_id: program_id,
        }
    }
}

static vertices: [GLfloat; 9] = [
    -1.0, -1.0, 0.0,
     1.0, -1.0, 0.0,
     0.0,  1.0, 0.0,
];

impl Renderer for GlslRenderer {
    fn render(&mut self, grid: &ParticleGrid) {
        unsafe {
            let mut vertex_array_id: GLuint = 0;
            gl::GenVertexArrays(1, &mut vertex_array_id);
            gl::BindVertexArray(vertex_array_id);

            // This will identify our vertex buffer
            let mut vertex_buffer_id: GLuint = 0;
            // Generate 1 buffer, put the resulting identifier in vertexbuffer
            gl::GenBuffers(1, &mut vertex_buffer_id);
            // The following commands will talk about our 'vertexbuffer' buffer
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_id);
            // Give our vertices to OpenGL.
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
                std::mem::transmute(&vertices[0]),
                gl::STATIC_DRAW
            );

            // NOTE: Until this point, it should all be in the new function

            // 1st attribute buffer : vertices
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_id);
            gl::VertexAttribPointer(
                0,                // attribute 0. No particular reason for 0, but must match the layout in the shader.
                3,                // size
                gl::FLOAT,        // type
                gl::FALSE,        // normalized?
                0,                // stride
                std::ptr::null(), // array buffer offset
            );
            // Draw the triangle !
            gl::DrawArrays(gl::TRIANGLES, 0, 3); // Starting from vertex 0; 3 vertices total -> 1 triangle
            gl::DisableVertexAttribArray(0);
        }
    }
}

fn link_program(vert_shader_id: GLuint, frag_shader_id: GLuint) -> GLuint {
    let program_id = unsafe { gl::CreateProgram() };

    let successful: bool;

    unsafe {
        gl::AttachShader(program_id, vert_shader_id);
        gl::AttachShader(program_id, frag_shader_id);
        gl::LinkProgram(program_id);

        successful = {
            let mut result: GLint = 0;
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut result);
            result != 0
        };
    }

    if successful {
        program_id
    } else {
        panic!("Failed to link the program:\n{}", get_link_log(program_id))
    }
}

fn get_link_log(program_id: GLuint) -> String {
    let mut len = 0;
    unsafe { gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len) };
    assert!(len > 0);

    let mut buf = Vec::with_capacity(len as usize);
    let buf_ptr = buf.as_mut_ptr() as *mut gl::types::GLchar;
    unsafe {
        gl::GetProgramInfoLog(program_id, len, std::ptr::null_mut(), buf_ptr);
        buf.set_len(len as usize);
    };

    match String::from_utf8(buf) {
        Ok(log) => log,
        Err(vec) => panic!("Could not convert link log from buffer: {}", vec)
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
