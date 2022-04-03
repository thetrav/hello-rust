use bevy::prelude::*;
use tiled::Loader;

pub struct TileMapPlugin;

impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut App) {
        
    }
}

fn load_tilemap {
    let mut loader = Loader::new();
    let map = loader.load_tmx_map("assets/test.tmx").unwrap();
    println!("{:?}", map);
    println!("{:?}", map.tilesets()[0].get_tile(0).unwrap().probability);
    
    let tileset = loader.load_tsx_tileset("assets/urban_prg.tsx").unwrap();
    assert_eq!(*map.tilesets()[0], tileset);
}