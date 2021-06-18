extern crate gl;

use crate::grid::{ParticleGrid, ParticleType, MAX_FILL};
use crate::{RenderContext};

use std::ffi::CString;

use gl::types::{GLfloat, GLenum, GLuint, GLint, GLchar, GLsizeiptr};

pub trait Renderer {
    fn render(&mut self, grid: &ParticleGrid, context: &RenderContext);
}

pub struct GlslRenderer {
    vertex_array_id: GLuint,
    grid_buffer_id: GLuint,
    program_id: GLuint,
    pixel_data: Vec<u32>,
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

        let grid_size = (context.grid_width * context.grid_height) as usize;

        type DataType = u32;
        let pixel_data: Vec<DataType> = (0..(grid_size as DataType)).collect();
        let mem_size = std::mem::size_of::<DataType>() * grid_size;

        GlslRenderer {
            vertex_array_id: GlslRenderer::load_fullscreen_vertex_buffer(),
            grid_buffer_id: GlslRenderer::allocate_grid_buffer(mem_size),
            program_id: program_id,
            pixel_data: pixel_data,
        }
    }

    fn allocate_grid_buffer(mem_size: usize) -> GLuint {
        let mut grid_buffer_id: GLuint = 0;

        unsafe {
            gl::GenBuffers(1, &mut grid_buffer_id);
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, grid_buffer_id);

            gl::NamedBufferData(
                grid_buffer_id,
                mem_size as GLsizeiptr,
                std::ptr::null(),
                // TODO: Try STREAM_DRAW to see performance diff
                gl::DYNAMIC_DRAW
            );

            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 3, grid_buffer_id);
        }

        grid_buffer_id
    }

    fn load_fullscreen_vertex_buffer() -> GLuint {
        let mut vertex_array_id: GLuint = 0;

        let vertices: [GLfloat; 18] = [
            // Top/left triangle
            -1.0,  1.0, 0.0,
            -1.0, -1.0, 0.0,
            1.0,  1.0, 0.0,
            // Bottom/right triangle
            1.0,  1.0, 0.0,
            1.0, -1.0, 0.0,
            -1.0, -1.0, 0.0,
        ];

        unsafe {
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
        }

        vertex_array_id
    }

    fn get_uniform_location(&mut self, name: &str) -> GLint {
        unsafe {
            let c_str = CString::new(name.as_bytes()).unwrap();
            gl::GetUniformLocation(self.program_id, c_str.as_ptr())
        }
    }

    fn set_uniform_f32(&mut self, name: &str, value: f32) {
        let loc = self.get_uniform_location(name);

        unsafe {
            gl::Uniform1f(loc, value);
        }
    }

    pub fn set_uniform_i32(&mut self, name: &str, value: i32) {
        let loc = self.get_uniform_location(name);

        unsafe {
            gl::Uniform1i(loc, value);
        }
    }
}

impl Renderer for GlslRenderer {
    fn render(&mut self, grid: &ParticleGrid, context: &RenderContext) {
        self.set_uniform_i32("grid_width", context.grid_width);
        self.set_uniform_i32("win_height", context.win_height as i32);
        self.set_uniform_f32("scale", context.scale);
        self.set_uniform_i32("max_fill", MAX_FILL as i32);

        self.set_uniform_i32("mouse_x", (context.mouse_x as f32 / context.scale) as i32);
        self.set_uniform_i32("mouse_y", (context.mouse_y as f32 / context.scale) as i32);

        for (i, p) in grid.grid.iter().enumerate() {
            let type_id: u32 = match p.p_type {
                ParticleType::Water => 1 << 8,
                ParticleType::Sand  => 2 << 8,
                ParticleType::Wood  => 3 << 8,
                _                   => 0 << 8,
            };

            self.pixel_data[i] = type_id + p.fill_ratio as u32;
        }

        unsafe {
            gl::NamedBufferSubData(
                self.grid_buffer_id,
                0,
                (self.pixel_data.len() * std::mem::size_of::<u32>()) as isize,
                std::mem::transmute(self.pixel_data.as_ptr())
            );

            gl::UseProgram(self.program_id);

            // 1st attribute buffer : vertices
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_array_id);
            gl::VertexAttribPointer(
                0,                // attribute 0. No particular reason for 0, but must match the layout in the shader.
                3,                // size
                gl::FLOAT,        // type
                gl::FALSE,        // normalized?
                0,                // stride
                std::ptr::null(), // array buffer offset
            );

            // Starting from vertex 0; 6 vertices total -> 2 triangles
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
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
