use crate::game::TextureDatabase;
use crate::ops::{Bindable, Drawable};
use crate::render::camera::Camera;
use crate::render::mesh::chunk_mesh::{generate_mesh, ChunkMeshCollection};
use crate::render::shaders::ShaderProgram;
use crate::render::texture::TextureArray;

use core::block::BlockRegistry;
use core::chunk::{Chunk, ChunkGridCoordinate, CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH};
use core::world::{World, LOAD_DISTANCE};
use math::container::{Volume, AABB};
use math::vector::Vector3;
use std::collections::{HashSet, HashMap};
use std::fs;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use core::utils::ThreadPool;

#[cfg(feature = "watchers")]
use crate::utils::watcher::*;

// TODO: remove the dependancy to glutin from this file.
use crate::input::InputHandler;
use glutin::event::VirtualKeyCode;

const TEXTURE_RESOLUTION: u32 = 16;
const MIN_RENDER_DISTANCE: u8 = 2;
const FOG: Vector3 = Vector3 {
    x: 0.62,
    y: 0.76,
    z: 1.0,
};

fn load_textures() -> TextureArray {
    let database = TextureDatabase::new();
    let textures = TextureArray::new(TEXTURE_RESOLUTION, database.len() as u32, 2);

    for (i, file) in database.iter() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("res/textures/block/{}.png", file));
        textures.add_file(&path, (*i as u32) - 1);
    }

    textures
}

type ChunkLoadingChannel = (Sender<(ChunkGridCoordinate, ChunkMeshCollection)>, Receiver<(ChunkGridCoordinate, ChunkMeshCollection)>);

pub struct ChunkRenderer {
    program: ShaderProgram,
    textures: TextureArray,
    meshes: HashMap<ChunkGridCoordinate, ChunkMeshCollection>,
    block_registry: BlockRegistry,
    pub render_distance: u8,

    // Threading
    chunk_loading_chan: ChunkLoadingChannel,
    threadpool: ThreadPool,
    loading_chunks: HashSet<ChunkGridCoordinate>,

    #[cfg(feature = "watchers")]
    texture_watcher: Watcher,
}

impl ChunkRenderer {
    pub fn new() -> Self {
        let vertex_src: &'static str = r#"
            #version 410 core

            layout (location=0) in vec3 position;
            layout (location=1) in uint info;

            out vec3 world_position;
            out vec2 uv;
            out float light;
            flat out uint texture_id;

            vec2 uvs[4] = vec2[4](
                vec2(0.0f, 0.0f),
                vec2(1.0f, 0.0f),
                vec2(1.0f, 1.0f),
                vec2(0.0f, 1.0f)
            );

            uniform vec2 chunk_position;
            uniform mat4 projection_view; // projection * view

            void main() {
                uint uv_index = info & 3u;

                uv = uvs[uv_index];
                light = float((info >> 2u & 3u) + 2) / 5.0 ;
                texture_id = info >> 4u;

                world_position = vec3(chunk_position.x, 0, chunk_position.y) + position;

                gl_Position = projection_view * vec4(world_position, 1.0);
            }
        "#;

        let fragment_src: &'static str = r#"
            #version 410 core

            in vec3 world_position;
            in vec2 uv;
            in float light;
            flat in uint texture_id;

            out vec4 color;
            //layout(location = 0) out vec4 color;

            uniform sampler2DArray diffuse_textures;
            uniform vec3 camera_position;
            uniform vec3 fog_color;
            uniform uint render_distance;

            vec4 get_color(uint id) {
                return light * texture(diffuse_textures, vec3(uv, id));
            }

            vec4 apply_fog(vec4 diffuse) {
                float fog_max = max(32.0, float(render_distance * 16));
                float fog_min = max(16.0, fog_max - 64.0);

                float distance = length(camera_position - world_position);
                float fog = (distance - fog_min) / (fog_max - fog_min);
                fog = max(0.0, min(fog, 1.0));

                return (1.0 - fog) * diffuse + fog * vec4(fog_color, diffuse.a);
            }

            void main() {
                if (texture_id == 2 || texture_id == 10 || texture_id == 4 || texture_id == 25) {
                    vec4 cheapColorMapOutput = vec4(0.492, 0.762, 0.348, 1.0); // jungle
                    //vec4 cheapColorMapOutput = vec4(0.73, 0.71, 0.395, 1.0); // desert

                    if (texture_id == 4) {
                        color = get_color(10) * cheapColorMapOutput;

                        if (color.w == 0.0) {
                            color = get_color(texture_id - 1);
                        }
                    } else {
                        color = get_color(texture_id - 1) * cheapColorMapOutput;
                    }

                    color = apply_fog(color);

                    if (color.a < 0.01) {
                        discard;
                    }

                    return;
                }

                if (texture_id == 7) {
                    // vec4 water_map = vec4(0.329, 0.631, 1.0, 1.0);
                    vec4 water_map = vec4(0.329, 0.584, 0.918, 1.0);

                    color = get_color(25) * water_map * get_color(texture_id - 1);
                    color = apply_fog(color);

                    return;
                }

                color = apply_fog(get_color(texture_id - 1));

                if (color.a < 0.01) {
                    discard;
                }
            }
        "#;

        let textures = load_textures();

        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("res/data/blocks.json");
        let path = path.to_str().unwrap();

        let data =
            fs::read_to_string(path).expect("<block_database> Could not read data from file");
        let block_registry = BlockRegistry::new(serde_json::from_str(&data).unwrap());

        match ShaderProgram::new(vertex_src, fragment_src) {
            Ok(program) => Self {
                program,
                textures,
                meshes: HashMap::new(),
                block_registry,
                render_distance: LOAD_DISTANCE,

                chunk_loading_chan: channel(),
                threadpool: ThreadPool::new(8),
                loading_chunks: HashSet::new(),

                #[cfg(feature = "watchers")]
                texture_watcher: Watcher::new(&Path::new(env!("CARGO_MANIFEST_DIR")).join("res/textures")),
            },
            Err(err) => {
                panic!(
                    "<chunk-renderer> could not compile the shader program:\n\n{}\n",
                    err
                );
            }
        }
    }

