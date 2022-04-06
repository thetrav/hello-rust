use bevy::prelude::*;
use tiled::*;

use bevy_prototype_lyon::prelude::*;

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
    shape: ShapeType
}

#[derive(Debug)]
enum ShapeType {
    Poly(PolyParams),
    Ellipse(EllipseParams)
}

#[derive(Debug)]
struct SpriteParams {
    index: usize, 
    offset: Vec3,
    name: String
}

#[derive(Debug)]
struct PolyParams {
    vertices: Vec<Vec2>, 
}

#[derive(Debug)]
struct EllipseParams {
    radii: Vec2,
    offset: Vec2
}

impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(ShapePlugin)
            .add_startup_system(load_tilemap);
    }
}


fn load_tilemap(
    mut commands: Commands, 
    tile_map: Res<SpriteSheet>,
) {
    let mut loader = Loader::new();
    let map = loader.load_tmx_map("assets/test.tmx").unwrap();
    let px = PixelTransformer{
        width: map.tile_width as f32, 
        height: map.tile_height as f32
    };

    let map_entity = commands.spawn()
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .insert(Name::new("assets/text.tmx"))
        .id();

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
                commands.entity(map_entity).add_child(layer_entity);
            },
            LoadedLayer::ObjectLayer(name, offset, objects) => {
                let layer_entity = commands.spawn()
                    .insert(Name::new(name))
                    .insert(Transform{
                        translation: offset,
                        ..Default::default()
                    })
                    .insert(GlobalTransform::default()).id();
                for params in objects {
                    match params.shape {
                        ShapeType::Ellipse(shape_params) => {
                            let shape = shapes::Ellipse {
                                    radii: shape_params.radii,
                                    center: shape_params.offset
                            };
                            let drawing = commands.spawn_bundle(GeometryBuilder::build_as(
                                &shape,
                                DrawMode::Outlined {
                                    fill_mode: FillMode::color(bevy::render::color::Color::rgba(0.0,1.0,0.0,0.2)),
                                    outline_mode: StrokeMode::new(bevy::render::color::Color::GREEN, 10.0),
                                },
                                Transform{
                                    translation: params.offset,
                                    ..Default::default()
                                }
                            )).insert(Name::new(params.name))
                            .id();
                            
                            commands.entity(layer_entity).add_child(drawing);
                        },
                        ShapeType::Poly(shape_params) => {
                            let shape = shapes::Polygon {
                                points: shape_params.vertices,
                                closed: true
                            };
                            let drawing = commands.spawn_bundle(GeometryBuilder::build_as(
                                &shape,
                                DrawMode::Outlined {
                                    fill_mode: FillMode::color(bevy::render::color::Color::rgba(0.0,1.0,0.0,0.2)),
                                    outline_mode: StrokeMode::new(bevy::render::color::Color::GREEN, 10.0),
                                },
                                Transform{
                                    translation: params.offset,
                                    ..Default::default()
                                }
                            )).insert(Name::new(params.name))
                            .id();
                            
                            commands.entity(layer_entity).add_child(drawing);
                        }
                    };
                }
                commands.entity(map_entity).add_child(layer_entity);
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
                loaded_layers.push(LoadedLayer::ObjectLayer(name, offset, params));
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

fn object_layer(data: ObjectLayer, px: PixelTransformer) -> Vec<ObjectParams> {
    let mut meshes = Vec::new();
    
    for obj in data.objects() {
        let (x,y) = &px.from_pixels(obj.x, obj.y);
        let offset = Vec3::new(*x, *y, 0.0);
        let name = obj.name.to_owned();
        let shape = match &obj.shape {
            tiled::ObjectShape::Point(_, _) => 
                ellipsis_shape(0.0, 0.0, TILE_SIZE, TILE_SIZE),
            tiled::ObjectShape::Rect { width, height } => 
                rect_shape(0.0,0.0, *width, *height),
            tiled::ObjectShape::Polygon { points } => 
                poly_shape(px, points),
            tiled::ObjectShape::Ellipse { width, height } => {
                ellipsis_shape(0.0,0.0, *width, *height)
            },
            // tiled::ObjectShape::Polyline { points } => {
            //     println!("\tpolyLine {},{} \n\t\t{:?}", x, y, points)
            // },
            _ => ellipsis_shape(0.0, 0.0, 0.0, 0.0),
        };
        meshes.push(ObjectParams{name, offset, shape});
    }
    return meshes;
}

fn poly_shape(px: PixelTransformer, points: &[(f32, f32)]) -> ShapeType {
    let mut vertices = Vec::new();
    for p in points {
        let (x, y) = px.from_pixels(p.0, p.1);
        vertices.push(Vec2::new(x, y));
    }
    return ShapeType::Poly(PolyParams{vertices});
}

fn ellipsis_shape(x: f32, y: f32, width: f32, height: f32) -> ShapeType {
    return ShapeType::Ellipse(EllipseParams{
        radii: Vec2::new(width, height),
        offset: Vec2::new(x, y)
    });
} 

fn rect_shape(x: f32, y: f32, width: f32, height: f32) -> ShapeType {
    let w = width / 2.0;
    let h = height / 2.0;
    return ShapeType::Poly(PolyParams{vertices: vec![
        Vec2::new(x-w, y-h),
        Vec2::new(x+w, y-h),
        Vec2::new(x+w, y+h),
        Vec2::new(x-w, y+h)
    ]});
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