use bevy::prelude::*;

use crate::TILE_SIZE;

pub struct SpriteSheetPlugin;

pub struct SpriteSheet(Handle<TextureAtlas>);

impl Plugin for SpriteSheetPlugin {
    fn build(&self, app:&mut App) {
        app.add_startup_system_to_stage(
            StartupStage::PreStartup,
            load_sprite_sheet);
    }
}

pub(crate) fn spawn_sprite(
    commands: &mut Commands,
    sprite_sheet: &SpriteSheet,
    index: usize,
    translation: Vec3
) -> Entity {
    let mut sprite = TextureAtlasSprite::new(index);
    sprite.custom_size = Some(Vec2::splat(TILE_SIZE));

    return commands.spawn_bundle(SpriteSheetBundle {
        texture_atlas: sprite_sheet.0.clone(),
        sprite: sprite,
        transform: Transform{
            translation: translation,
            ..Default::default()
        },
        ..Default::default()
    }).id();
}

fn load_sprite_sheet(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlasses: ResMut<Assets<TextureAtlas>>
) {
    let image:Handle<Image> = assets.load("sprite_sheet.png");
    let atlas = TextureAtlas::from_grid_with_padding(
        image,
        Vec2::splat(16.0),
        27,
        18,
        Vec2::splat(1.0)
    );
    let atlas_handle = texture_atlasses.add(atlas);
    commands.insert_resource(SpriteSheet(atlas_handle));
}