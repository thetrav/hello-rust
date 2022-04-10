use bevy::prelude::*;
use tiled::*;
use bevy::render::color::Color;

use crate::{spritesheet::{SpriteSheet, spawn_sprite}, TILE_SIZE};

pub struct TileMapPlugin;

#[derive(Debug)]
enum LoadedLayer {
    SpriteLayer(String, Vec3, Vec<SpriteParams>),
    ObjectLayer(String, Vec3, Vec<ObjectParams>)
    // GroupLayer(String, Vec3, Vec<LoadedLayer>),
    // Ignored
}

#[derive(Debug)]
struct ObjectParams {
    name: String, 
    offset: Vec3,
    points: Vec<Vec2>,
    point_size: Vec2,
    color: Color
}

#[derive(Debug)]
struct SpriteParams {
    index: usize, 
    offset: Vec3,
    name: String
}

impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(load_tilemap);
    }
}


fn load_tilemap(
    mut commands: Commands, 
    tile_map: Res<SpriteSheet>,
) {
    let mut loader = Loader::new();
    let map = loader.load_tmx_map("assets/test.tmx").unwrap();

    let map_entity = commands.spawn()
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .insert(Name::new("assets/text.tmx"))
        .id();

    let layers = load_layers(map.layers(), 0.0);
    for layer in layers {
        match layer {
            LoadedLayer::SpriteLayer(name, offset, sprite_params) => {
                let layer_entity = commands.spawn()
                    .insert(Name::new(name))
                    .insert(Transform{
                        translation: offset,
                        ..Default::default()
                    })
                    .insert(GlobalTransform::default()).id();
                for params in sprite_params {
                    let sprite = spawn_sprite(
                        &mut commands, 
                        &tile_map, 
                        params.index, 
                        params.offset
                    );
                    let named_sprite = commands.entity(sprite)
                        .insert(Name::new(params.name))
                        .id();
                    commands.entity(layer_entity).add_child(named_sprite);
                }
                commands.entity(map_entity).add_child(layer_entity);
            },
            LoadedLayer::ObjectLayer(name, offset, objects) => {
                let layer_entity = commands.spawn()
                    .insert(Name::new(name))
                    .insert(Transform{
                        translation: Vec3::new(offset.x - TILE_SIZE/2.0, offset.y + TILE_SIZE/2.0, offset.z),
                        ..Default::default()
                    })
                    .insert(GlobalTransform::default()).id();
                for obj in objects {
                    let obj_ent = commands.spawn()
                        .insert(GlobalTransform::default())
                        .insert(Transform{
                            translation: obj.offset,
                            ..Default::default()
                        })
                        .insert(Name::new(obj.name.to_owned()))
                        .id();
                    for p in obj.points {
                        let drawing = commands
                            .spawn_bundle(SpriteBundle {
                                sprite: Sprite {
                                    color: obj.color,
                                    ..Default::default()
                                },
                                transform: Transform {
                                    translation: Vec3::new(p.x, p.y, 0.0),
                                    scale: Vec3::new(obj.point_size.x, obj.point_size.y, 1.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .id();
                        commands.entity(obj_ent).add_child(drawing);
                    }
                    commands.entity(layer_entity).add_child(obj_ent);
                }
                commands.entity(map_entity).add_child(layer_entity);
            }
        }
    }
}

fn load_layers<'a>(layers: impl Iterator<Item = Layer<'a>>, z: f32) -> Vec<LoadedLayer>{
    let mut loaded_layers = Vec::new();
    let mut lz = z;
    for layer in layers {
        lz += 100.0;
        let name = layer.name.to_owned();
        let offset = Vec3::new(layer.offset_x, layer.offset_y, lz);
        match layer.layer_type() {
            LayerType::TileLayer(TileLayer::Finite(data)) => {
                let params = finite_tile_layer(data);
                loaded_layers.push(LoadedLayer::SpriteLayer(name, offset, params));
            },
            LayerType::ObjectLayer(data) => {
                let params = object_layer(data);
                loaded_layers.push(LoadedLayer::ObjectLayer(name, offset, params));
            },
            _ => {
                println!("Unimplemented layer ignored: {}", layer.name);
            }
        }
    }
    return loaded_layers;
}

fn to_vec(points: &Vec<(f32, f32)>) -> Vec<Vec2> {
    let mut mapped = Vec::new();
    for point in points {
        mapped.push(Vec2::new(point.0, point.1*-1.0));
    }
    return mapped;
}

fn object_layer(data: ObjectLayer) -> Vec<ObjectParams> {
    let mut meshes = Vec::new();
    
    for obj in data.objects() {
        let offset = Vec3::new(obj.x, obj.y*-1.0, 0.0);
        let name = obj.name.to_owned();
        let (points, size, color) = match &obj.shape {
            tiled::ObjectShape::Point(_,_) => 
                (vec![Vec2::new(0.0, 0.0)], Vec2::new(TILE_SIZE, TILE_SIZE), Color::rgba(0.0,1.0,0.0,0.8)),
            tiled::ObjectShape::Rect { width, height } => 
                (vec![Vec2::new(0.0, 0.0)], Vec2::new(*width, *height), Color::rgba(1.0,0.0,0.0,0.8)),
            tiled::ObjectShape::Polygon { points } => 
                (to_vec(points), Vec2::new(TILE_SIZE/4.0, TILE_SIZE/4.0), Color::rgba(1.0, 1.0, 0.0, 0.8)),
            tiled::ObjectShape::Ellipse { width, height } => 
                (vec![Vec2::new(0.0, 0.0)], Vec2::new(*width, *height), Color::rgba(0.0,0.0,1.0,0.8)),
            // tiled::ObjectShape::Polyline { points } => {
            //     println!("\tpolyLine {},{} \n\t\t{:?}", x, y, points)
            // },
            _ => (vec![], Vec2::new(0.0,0.0), Color::BLACK)
        };
        meshes.push(ObjectParams{name, offset, points, point_size: size, color});
    }
    return meshes;
}

fn finite_tile_layer(data: FiniteTileLayer) -> Vec<SpriteParams> {
    let mut tiles = Vec::new();
    for y in 0..(data.height()) {
        for x in 0..(data.width()) {
            let tx = x as f32 * TILE_SIZE;
            let ty = y as f32 * -TILE_SIZE;
            data.get_tile(x as i32, y as i32).map(|tile_index| {
                tiles.push(SpriteParams{
                    name: format!("{},{}", x, y),
                    index: tile_index.id().try_into().unwrap(), 
                    offset: Vec3::new(tx, ty, 0.0)
                });
            });
        }
    }
    return tiles;
}