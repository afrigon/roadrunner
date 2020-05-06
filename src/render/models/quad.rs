use crate::math::vector::{v2, v3};
use crate::render::models::Model;
use crate::utils::traits::Bindable;

pub struct Quad {
    model: Model,
}

impl Quad {
    pub fn new_face() -> Quad {
        Quad::new(
            v3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            v3 {
                x: 1.0,
                y: 1.0,
                z: 0.0,
            },
            v3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            v3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        )
    }

    pub fn new(v1: v3, v2: v3, v3: v3, v4: v3) -> Quad {
        Quad {
            model: Model::new_textured(
                vec![v1, v2, v3, v4],
                vec![
                    v2 { x: 0.0, y: 0.0 },
                    v2 { x: 1.0, y: 0.0 },
                    v2 { x: 1.0, y: 1.0 },
                    v2 { x: 0.0, y: 1.0 },
                ],
                vec![0, 3, 1, 1, 3, 2],
            ),
        }
    }
}

impl Bindable for Quad {
    fn bind(&self) {
        self.model.bind();
    }

    fn unbind(&self) {
        self.model.unbind();
    }
}
