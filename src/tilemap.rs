use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_inspector_egui::Inspectable;
use tiled::*;
use bevy_earcutr::*;

use crate::{spritesheet::{SpriteSheet, spawn_sprite}, TILE_SIZE};

pub struct TileMapPlugin;

#[derive(Debug)]
enum LoadedLayer {
    SpriteLayer(String, Vec3, Vec<SpriteParams>),
    MeshLayer(String, Vec3, Vec<MeshParams>)
    // GroupLayer(String, Vec3, Vec<LoadedLayer>),
    // Ignored
}

#[derive(Debug)]
struct SpriteParams {
    name: String, 
    index: usize, 
    offset: Vec3
}

#[derive(Debug, Component, Inspectable)]
pub struct MeshParams {
    name: String, 
    offset: Vec3,
    vertices: Vec<f64>
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
    let px = PixelTransformer{
        width: map.tile_width as f32, 
        height: map.tile_height as f32
    };

    let layers = load_layers(map.layers(), px, 0.0);
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
            },
            LoadedLayer::MeshLayer(name, offset, mesh_params) => {
                let layer_entity = commands.spawn()
                    .insert(Name::new(name))
                    .insert(Transform{
                        translation: offset,
                        ..Default::default()
                    })
                    .insert(GlobalTransform::default()).id();
                for params in mesh_params {
                    let mut builder = PolygonMeshBuilder::new();

                    builder.add_earcutr_input(EarcutrInput {
                        vertices: params.vertices,
                        interior_indices: vec![]
                    });
                
                    let mesh = builder.build().unwrap();
                    
                    let mesh_entity = commands.spawn_bundle(MaterialMesh2dBundle {
                        mesh: meshes.add(mesh).into(),
                        transform: Transform{
                            translation: params.offset,
                            ..Default::default()
                        },
                        material: materials.add(ColorMaterial::from(bevy::prelude::Color::GREEN)),
                        global_transform: GlobalTransform::default(),
                        visibility: Visibility::default(),
                        computed_visibility: ComputedVisibility::default(),
                    })
                    .insert(Name::new(params.name))
                    .id();
                    commands.entity(layer_entity).add_child(mesh_entity);
                }
            }
        }
    }
}

fn load_layers<'a>(layers: impl Iterator<Item = Layer<'a>>, px: PixelTransformer, z: f32) -> Vec<LoadedLayer>{
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
                let params = object_layer(data, px);
                loaded_layers.push(LoadedLayer::MeshLayer(name, offset, params));
            },
            _ => {
                println!("Unimplemented layer ignored: {}", layer.name);
            }
        }
    }
    return loaded_layers;
}

#[derive(Copy, Clone)]
struct PixelTransformer {
    width: f32,
    height: f32
}

impl PixelTransformer {
    fn from_pixels(self: &Self, x:f32, y:f32) -> (f32, f32) {
        let tx = (x / self.width as f32) * TILE_SIZE;
        let ty = 1.0 - (y / self.height as f32) * TILE_SIZE;
        return (tx, ty);
    }
}

fn object_layer(data: ObjectLayer, px: PixelTransformer) -> Vec<MeshParams> {
    let mut meshes = Vec::new();
    
    for obj in data.objects() {
        let (x,y) = &px.from_pixels(obj.x, obj.y);
        let offset = Vec3::new(*x, *y, 0.0);
        let name = obj.name.to_owned();
        let vertices = match &obj.shape {
            tiled::ObjectShape::Point(_, _) => 
                rect_mesh(0.0, 0.0, TILE_SIZE, TILE_SIZE),
            tiled::ObjectShape::Rect { width, height } => 
                rect_mesh(0.0,0.0, *width, *height),
            tiled::ObjectShape::Polygon { points } => 
                poly_mesh(px, points),
            // tiled::ObjectShape::Ellipse { width, height } => {
            //     println!("\telipse {},{} {} x {}", x, y, width, height)
            // },
            // tiled::ObjectShape::Polyline { points } => {
            //     println!("\tpolyLine {},{} \n\t\t{:?}", x, y, points)
            // },
            _ => vec![]
        };
        meshes.push(MeshParams{name, offset, vertices});
    }
    return meshes;
}

fn poly_mesh(px: PixelTransformer, points: &[(f32, f32)]) -> Vec<f64> {
    let mut vertices = Vec::new();
    for p in points {
        let (x, y) = px.from_pixels(p.0, p.1);
        vertices.push(x as f64);
        vertices.push(y as f64);
    }
    return vertices;
}

fn rect_mesh(x: f32, y: f32, width: f32, height: f32) -> Vec<f64> {
    let x = x as f64;
    let y = y as f64;
    let w = (width / 2.0) as f64;
    let h = (height / 2.0) as f64;
    return vec![
        x-w, y-h,
        x+w, y-h,
        x+w, y+h,
        x-w, y+h
    ];
} 

fn finite_tile_layer(data: FiniteTileLayer) -> Vec<SpriteParams> {
    let mut tiles = Vec::new();
    for y in 0..(data.height()-1) {
        for x in 0..(data.width()-1) {
            let tx = x as f32 * TILE_SIZE;
            let ty = 1.0 - (y as f32 * TILE_SIZE);
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