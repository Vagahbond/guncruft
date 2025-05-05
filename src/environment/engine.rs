use std::sync::Arc;

use bevy::{
    asset::{LoadState, RenderAssetUsages},
    math::IVec3,
    platform::collections::{HashMap, HashSet},
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future},
};

use super::{
    block::{BlockData, BlockType},
    chunk::{CHUNK_SIZE3, ChunkData, ChunksRefs},
    mesher::{self, ChunkMesh},
    rendering::{ATTRIBUTE_VOXEL, GlobalChunkMaterial},
    scanner::Scanner,
    utils::{get_edging_chunk, vec3_to_index},
};

pub const MAX_DATA_TASKS: usize = 64;
pub const MAX_MESH_TASKS: usize = 32;

pub struct ChunkModification(pub IVec3, pub BlockType);

///! level of detail
#[derive(Copy, Clone)]
pub enum Lod {
    L32,
    L16,
    L8,
    L4,
    L2,
}

impl Lod {
    ///! the amount of voxels per axis
    pub fn size(&self) -> i32 {
        match self {
            Lod::L32 => 32,
            Lod::L16 => 16,
            Lod::L8 => 8,
            Lod::L4 => 4,
            Lod::L2 => 2,
        }
    }

    ///! how much to multiply to reach next voxel
    ///! lower lod gives higher jump
    pub fn jump_index(&self) -> i32 {
        match self {
            Lod::L32 => 1,
            Lod::L16 => 2,
            Lod::L8 => 4,
            Lod::L4 => 8,
            Lod::L2 => 16,
        }
    }
}

#[derive(Resource)]
pub struct Engine {
    pub world_data: HashMap<IVec3, Arc<ChunkData>>,
    pub load_data_queue: Vec<IVec3>,
    pub load_mesh_queue: Vec<IVec3>,
    pub unload_data_queue: Vec<IVec3>,
    pub unload_mesh_queue: Vec<IVec3>,
    pub data_tasks: HashMap<IVec3, Option<Task<ChunkData>>>,
    pub mesh_tasks: Vec<(IVec3, Option<Task<Option<ChunkMesh>>>)>,
    pub chunk_entities: HashMap<IVec3, Entity>,
    pub lod: Lod,
    pub chunk_modifications: HashMap<IVec3, Vec<ChunkModification>>,
}

impl Default for Engine {
    fn default() -> Engine {
        return Engine {
            world_data: HashMap::new(),
            load_data_queue: Vec::new(),
            load_mesh_queue: Vec::new(),
            unload_data_queue: Vec::new(),
            unload_mesh_queue: Vec::new(),
            data_tasks: HashMap::new(),
            mesh_tasks: Vec::new(),
            chunk_entities: HashMap::new(),
            lod: Lod::L32,
            chunk_modifications: HashMap::new(),
        };
    }
}

impl Engine {
    pub fn unload_all_meshes(&mut self, scanner: &Scanner, scanner_transform: &GlobalTransform) {
        // stop all any current proccessing
        self.load_mesh_queue.clear();
        self.mesh_tasks.clear();
        let scan_pos =
            ((scanner_transform.translation() - Vec3::splat(16.0)) * (1.0 / 32.0)).as_ivec3();
        for offset in &scanner.mesh_sampling_offsets {
            let wpos = scan_pos + *offset;
            self.load_mesh_queue.push(wpos);
            // self.unload_mesh_queue.push(wpos);
        }
    }
}

