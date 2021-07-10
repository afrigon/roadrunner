use crate::ops::{Bindable, Drawable};
use crate::render::mesh::Mesh;
use core::block::BlockRegistry;
use core::chunk::{ChunkGridCoordinate, ChunkGroup, CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH};

use gl::types::GLuint;
use math::vector::Vector3;

pub fn generate_mesh(
    group: ChunkGroup,
    block_registry: BlockRegistry,
) -> (ChunkGridCoordinate, ChunkMeshCollection) {
    (
        group.current.coords,
        ChunkMeshCollection::generate(&group, &block_registry),
    )
}

pub struct ChunkMeshCollection {
    solid: ChunkMesh,
    flora: ChunkMesh,
    water: ChunkMesh,
}

impl ChunkMeshCollection {
    pub fn generate(chunks: &ChunkGroup, block_registry: &BlockRegistry) -> Self {
        let mut solid_mesh = ChunkMesh::default();
        let mut flora_mesh = ChunkMesh::default();
        let mut water_mesh = ChunkMesh::default();

        for x in 0..CHUNK_WIDTH {
            for y in 0..CHUNK_HEIGHT {
                for z in 0..CHUNK_DEPTH {
                    let x = x as i8;
                    let y = y as i16;
                    let z = z as i8;

                    let current_block = chunks.get_block(x, y, z).unwrap();

                    if current_block.id == 0 {
                        continue;
                    }

                    if let Some(properties) = block_registry.properties(current_block.id) {
                        let position = Vector3 {
                            x: x as f32,
                            y: y as f32,
                            z: z as f32,
                        };

                        // clean this up with mesh type property
                        if properties.flora {
                            flora_mesh.add_face(CROSS_A_FACE, position, properties.texture.front);
                            flora_mesh.add_face(CROSS_B_FACE, position, properties.texture.front);
                            continue;
                        }

                        if current_block.id == 9 {
                            let block = chunks.get_block(x, y + 1, z);

                            if block.is_none()
                                || (block.unwrap().id != 9
                                    && !block_registry.is_opaque(block.unwrap().id))
                            {
                                water_mesh.add_face(TOP_FACE, position, properties.texture.top);
                            }

                            continue;
                        }

                        // clean this up with block type property
                        let mesh = &mut solid_mesh;

                        let mut block = chunks.get_block(x, y, z - 1);
                        if block.is_none() || !block_registry.is_opaque(block.unwrap().id) {
                            mesh.add_face(FRONT_FACE, position, properties.texture.front);
                        }

                        block = chunks.get_block(x, y, z + 1);
                        if block.is_none() || !block_registry.is_opaque(block.unwrap().id) {
                            mesh.add_face(BACK_FACE, position, properties.texture.back);
                        }

                        block = chunks.get_block(x - 1, y, z);
                        if block.is_none() || !block_registry.is_opaque(block.unwrap().id) {
                            mesh.add_face(LEFT_FACE, position, properties.texture.left);
                        }

                        block = chunks.get_block(x + 1, y, z);
                        if block.is_none() || !block_registry.is_opaque(block.unwrap().id) {
                            mesh.add_face(RIGHT_FACE, position, properties.texture.right);
                        }

                        block = chunks.get_block(x, y + 1, z);
                        if block.is_none() || !block_registry.is_opaque(block.unwrap().id) {
                            mesh.add_face(TOP_FACE, position, properties.texture.top);
                        }

                        block = chunks.get_block(x, y - 1, z);
                        if block.is_none() || !block_registry.is_opaque(block.unwrap().id) {
                            mesh.add_face(BOTTOM_FACE, position, properties.texture.bottom);
                        }
                    }
                }
            }
        }

        Self {
            solid: solid_mesh,
            flora: flora_mesh,
            water: water_mesh,
        }
    }

    pub fn upload_mesh(&mut self) {
        self.solid.generate();
        self.flora.generate();
        self.water.generate();
    }

    pub fn draw_water(&self) {
        unsafe {
            gl::Enable(gl::BLEND);

            self.water.draw();

            gl::Disable(gl::BLEND);
        }
    }
}

impl Drawable for ChunkMeshCollection {
    fn draw(&self) {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);

            self.solid.draw();

            gl::Disable(gl::CULL_FACE);

            self.flora.draw();
        }
    }
}

#[derive(Default)]
pub struct ChunkMesh {
    mesh: Option<Mesh>,
    vertices: Vec<Vector3>,
    vertices_info: Vec<GLuint>,
    vertex_count: GLuint,
    indices: Vec<GLuint>,
}

impl ChunkMesh {
    fn add_face(&mut self, face: Face, position: Vector3, texture_id: u8) {
        for i in 0..4 {
            self.vertices.push(face.vertices[i] + position);

            let info: GLuint = (texture_id as GLuint) << 4
                | ((face.light & 0b11) << 2) as GLuint
                | (i & 0b11) as GLuint;
            self.vertices_info.push(info);
        }

        self.indices.push(self.vertex_count);
        self.indices.push(self.vertex_count + 3);
        self.indices.push(self.vertex_count + 1);
        self.indices.push(self.vertex_count + 1);
        self.indices.push(self.vertex_count + 3);
        self.indices.push(self.vertex_count + 2);

        self.vertex_count += 4;
    }

