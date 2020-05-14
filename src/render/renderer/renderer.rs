use crate::input::InputHandler;
use crate::render::camera::Camera;
use crate::render::renderer::ChunkRenderer;
use math::vector::Vector3;

use core::world::World;

#[derive(Default)]
pub struct Renderer {
    chunk_renderer: ChunkRenderer,
}

impl Renderer {
    pub fn update(&mut self, world: &World, player_position: Vector3, input: &InputHandler) {
        self.chunk_renderer.update(world, player_position, input);
    }

    pub fn draw<C: Camera>(&mut self, camera: &C) {
        unsafe {
            gl::ClearColor(116.0 / 255.0, 173.0 / 255.0, 251.0 / 255.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        self.chunk_renderer.draw(camera);
    }
}
