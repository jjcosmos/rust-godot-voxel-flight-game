use std::collections::{HashMap, HashSet};

const OFFSET: f32 = 453432.5345_f32;
const CHUNK_DIM: i32 = 16;

use godot::{
    classes::{
        mesh::{ArrayType, PrimitiveType}, ArrayMesh, FastNoiseLite, MeshInstance3D, ShaderMaterial
    },
    obj::NewAlloc,
    prelude::*,
};

use crate::player::Player;

#[derive(Clone)]
struct Chunk {
    pos: Vector3i,
    mesh_inst: Gd<MeshInstance3D>,
    array_mesh: Gd<ArrayMesh>,
}

struct ChunkOperation {
    create: bool,
    position: Vector3i,
}

#[derive(GodotClass)]
#[class(base=Node3D)]
struct CubeSpawner {
    #[export]
    seed: i32,
    #[export]
    threshold: f32,
    first_load: bool,
    #[export]
    mesh_size: i32,
    #[export]
    no_spawn: i32,
    #[export]
    view_range: i32,
    #[export]
    player: Option<Gd<Player>>,
    #[export]
    material: Option<Gd<ShaderMaterial>>,
    chunks: HashMap<Vector3i, Chunk>,
    player_last_chunk: Vector3i,
    chunk_operation_queue: Vec<ChunkOperation>,
    noise: Gd<FastNoiseLite>,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for CubeSpawner {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            seed: 1234,
            threshold: 0.0,
            mesh_size: 1,
            no_spawn: 10,
            first_load: true,
            base,

            material: None,
            player: None,
            player_last_chunk: Vector3i::ZERO * 10000,
            chunks: HashMap::new(),
            view_range: 5,
            chunk_operation_queue: Vec::new(),
            noise: FastNoiseLite::new_gd(),
        }
    }

    fn ready(&mut self) {
        self.player_last_chunk = Vector3i::MAX;
        self.noise.set_seed(self.seed);
    }

    fn process(&mut self, _delta: f64) {
        self.poll_chunk_changed();

        self.try_operate();

        let op_len = self.chunk_operation_queue.len();

        if op_len > 0 {
            if self.first_load {
                self.base_mut().emit_signal(
                    "load_progress_updated".into(),
                    &[(1.0 - op_len as f32 / 500.0).to_variant()],
                );
            }
        } else if self.first_load {
            self.first_load = false;
            self.base_mut().emit_signal("load_complete".into(), &[]);
        }
    }
}

#[godot_api]
impl CubeSpawner {
    #[signal]
    fn load_progress_updated(frac: f32) {}

    #[signal]
    fn load_complete() {}

    fn try_operate(&mut self) {
        if let Some(op) = self.chunk_operation_queue.pop() {
            if op.create && self.chunks.contains_key(&op.position) {
                // Load requested for existing chunk
                self.try_operate();
                return;
            }

            if !op.create && !self.chunks.contains_key(&op.position) {
                // delete requested for non existant chunk
                self.try_operate();
                return;
            }

            if op.create {
                self.make_chunk(op.position);
            } else {
                if let Some(chunk) = self.chunks.get_mut(&op.position) {
                    chunk.mesh_inst.queue_free();
                    self.chunks.remove(&op.position);
                    self.try_operate();
                    return;
                }
            }
        }
    }

    fn poll_chunk_changed(&mut self) {
        let Some(player) = self.get_player() else {
            godot_print!("No player assigned!");
            return;
        };

        let player_pos = player.get_global_position();
        let current_chunk = if player.bind().is_dead {Vector3i::ZERO} else {Vector3i {
            x: (player_pos.x / (CHUNK_DIM * self.mesh_size) as f32) as i32,
            y: (player_pos.y / (CHUNK_DIM * self.mesh_size) as f32) as i32,
            z: (player_pos.z / (CHUNK_DIM * self.mesh_size) as f32) as i32,
        }};

        //godot_print!("Current chunk position is {}. Last is {}", current_chunk, self.player_last_chunk);

        if current_chunk != self.player_last_chunk {
            self.base_mut().emit_signal(
                "current_chunk_changed".into(),
                &[current_chunk.to_variant()],
            );
            self.player_last_chunk = current_chunk;

            let mut to_push = vec![];
            // Push all chunks in player radius
            for x in current_chunk.x - self.view_range..current_chunk.x + self.view_range {
                for y in current_chunk.y - self.view_range..current_chunk.y + self.view_range {
                    for z in current_chunk.z - self.view_range..current_chunk.z + self.view_range {
                        to_push.push(ChunkOperation {
                            create: true,
                            position: Vector3i { x, y, z },
                        });
                    }
                }
            }

            // Check if loaded chunks are part of the new chunk operations
            // if not, push a remove operation for the chunk
            for loaded in self.chunks.keys() {
                if !to_push.iter().any(|c| c.position == *loaded) {
                    self.chunk_operation_queue.push(ChunkOperation {
                        create: false,
                        position: *loaded,
                    });
                }
            }

            //godot_print!("Chunk changed. Pushing {} operations", to_push.len());
            // add the remainder of the operations
            self.chunk_operation_queue.append(&mut to_push);
        }
    }

