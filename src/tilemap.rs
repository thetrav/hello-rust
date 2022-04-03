use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use tiled::Loader;


use crate::{spritesheet::{SpriteSheet, spawn_sprite}, TILE_SIZE};

pub struct TileMapPlugin;

#[derive(Component, Inspectable)]
pub struct TileMapLayer {
    z:f32,
}


impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_tilemap);
    }
}

fn load_tilemap(
    mut commands: Commands, 
    tile_map: Res<SpriteSheet>
) {
    let mut loader = Loader::new();
    let map = loader.load_tmx_map("assets/test.tmx").unwrap();
    let mut z = 0.0;
    let mut layers = Vec::new();
    for layer in map.layers() {
        z += 100.0;
        let mut tiles = Vec::new();
        match layer.layer_type() {
            tiled::LayerType::TileLayer(layer) => {
                match layer {
                    tiled::TileLayer::Finite(data) => {
                        for y in 0..(data.height()-1) {
                            for x in 0..(data.width()-1) {
                                let tx = x as f32 * TILE_SIZE;
                                let ty = 1.0 - (y as f32 * TILE_SIZE);
                                data.get_tile(x as i32, y as i32).map(|tile_index| {
                                    let tile = spawn_sprite(
                                        &mut commands, 
                                        &tile_map, 
                                        tile_index.id().try_into().unwrap(), 
                                        Vec3::new(tx,ty,z)
                                    );
                                    tiles.push(tile)
                                });
                            }
                        }
                    }
                    tiled::TileLayer::Infinite(data) => {
                        println!(
                            "Infinite tile layer; Tile @ (-5, 0) = {:?}",
                            data.get_tile(0, 0)
                        )
                    }
                }
            }
            tiled::LayerType::ObjectLayer(layer) => {
                println!("Object layer with {} objects", layer.objects().len())
            }
            tiled::LayerType::ImageLayer(layer) => {
                println!(
                    "Image layer with {}",
                    match &layer.image {
                        Some(img) =>
                            format!("an image with source = {}", img.source.to_string_lossy()),
                        None => "no image".to_owned(),
                    }
                )
            }
            tiled::LayerType::GroupLayer(layer) => {
                println!("Group layer with {} sublayers", layer.layers().len())
            }
        }
        let layer = commands.spawn()
            .insert(Name::new(layer.name.to_owned()))
            .insert(Transform::default())
            .insert(GlobalTransform::default())
            .insert(TileMapLayer{z})
            .push_children(&tiles)
            .id();
        layers.push(layer);
    }

    commands.spawn()
        .insert(Name::new("TileMap"))
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .push_children(&layers);
}