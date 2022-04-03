use bevy::prelude::*;
use tiled::Loader;


use crate::{spritesheet::{SpriteSheet, spawn_sprite}, TILE_SIZE};

pub struct TileMapPlugin;

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
    for layer in map.layers() {
        z += 100.0;
        print!("Layer \"{}\":\n\t", layer.name);

        match layer.layer_type() {
            tiled::LayerType::TileLayer(layer) => {
                match layer {
                    tiled::TileLayer::Finite(data) => {
                        println!(
                            "Finite tile layer with width = {} and height = {}",
                            data.width(),
                            data.height(),
                        );
                        for y in 0..(data.height()-1) {
                            for x in 0..(data.width()-1) {
                                let tx = x as f32 * TILE_SIZE + TILE_SIZE/2.0;
                                let ty = 1.0 - (y as f32 * TILE_SIZE) + TILE_SIZE/2.0;
                                data.get_tile(x as i32, y as i32).map(|tile_index| spawn_sprite(
                                    &mut commands, 
                                    &tile_map, 
                                    tile_index.id().try_into().unwrap(), 
                                    Vec3::new(tx,ty,z)
                                ));
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
    }
}