    fn generate(&mut self) {
        if !self.vertices.is_empty() {
            self.mesh = Some(Mesh::new(&self.vertices, &self.indices));
            self.mesh.as_mut().unwrap().add_vbo_u32(&self.vertices_info);
        }
    }
}

impl Drawable for ChunkMesh {
    fn draw(&self) {
        if let Some(mesh) = self.mesh.as_ref() {
            return mesh.draw();
        }
    }
}

impl Bindable for ChunkMesh {
    fn bind(&self) {
        if let Some(mesh) = self.mesh.as_ref() {
            mesh.bind();
        }
    }

    fn unbind(&self) {
        if let Some(mesh) = self.mesh.as_ref() {
            mesh.unbind();
        }
    }
}

struct Face {
    vertices: [Vector3; 4],
    light: u8,
}

const FRONT_FACE: Face = Face {
    vertices: [
        Vector3 {
            x: -0.5,
            y: 1.0,
            z: -0.5,
        },
        Vector3 {
            x: 0.5,
            y: 1.0,
            z: -0.5,
        },
        Vector3 {
            x: 0.5,
            y: 0.0,
            z: -0.5,
        },
        Vector3 {
            x: -0.5,
            y: 0.0,
            z: -0.5,
        },
    ],
    light: 2,
};

const BACK_FACE: Face = Face {
    vertices: [
        Vector3 {
            x: 0.5,
            y: 1.0,
            z: 0.5,
        },
        Vector3 {
            x: -0.5,
            y: 1.0,
            z: 0.5,
        },
        Vector3 {
            x: -0.5,
            y: 0.0,
            z: 0.5,
        },
        Vector3 {
            x: 0.5,
            y: 0.0,
            z: 0.5,
        },
    ],
    light: 2,
};

const RIGHT_FACE: Face = Face {
    vertices: [
        Vector3 {
            x: 0.5,
            y: 1.0,
            z: -0.5,
        },
        Vector3 {
            x: 0.5,
            y: 1.0,
            z: 0.5,
        },
        Vector3 {
            x: 0.5,
            y: 0.0,
            z: 0.5,
        },
        Vector3 {
            x: 0.5,
            y: 0.0,
            z: -0.5,
        },
    ],
    light: 1,
};

const LEFT_FACE: Face = Face {
    vertices: [
        Vector3 {
            x: -0.5,
            y: 1.0,
            z: 0.5,
        },
        Vector3 {
            x: -0.5,
            y: 1.0,
            z: -0.5,
        },
        Vector3 {
            x: -0.5,
            y: 0.0,
            z: -0.5,
        },
        Vector3 {
            x: -0.5,
            y: 0.0,
            z: 0.5,
        },
    ],
    light: 1,
};

const TOP_FACE: Face = Face {
    vertices: [
        Vector3 {
            x: -0.5,
            y: 1.0,
            z: 0.5,
        },
        Vector3 {
            x: 0.5,
            y: 1.0,
            z: 0.5,
        },
        Vector3 {
            x: 0.5,
            y: 1.0,
            z: -0.5,
        },
        Vector3 {
            x: -0.5,
            y: 1.0,
            z: -0.5,
        },
    ],
    light: 3,
};

const BOTTOM_FACE: Face = Face {
    vertices: [
        Vector3 {
            x: -0.5,
            y: 0.0,
            z: -0.5,
        },
        Vector3 {
            x: 0.5,
            y: 0.0,
            z: -0.5,
        },
        Vector3 {
            x: 0.5,
            y: 0.0,
            z: 0.5,
        },
        Vector3 {
            x: -0.5,
            y: 0.0,
            z: 0.5,
        },
    ],
    light: 0,
};

const CROSS_A_FACE: Face = Face {
    vertices: [
        Vector3 {
            x: -0.5,
            y: 1.0,
            z: -0.5,
        },
        Vector3 {
            x: 0.5,
            y: 1.0,
            z: 0.5,
        },
        Vector3 {
            x: 0.5,
            y: 0.0,
            z: 0.5,
        },
        Vector3 {
            x: -0.5,
            y: 0.0,
            z: -0.5,
        },
    ],
    light: 3,
};

const CROSS_B_FACE: Face = Face {
    vertices: [
        Vector3 {
            x: -0.5,
            y: 1.0,
            z: 0.5,
        },
        Vector3 {
            x: 0.5,
            y: 1.0,
            z: -0.5,
        },
        Vector3 {
            x: 0.5,
            y: 0.0,
            z: -0.5,
        },
        Vector3 {
            x: -0.5,
            y: 0.0,
            z: 0.5,
        },
    ],
    light: 2,
};
