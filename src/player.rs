use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

use crate::{spritesheet::{SpriteSheet, spawn_sprite}, TILE_SIZE};

pub struct PlayerPlugin;

#[derive(Component, Inspectable)]
pub struct Player {
    speed: f32
}

impl Plugin for PlayerPlugin {
    fn build(&self, app:&mut App) {
        app
            .add_startup_system(spawn_player)
            .add_system(player_movement.label("movement"))
            .add_system(camera_follow.after("movement"));
    }
}

fn camera_follow(player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (Without<Player>, With<Camera>)>) {
        let player_transform = player_query.single();
        let mut camera_transform = camera_query.single_mut();
        camera_transform.translation.x = player_transform.translation.x;
        camera_transform.translation.y = player_transform.translation.y;
    }

fn player_movement(
    mut player_query: Query<(&Player, &mut Transform)>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (player, mut transform) = player_query.single_mut();
    if keyboard.pressed(KeyCode::W) {
        transform.translation.y += player.speed * TILE_SIZE * time.delta_seconds();
    }
    if keyboard.pressed(KeyCode::S) {
        transform.translation.y -= player.speed * TILE_SIZE * time.delta_seconds();
    }
    if keyboard.pressed(KeyCode::A) {
        transform.translation.x -= player.speed * TILE_SIZE * time.delta_seconds();
    }
    if keyboard.pressed(KeyCode::D) {
        transform.translation.x += player.speed * TILE_SIZE * time.delta_seconds();
    }
}

fn spawn_player(mut commands: Commands, tile_map: Res<SpriteSheet>) {
    let player = spawn_sprite(
        &mut commands, 
        &tile_map, 
        24, 
        Vec3::new(8.0,-1.0,900.0));
    
    commands.entity(player)
        .insert(Name::new("Player"))
        .insert(Player{speed: 3.0}).id();
}