    pub fn update(&mut self, world: &World, input: &InputHandler) {
        if input.key_pressed(VirtualKeyCode::J) && self.render_distance > MIN_RENDER_DISTANCE {
            self.render_distance -= 1;
        }

        if input.key_pressed(VirtualKeyCode::K) && self.render_distance < LOAD_DISTANCE {
            self.render_distance += 1;
        }

        #[cfg(feature = "watchers")]
        if self.texture_watcher.poll() {
            self.textures = load_textures();
        }

        let (_, receiver) = &self.chunk_loading_chan;
        match receiver.try_recv() {
            Ok((coords, mut chunk)) => {
                self.loading_chunks.remove(&coords);
                chunk.upload_mesh();
                self.meshes.insert(coords, chunk);
            }
            Err(_) => (),
        };

        // remove unloaded chunk
        self.meshes.retain(|coords, _| 
            world.chunks.contains_key(coords)
        );

        // generate missing geometry for loaded chunks
        for coords in world.chunks.keys() {
            if !self.meshes.contains_key(coords) && !self.loading_chunks.contains(coords) {
                let chunk_group = world.get_chunk_group(*coords);

                if chunk_group.is_none() {
                    continue;
                }

                let (sender, _) = &self.chunk_loading_chan;
                let tx = sender.clone();
                let registry = self.block_registry.clone();

                self.threadpool.run(move ||
                    tx.send(
                       generate_mesh(
                           chunk_group.unwrap(),
                           registry
                       )
                    ).unwrap()
                );

                self.loading_chunks.insert(*coords);

            }
        }
    }

    pub fn draw<C: Camera>(&self, camera: &C) {
        self.program.use_program();
        self.program
            .set_uniform_m4("projection_view", camera.projection_view());
        self.program
            .set_uniform_texture("diffuse_textures", self.textures.unit());
        self.program
            .set_uniform_v3("camera_position", camera.position());
        self.program.set_uniform_v3("fog_color", FOG);
        self.program
            .set_uniform_u32("render_distance", self.render_distance as u32);

        self.textures.bind();

        let visible_chunks = self.meshes.iter().filter(|(coords, _)| {
            let position = coords.abs();

            let chunk_volume = AABB::new(Volume::new(
                position.x as i64,
                0,
                position.y as i64,
                CHUNK_WIDTH as i64,
                CHUNK_HEIGHT as i64,
                CHUNK_DEPTH as i64,
            ));

            camera.frustum().contains(&chunk_volume)
        });

        for (coords, mesh) in visible_chunks.clone() {
            self.program.set_uniform_v2("chunk_position", coords.abs());

            mesh.draw();
        }

        for (coords, mesh) in visible_chunks {
            self.program.set_uniform_v2("chunk_position", coords.abs());

            mesh.draw_water();
        }
    }
}
