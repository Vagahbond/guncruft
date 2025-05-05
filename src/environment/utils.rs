use bevy::math::IVec3;

///! generate a vec of indices
///! assumes vertices are made of quads, and counter clockwise ordered
#[inline]
pub fn generate_indices(vertex_count: usize) -> Vec<u32> {
    let indices_count = vertex_count / 4;
    let mut indices = Vec::<u32>::with_capacity(indices_count);
    (0..indices_count).into_iter().for_each(|vert_index| {
        let vert_index = vert_index as u32 * 4u32;
        indices.push(vert_index);
        indices.push(vert_index + 1);
        indices.push(vert_index + 2);
        indices.push(vert_index);
        indices.push(vert_index + 2);
        indices.push(vert_index + 3);
    });
    indices
}

#[inline]
pub fn vec3_to_index(pos: IVec3, bounds: i32) -> usize {
    let x_i = pos.x % bounds;
    // let y_i = (pos.y * bounds) % bounds;
    let y_i = pos.y * bounds;
    let z_i = pos.z * (bounds * bounds);
    // let x_i = pos.x % bounds;
    // let y_i = (pos.y / bounds) % bounds;
    // let z_i = pos.z / (bounds * bounds);
    (x_i + y_i + z_i) as usize
}

///! if lying on the edge of our chunk, return the edging chunk
#[inline]
pub fn get_edging_chunk(pos: IVec3) -> Option<IVec3> {
    let mut chunk_dir = IVec3::ZERO;
    if pos.x == 0 {
        chunk_dir.x = -1;
    } else if pos.x == 31 {
        chunk_dir.x = 1;
    }
    if pos.y == 0 {
        chunk_dir.y = -1;
    } else if pos.y == 31 {
        chunk_dir.y = 1;
    }
    if pos.z == 0 {
        chunk_dir.z = -1;
    } else if pos.z == 31 {
        chunk_dir.z = 1;
    }
    if chunk_dir == IVec3::ZERO {
        None
    } else {
        Some(chunk_dir)
    }
}

#[inline]
pub fn index_to_ivec3_bounds(i: i32, bounds: i32) -> IVec3 {
    let x = i % bounds;
    let y = (i / bounds) % bounds;
    let z = i / (bounds * bounds);
    IVec3::new(x, y, z)
}

// pos 18 bits, ao 3 bits, normal 4 bits
// 18-21-25-   left 32-25 = 7
#[inline]
pub fn make_vertex_u32(
    // position: [i32; 3], /*, normal: i32, color: Color, texture_id: u32*/
    pos: IVec3, /*, normal: i32, color: Color, texture_id: u32*/
    ao: u32,
    normal: u32,
    block_type: u32,
) -> u32 {
    pos.x as u32
        | (pos.y as u32) << 6u32
        | (pos.z as u32) << 12u32
        | ao << 18u32
        | normal << 21u32
        | block_type << 25u32
    // | (normal as u32) << 18u32
    // | (texture_id) << 21u32
}
