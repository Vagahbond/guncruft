use bevy::prelude::*;

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

struct Entity(u64);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // .add_systems(Startup, add_people)
        // .add_systems(Update, (hello_world, greet_people))
        .run();
}

fn print_position_system(query: Query<&Position>) {
    for position in &query {
        println!("position: {} {}", position.x, position.y);
    }
}
