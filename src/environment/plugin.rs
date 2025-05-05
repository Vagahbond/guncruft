use crate::environment::engine::*;
use bevy::prelude::*;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Engine::default());
        app.add_systems(PostUpdate, (start_data_tasks, start_mesh_tasks));
        app.add_systems(Update, start_modifications);
        app.add_systems(
            Update,
            ((join_data, join_mesh), (unload_data, unload_mesh)).chain(),
        );
    }
}
