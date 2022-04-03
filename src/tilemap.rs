use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
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
    tile_map: Res<SpriteSheet>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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
            tiled::LayerType::ObjectLayer(o_layer) => {
                println!("Object layer {} with {} objects", layer.name, o_layer.objects().len());
                for obj in o_layer.objects() {
                    println!("\tobj {}: {}", obj.name, obj.obj_type);
                    //TODO: create re-usable functions for coord transforms
                    let x = obj.x / map.tile_width as f32 * TILE_SIZE;
                    let y = 1.0 - obj.y / map.tile_height as f32 * TILE_SIZE;

                    match &obj.shape {
                        tiled::ObjectShape::Point(px, py) => {
                            println!("\tpoint {},{} {},{}", x, y, px, py)
                        },
                        tiled::ObjectShape::Rect { width, height } => {
                            println!("\trect {},{} {} x {}", x, y, width, height)
                        },
                        tiled::ObjectShape::Ellipse { width, height } => {
                            println!("\telipse {},{} {} x {}", x, y, width, height)
                        },
                        tiled::ObjectShape::Polyline { points } => {
                            println!("\tpolyLine {},{} \n\t\t{:?}", x, y, points)
                        },
                        tiled::ObjectShape::Polygon { points } => {
                            println!("\tpolygon {},{} \n\t\t{:?}", x, y, points)
                        }
                    }
                    //TODO: https://stackoverflow.com/questions/63643682/bevy-how-to-render-triangle-or-polygon-in-2d
                    // convert above shapes to meshes
                    // there isn't any 2d canvas support built into bevy, maybe consider pulling in a library in the mean time
                    // after all, this is just debug stuff

                    commands.spawn_bundle(MaterialMesh2dBundle {
                        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
                        transform: Transform{
                            translation: Vec3::new(x, y, z),
                            ..Default::default()
                        },
                        material: materials.add(ColorMaterial::from(Color::GREEN)),
                        global_transform: GlobalTransform::default(),
                        visibility: Visibility::default(),
                        computed_visibility: ComputedVisibility::default(),
                    }).insert(Name::new(obj.name.to_owned()));
                }
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