// start
pub fn start_modifications(mut voxel_engine: ResMut<Engine>) {
    let Engine {
        world_data,
        chunk_modifications,
        load_mesh_queue,
        ..
    } = voxel_engine.as_mut();

    for (pos, mods) in chunk_modifications.drain() {
        // say i want to load mesh now :)
        let Some(chunk_data) = world_data.get_mut(&pos) else {
            continue;
        };

        let new_chunk_data = Arc::make_mut(chunk_data);
        let mut adj_chunk_set = HashSet::new();

        for ChunkModification(local_pos, block_type) in mods.into_iter() {
            // Transform position to index in the chunk
            let i = vec3_to_index(local_pos, 32);

            // if the chunk has only one voxel
            if new_chunk_data.voxels.len() == 1 {
                let mut voxels = vec![];
                // Fill the whole chunk with block type from voxel
                for _ in 0..CHUNK_SIZE3 {
                    voxels.push(BlockData {
                        block_type: new_chunk_data.voxels[0].block_type,
                    });
                }
                new_chunk_data.voxels = voxels;
            }

            // apply modification
            new_chunk_data.voxels[i].block_type = block_type;

            // If there is another chunk next to current chunk, we add it to our hashset.
            if let Some(edge_chunk) = get_edging_chunk(local_pos) {
                adj_chunk_set.insert(edge_chunk);
            }
        }

        // Re-do rendenring of adjascent chunks if relevant
        for adj_chunk in adj_chunk_set.into_iter() {
            load_mesh_queue.push(pos + adj_chunk);
        }

        load_mesh_queue.push(pos);
    }
}

///! begin data building tasks for chunks in range
pub fn start_data_tasks(
    mut voxel_engine: ResMut<Engine>,
    scanners: Query<&GlobalTransform, With<Scanner>>,
) {
    //let task_pool = AsyncComputeTaskPool::get();

    let Engine {
        load_data_queue,
        //   data_tasks,
        ..
    } = voxel_engine.as_mut();

    // Get engine's scanner
    let scanner_g = scanners.single().unwrap();

    // adjust scanner position (but to what!?)
    let scan_pos = ((scanner_g.translation() - Vec3::splat(16.0)) * (1.0 / 32.0)).as_ivec3();

    // Sort blocks by distance to the scanner (player?)
    load_data_queue.sort_by(|a, b| {
        a.distance_squared(scan_pos)
            .cmp(&b.distance_squared(scan_pos))
    });

    // Tasks left to compute before either the queue is empty or the task vec is full
    // let tasks_left = (MAX_DATA_TASKS as i32 - data_tasks.len() as i32)
    //    .min(load_data_queue.len() as i32)
    //    .max(0) as usize;

    // Extract elements from load queue and process them
    //for world_pos in load_data_queue.drain(0..tasks_left) {
    //    let k = world_pos;
    //    let task = task_pool.spawn(async move {
    //        let cd = ChunkData::generate(k);
    //        cd
    //    });
    // add thread amd coords to current tasks
    //   data_tasks.insert(world_pos, Some(task));
    //}
}

///! destroy enqueued, chunk data
pub fn unload_data(mut voxel_engine: ResMut<Engine>) {
    let Engine {
        unload_data_queue,
        world_data,
        ..
    } = voxel_engine.as_mut();

    for chunk_pos in unload_data_queue.drain(..) {
        world_data.remove(&chunk_pos);
    }
}

///! begin mesh building tasks for chunks in range
pub fn start_mesh_tasks(
    mut voxel_engine: ResMut<Engine>,
    scanners: Query<&GlobalTransform, With<Scanner>>,
) {
    let task_pool = AsyncComputeTaskPool::get();

    let Engine {
        load_mesh_queue,
        mesh_tasks,
        world_data,
        lod,
        ..
    } = voxel_engine.as_mut();

    let scanner_g = scanners.single().unwrap();

    let scan_pos = ((scanner_g.translation() - Vec3::splat(16.0)) * (1.0 / 32.0)).as_ivec3();

    // Sort blocks by distance to the scanner (player?)
    load_mesh_queue.sort_by(|a, b| {
        a.distance_squared(scan_pos)
            .cmp(&b.distance_squared(scan_pos))
    });

    // Tasks left to compute before either the queue is empty or the task vec is full
    let tasks_left = (MAX_MESH_TASKS as i32 - mesh_tasks.len() as i32)
        .min(load_mesh_queue.len() as i32)
        .max(0) as usize;

    for world_pos in load_mesh_queue.drain(0..tasks_left) {
        // for world_pos in load_mesh_queue.drain(..) {
        let Some(chunks_refs) = ChunksRefs::try_new(world_data, world_pos) else {
            continue;
        };

        let llod = *lod;

        let task = task_pool.spawn(async move { mesher::build_chunk_mesh(&chunks_refs, llod) });

        mesh_tasks.push((world_pos, Some(task)));
    }
}