    fn make_chunk(&mut self, chunk_space_position: Vector3i) {
        //godot_print!("Making chunk at: {}", chunk_space_position);
        let mut mesh_inst = MeshInstance3D::new_alloc();
        let array_mesh = ArrayMesh::new_gd();

        mesh_inst.set_name(chunk_space_position.to_string().to_godot());

        let mut positions = vec![];
        let mesh_size = self.get_mesh_size();
        // comb the cube of points around the origin
        for x in 0..CHUNK_DIM {
            for y in 0..CHUNK_DIM {
                for z in 0..CHUNK_DIM {
                    // translate chunk index into world offset
                    // then offset by the chunk's root position
                    let world_position = Vector3::new(x as f32, y as f32, z as f32)
                        * mesh_size as f32
                        + Vector3::new(
                            chunk_space_position.x as f32,
                            chunk_space_position.y as f32,
                            chunk_space_position.z as f32,
                        ) * mesh_size as f32
                            * CHUNK_DIM as f32;

                    // Check safe zone
                    if (world_position.x).abs() as i32 > self.no_spawn
                        || (world_position.y).abs() as i32 > self.no_spawn
                        || (world_position.z).abs() as i32 > self.no_spawn
                    {
                        let val = self.noise.get_noise_3d(
                            world_position.x as f32 + OFFSET,
                            world_position.y as f32 + OFFSET,
                            world_position.z as f32 + OFFSET,
                        );
                        if val > self.threshold {
                            positions.push(Vector3i {
                                x: world_position.x as i32,
                                y: world_position.y as i32,
                                z: world_position.z as i32,
                            });
                        }
                    }
                }
            }
        }

        let mut chunk = Chunk {
            pos: chunk_space_position,
            //blocks: HashSet::from_iter(positions.iter().cloned()),
            mesh_inst,
            array_mesh,
        };

        let mut mesh_info = MeshInfo {
            vertices: PackedVector3Array::new(),
            normals: PackedVector3Array::new(),
            indices: PackedInt32Array::new(),
        };

        let position_set = HashSet::from_iter(positions.iter());

        for pos in &positions {
            CubeSpawner::construct_cube(
                Vector3 {
                    x: pos.x as f32,
                    y: pos.y as f32,
                    z: pos.z as f32,
                },
                &mut mesh_info,
                &position_set,
                self.mesh_size,
            );
        }

        let mut array: VariantArray = VariantArray::new();
        array.resize(ArrayType::MAX.ord() as usize, &Variant::nil());
        array.set(
            ArrayType::VERTEX.ord() as usize,
            mesh_info.vertices.to_variant(),
        );
        array.set(
            ArrayType::INDEX.ord() as usize,
            mesh_info.indices.to_variant(),
        );
        array.set(
            ArrayType::NORMAL.ord() as usize,
            mesh_info.normals.to_variant(),
        );

        self.chunks.insert(chunk.pos, chunk.clone());

        if positions.len() != 0 {
            chunk
                .array_mesh
                .add_surface_from_arrays(PrimitiveType::TRIANGLES, array);
            chunk.array_mesh.surface_set_material(0, self.get_material());
            chunk.mesh_inst.set_mesh(chunk.array_mesh);
            chunk.mesh_inst.create_trimesh_collision();
        }

        self.base_mut().add_child(chunk.mesh_inst);
    }

