use crate::input::InputHandler;

pub trait Axis {
    fn get_value(&self, input_handler: &InputHandler) -> f32;
}