///! join the chunkdata threads
pub fn join_data(mut voxel_engine: ResMut<Engine>) {
    let Engine {
        world_data,
        data_tasks,
        ..
    } = voxel_engine.as_mut();
    for (world_pos, task_option) in data_tasks.iter_mut() {
        let Some(mut task) = task_option.take() else {
            // should never happend, because we drop None values later
            warn!("someone modified task?");
            continue;
        };

        // Get the new chunk data from the task if it is done
        let Some(chunk_data) = block_on(future::poll_once(&mut task)) else {
            *task_option = Some(task);
            continue;
        };

        // inert the new chunk in the word
        world_data.insert(*world_pos, Arc::new(chunk_data));
    }
    data_tasks.retain(|_k, op| op.is_some());
}

#[derive(Component)]
pub struct WaitingToLoadMeshTag;

pub fn promote_dirty_meshes(
    mut commands: Commands,
    children: Query<(Entity, &Mesh3d, &ChildOf), With<WaitingToLoadMeshTag>>,
    mut parents: Query<&mut Mesh3d, Without<WaitingToLoadMeshTag>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, handle, parent) in children.iter() {
        if let Some(state) = asset_server.get_load_state(handle.id()) {
            match state {
                LoadState::Loaded | LoadState::Failed(_) => {
                    let Ok(mut parent_handle) = parents.get_mut(parent.parent()) else {
                        continue;
                    };
                    info!("updgraded!");
                    // Parent heandle becomes child handle
                    *parent_handle = handle.clone();
                    // Child is despawned
                    commands.entity(entity).despawn();
                }
                LoadState::Loading => {
                    info!("loading cool");
                }
                _ => (),
            }
        }
    }
}

///! join the multithreaded chunk mesh tasks, and construct a finalized chunk entity
pub fn join_mesh(
    mut voxel_engine: ResMut<Engine>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    global_chunk_material: Res<GlobalChunkMaterial>,
) {
    let Engine {
        mesh_tasks,
        chunk_entities,
        ..
    } = voxel_engine.as_mut();

    // Iter through meshing tasks
    for (world_pos, task_option) in mesh_tasks.iter_mut() {
        let Some(mut task) = task_option.take() else {
            // should never happend, because we drop None values later
            warn!("someone modified task?");
            continue;
        };

        let Some(chunk_mesh_option) = block_on(future::poll_once(&mut task)) else {
            // failed polling, keep task alive
            *task_option = Some(task);
            continue;
        };

        let Some(mesh) = chunk_mesh_option else {
            continue;
        };

        let mut bevy_mesh = Mesh::new(
            bevy::render::mesh::PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );

        // give existing chunk mesh attributes to new mesh
        bevy_mesh.insert_attribute(ATTRIBUTE_VOXEL, mesh.vertices.clone());
        bevy_mesh.insert_indices(bevy::render::mesh::Indices::U32(
            mesh.indices.clone().into(),
        ));

        // add the newly created mesh to the game
        let mesh_handle = meshes.add(bevy_mesh);

        // despawn chink from the world
        if let Some(entity) = chunk_entities.get(world_pos) {
            commands.entity(*entity).despawn();
        }

        // spawn chunk entity
        let chunk_entity = commands
            .spawn((
                // Aabb::from_min_max(Vec3::ZERO, Vec3::splat(32.0)),
                Transform::from_translation(world_pos.as_vec3() * Vec3::splat(32.0)),
                Mesh3d(mesh_handle),
                // MeshMaterial3d(global_chunk_material.0.clone()),
            ))
            .id();
        chunk_entities.insert(*world_pos, chunk_entity);
    }
    mesh_tasks.retain(|(_p, op)| op.is_some());
}