    pub fn construct_cube(
        position: Vector3,
        mesh_info: &mut MeshInfo,
        chunk_positions: &HashSet<&Vector3i>,
        mesh_size: i32,
    ) {
        let p_as_int: Vector3i = Vector3i {
            x: position.x as i32,
            y: position.y as i32,
            z: position.z as i32,
        };

        //let half_size = mesh_size / 2.0;
        let f_mesh_size = mesh_size as f32;

        // Clockwise winding order

        {
            // -------------- y axis ------------- //
            let has_pos_y_n = chunk_positions.contains(&Vector3i {
                x: p_as_int.x,
                y: p_as_int.y + mesh_size,
                z: p_as_int.z,
            });

            let has_neg_y_n = chunk_positions.contains(&Vector3i {
                x: p_as_int.x,
                y: p_as_int.y - mesh_size,
                z: p_as_int.z,
            });

            if !has_pos_y_n {
                // UP
                let a = position + Vector3::UP * f_mesh_size;
                // UP FORWARD
                let b = position + Vector3::UP * f_mesh_size + Vector3::FORWARD * f_mesh_size;
                // UP FORWARD RIGHT
                let c = position
                    + Vector3::UP * f_mesh_size
                    + Vector3::FORWARD * f_mesh_size
                    + Vector3::RIGHT * f_mesh_size;
                // UP RIGHT
                let d = position + Vector3::UP * f_mesh_size + Vector3::RIGHT * f_mesh_size;

                CubeSpawner::make_quad(&a, &b, &c, &d, &Vector3::UP, mesh_info);
            }

            if !has_neg_y_n {
                // REVERSED FROM ABOVE
                // ORIGIN
                let a = position;
                // ORIGIN FORWARD
                let b = position + Vector3::FORWARD * f_mesh_size;
                // ORIGIN FORWARD RIGHT
                let c = position + Vector3::FORWARD * f_mesh_size + Vector3::RIGHT * f_mesh_size;
                // ORIGIN RIGHT
                let d = position + Vector3::RIGHT * f_mesh_size;

                CubeSpawner::make_quad(&d, &c, &b, &a, &Vector3::DOWN, mesh_info);
            }
        }
        {
            // -------------- x axis ------------- //
            let has_pos_x_n = chunk_positions.contains(&Vector3i {
                x: p_as_int.x + mesh_size,
                y: p_as_int.y,
                z: p_as_int.z,
            });

            let has_neg_x_n = chunk_positions.contains(&Vector3i {
                x: p_as_int.x - mesh_size,
                y: p_as_int.y,
                z: p_as_int.z,
            });

            if !has_pos_x_n {
                // RIGHT
                let a = position + Vector3::RIGHT * f_mesh_size;
                // RIGHT UP
                let b = position + Vector3::RIGHT * f_mesh_size + Vector3::UP * f_mesh_size;
                // RIGHT UP FORWARD
                let c = position
                    + Vector3::RIGHT * f_mesh_size
                    + Vector3::UP * f_mesh_size
                    + Vector3::FORWARD * f_mesh_size;
                // RIGHT FORWARD
                let d = position + Vector3::RIGHT * f_mesh_size + Vector3::FORWARD * f_mesh_size;

                CubeSpawner::make_quad(&a, &b, &c, &d, &Vector3::RIGHT, mesh_info);
            }

            if !has_neg_x_n {
                // ORIGIN
                let a = position;
                // ORIGIN UP
                let b = position + Vector3::UP * f_mesh_size;
                // ORIGIN UP FORWARD
                let c = position + Vector3::UP * f_mesh_size + Vector3::FORWARD * f_mesh_size;
                // ORIGIN FORWARD
                let d = position + Vector3::FORWARD * f_mesh_size;

                CubeSpawner::make_quad(&d, &c, &b, &a, &Vector3::LEFT, mesh_info);
            }
        }
        {
            // -------------- z axis ------------- //
            let has_pos_z_n = chunk_positions.contains(&Vector3i {
                x: p_as_int.x,
                y: p_as_int.y,
                z: p_as_int.z + mesh_size,
            });

            let has_neg_z_n = chunk_positions.contains(&Vector3i {
                x: p_as_int.x,
                y: p_as_int.y,
                z: p_as_int.z - mesh_size,
            });

            // Forward face
            if !has_neg_z_n {
                // FORWARD
                let a = position + Vector3::FORWARD * f_mesh_size;
                // FORWARD RIGHT
                let b = position + Vector3::FORWARD * f_mesh_size + Vector3::RIGHT * f_mesh_size;
                // FORWARD RIGHT UP
                let c = position
                    + Vector3::FORWARD * f_mesh_size
                    + Vector3::RIGHT * f_mesh_size
                    + Vector3::UP * f_mesh_size;
                // FORWARD UP
                let d = position + Vector3::FORWARD * f_mesh_size + Vector3::UP * f_mesh_size;

                CubeSpawner::make_quad(&a, &b, &c, &d, &Vector3::FORWARD, mesh_info);
            }

            // Backward face
            if !has_pos_z_n {
                // ORIGIN
                let a = position;
                // ORIGIN RIGHT
                let b = position + Vector3::RIGHT * f_mesh_size;
                // ORIGIN RIGHT UP
                let c = position + Vector3::RIGHT * f_mesh_size + Vector3::UP * f_mesh_size;
                // ORIGIN UP
                let d = position + Vector3::UP * f_mesh_size;

                CubeSpawner::make_quad(&d, &c, &b, &a, &Vector3::BACK, mesh_info);
            }
        }
    }

    fn make_quad(
        a: &Vector3,
        b: &Vector3,
        c: &Vector3,
        d: &Vector3,
        normal_dir: &Vector3,
        mesh_info: &mut MeshInfo,
    ) {
        let vert_array_len = mesh_info.vertices.len() as i32;

        mesh_info.indices.extend_array(
            &[
                vert_array_len,
                vert_array_len + 1,
                vert_array_len + 2,
                vert_array_len,
                vert_array_len + 2,
                vert_array_len + 3,
            ]
            .iter()
            .cloned()
            .collect::<PackedInt32Array>(),
        );

        mesh_info
            .vertices
            .extend_array(&[*a, *b, *c, *d].iter().cloned().collect());

        mesh_info.normals.extend_array(
            &[*normal_dir, *normal_dir, *normal_dir, *normal_dir]
                .iter()
                .cloned()
                .collect(),
        );
    }
}

struct MeshInfo {
    vertices: PackedVector3Array,
    normals: PackedVector3Array,
    indices: PackedInt32Array,
}
