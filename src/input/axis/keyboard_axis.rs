use glutin::event::VirtualKeyCode;
use crate::input::InputHandler;
use crate::input::axis::Axis;

pub struct KeyboardAxis {
    up: VirtualKeyCode,
    down: VirtualKeyCode,
}

impl KeyboardAxis {
    pub fn new(up: VirtualKeyCode, down: VirtualKeyCode) -> Self {
        Self {
            up: up,
            down: down,
        }
    }
}

impl Axis for KeyboardAxis {
    fn get_value(&self, input_handler: &InputHandler) -> f32 {
        let mut val = 0.0;
        if input_handler.key_pressed(self.up) { val += 1.0 };
        if input_handler.key_pressed(self.down) { val -= 1.0 };
        return val
    }
}
