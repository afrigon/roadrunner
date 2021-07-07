use crate::input::InputHandler;
use crate::render::camera::{Camera, PerspectiveCamera};

use core::world::WorldCoordinate;
use glutin::event::VirtualKeyCode;
use math::vector::Vector3;

const SPEED: f32 = 20.0;
const ACCELERATION: f32 = 65.0;
const DECELERATION: f32 = 65.0;

const SENSITIVITY: f32 = 0.2;

pub struct MainPlayer {
    pub camera: PerspectiveCamera,
    velocity: Vector3,
}

impl MainPlayer {
    pub fn new(position: WorldCoordinate) -> Self {
        let mut p = Self {
            camera: PerspectiveCamera::new(70.0, 0.1, 1024.0),
            velocity: Vector3::zero(),
        };

        p.set_position(position);
        p
    }

    fn set_position(&mut self, position: WorldCoordinate) {
        self.camera.set_position(position);
    }

    pub fn position(&self) -> Vector3 {
        self.camera.position()
    }

    pub fn update(&mut self, time_delta: f64, input: &InputHandler) {
        let cursor_delta = input.get_cursor_delta();
        let camera_delta = Vector3 {
            x: cursor_delta.y as f32,
            y: cursor_delta.x as f32,
            z: 0.0,
        } * SENSITIVITY;
        let mut camera_angles = self.camera.euler_angles() + camera_delta;

        // make sure you can't do backflips
        if camera_angles.x > 90.0 {
            camera_angles.x = 90.0;
        } else if camera_angles.x < -90.0 {
            camera_angles.x = -90.0;
        }

        // make sure you can spin to infinity
        camera_angles.y %= 360.0;

        self.camera.set_euler_angles(camera_angles);

        let mut axis = Vector3::zero();

        if input.is_key_pressed(VirtualKeyCode::W) {
            axis.z += 1.0;
        }

        if input.is_key_pressed(VirtualKeyCode::S) {
            axis.z -= 1.0;
        }

        if input.is_key_pressed(VirtualKeyCode::A) {
            axis.x -= 1.0;
        }

        if input.is_key_pressed(VirtualKeyCode::D) {
            axis.x += 1.0;
        }

        if input.is_key_pressed(VirtualKeyCode::Space) {
            axis.y += 1.0;
        }

        if input.is_key_pressed(VirtualKeyCode::LShift) {
            axis.y -= 1.0;
        }

        // handle acceleration
        let deceleration = DECELERATION * time_delta as f32;

        if axis.x != 0.0 {
            self.velocity.x += axis.x * ACCELERATION * time_delta as f32;
            self.velocity.x = self.velocity.x.max(-SPEED).min(SPEED);
        } else {
            if self.velocity.x > deceleration {
                self.velocity.x -= deceleration;
            } else if self.velocity.x < -deceleration {
                self.velocity.x += deceleration;
            } else {
                self.velocity.x = 0.0;
            }
        }

        if axis.y != 0.0 {
            self.velocity.y += axis.y * ACCELERATION * time_delta as f32;
            self.velocity.y = self.velocity.y.max(-SPEED).min(SPEED);
        } else {
            if self.velocity.y > deceleration {
                self.velocity.y -= deceleration;
            } else if self.velocity.y < -deceleration {
                self.velocity.y += deceleration;
            } else {
                self.velocity.y = 0.0;
            }
        }

        if axis.z != 0.0 {
            self.velocity.z += axis.z * ACCELERATION * time_delta as f32;
            self.velocity.z = self.velocity.z.max(-SPEED).min(SPEED);
        } else {
            if self.velocity.z > deceleration {
                self.velocity.z -= deceleration;
            } else if self.velocity.z < -deceleration {
                self.velocity.z += deceleration;
            } else {
                self.velocity.z = 0.0;
            }
        }

        let angle = self.camera.euler_angles().y.to_radians();

        let mut delta = Vector3 {
            x: self.velocity.x * angle.cos() + self.velocity.z * angle.sin(),
            y: self.velocity.y,
            z: -self.velocity.x * angle.sin() + self.velocity.z * angle.cos(),
        };

        delta *= time_delta as f32;

        self.set_position(self.camera.position() + delta);
    }
}
