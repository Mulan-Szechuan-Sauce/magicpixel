use sfml::graphics::{RenderStates, RenderTarget,RenderWindow, RectangleShape, Shader};
use sfml::system::{Vector2f};

use crate::grid::{ParticleGrid, ParticleType};
use crate::{RenderContext};

pub trait Renderer {
    fn render(&mut self, window: &mut RenderWindow, grid: &ParticleGrid);
}

pub struct GlslRenderer<'a> {
    shader: Shader<'a>,
    rect: RectangleShape<'a>,
    particles: Vec<f32>,
}

impl <'a> GlslRenderer<'a> {
    pub fn new(shader_path: String, context: &RenderContext) -> GlslRenderer<'a> {
        let mut shader = Shader::from_file(None, None, Some(shader_path.as_str())).unwrap();
        shader.set_uniform_int("win_height", context.win_height as i32);
        shader.set_uniform_float("scale", context.scale);
        shader.set_uniform_int("grid_width", context.grid_width as i32);

        GlslRenderer {
            rect: RectangleShape::with_size(
                Vector2f::new(context.win_width as f32, context.win_height as f32)
            ),
            shader: shader,
            particles: vec![0.0; context.grid_width as usize * context.grid_height as usize],
        }
    }
}

impl <'a> Renderer for GlslRenderer<'a> {
    fn render(&mut self, window: &mut RenderWindow, grid: &ParticleGrid) {
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
        window.draw_with_renderstates(&self.rect, &states);
    }
}
