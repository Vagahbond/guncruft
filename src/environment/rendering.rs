use bevy::{
    asset::{Asset, Handle},
    ecs::resource::Resource,
    reflect::Reflect,
    render::{
        mesh::{MeshVertexAttribute, VertexFormat},
        render_resource::AsBindGroup,
    },
};

// This is the struct that will be passed to your shader
#[derive(Asset, Reflect, AsBindGroup, Debug, Clone)]
pub struct ChunkMaterial {
    #[uniform(0)]
    pub reflectance: f32,
    #[uniform(0)]
    pub perceptual_roughness: f32,
    #[uniform(0)]
    pub metallic: f32,
}

#[derive(Resource, Reflect)]
pub struct GlobalChunkMaterial(pub Handle<ChunkMaterial>);

// A "high" random id should be used for custom attributes to ensure consistent sorting and avoid collisions with other attributes.
// See the MeshVertexAttribute docs for more info.
pub const ATTRIBUTE_VOXEL: MeshVertexAttribute =
    MeshVertexAttribute::new("Voxel", 988540919, VertexFormat::Uint32);
