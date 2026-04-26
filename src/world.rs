use bevy::prelude::*;

pub const TILE_SIZE: f32 = 40.0;
const TILE_SOURCE: u32 = 16;

// Tile IDs
const G:  u8 = 0;  // grass
const D:  u8 = 1;  // dirt (unused but available)
const PH: u8 = 2;  // path horizontal
const PV: u8 = 3;  // path vertical
const PC: u8 = 4;  // path cross
const WF: u8 = 5;  // wall front
const WS: u8 = 6;  // wall side
const RF: u8 = 7;  // roof
const DR: u8 = 8;  // door
const WN: u8 = 9;  // window
const TR: u8 = 14; // tree
const FL: u8 = 15; // flower

pub const MAP_W: usize = 40;
pub const MAP_H: usize = 37;

// Layer definitions for z-indexing
pub const LAYER_BACKGROUND: f32 = 0.0;
pub const LAYER_PLAYER: f32 = 1.0;
pub const LAYER_FOREGROUND: f32 = 2.0;

// Collision data - which tiles block movement
pub fn is_solid_tile(tile_id: u8) -> bool {
    matches!(tile_id, WF | WS | TR | RF)
}

/// Converts a world position to the mapped tile under it, if any.
pub fn world_tile_at_position(position: Vec3) -> Option<u8> {
    let tile_x = ((position.x + (MAP_W as f32 * TILE_SIZE) / 2.0) / TILE_SIZE).floor() as i32;
    let tile_y = (((MAP_H as f32 * TILE_SIZE) / 2.0 - position.y) / TILE_SIZE).floor() as i32;

    if tile_x < 0 || tile_x >= MAP_W as i32 || tile_y < 0 || tile_y >= MAP_H as i32 {
        return None;
    }

    Some(MAP[tile_y as usize][tile_x as usize])
}

/// Returns true when the supplied world position is not inside a solid tile.
pub fn is_walkable_position(position: Vec3) -> bool {
    world_tile_at_position(position)
        .map_or(false, |tile_id| !is_solid_tile(tile_id))
}

// Get the appropriate layer for a tile
pub fn get_tile_layer(tile_id: u8) -> f32 {
    match tile_id {
        WF | WS | RF | DR | WN => LAYER_FOREGROUND, // Walls, roofs, doors, windows in front
        TR => LAYER_FOREGROUND, // Trees in front
        _ => LAYER_BACKGROUND,   // Everything else in background
    }
}

#[rustfmt::skip]
pub const MAP: [[u8; MAP_W]; MAP_H] = [
    // row 0 — top tree border
    [TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR],
    // rows 1-3 — extra grass buffer
    [TR, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,TR],
    [TR, G, G,FL, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G, G, G, G, G,FL, G, G, G, G, G, G,TR],
    [TR, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,TR],
    // row 4 — buildings row 1
    [TR, G,RF,RF,RF,RF, G, G,RF,RF,RF,RF, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G,RF,RF,RF,RF, G, G,RF,RF,RF,RF, G, G, G,TR],
    [TR, G,WS,WF,WF,WS, G, G,WS,WF,WF,WS, G,FL, G,FL, G, G, G,PV,PV, G, G,FL, G, G,WS,WF,WF,WS, G, G,WS,WF,WF,WS, G, G, G,TR],
    [TR, G,WS,DR,WN,WS, G, G,WS,DR,WN,WS, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G,WS,DR,WN,WS, G, G,WS,DR,WN,WS, G, G, G,TR],
    [TR, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,TR],
    [TR, G, G,FL, G, G, G,FL, G, G, G, G, G, G,TR, G, G, G, G,PV,PV, G, G, G,TR, G, G,FL, G, G, G, G, G,TR, G,FL, G, G, G,TR],
    [TR, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,TR],
    [TR, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,TR],
    // rows 11-12 — horizontal road
    [PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PC,PC,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH],
    [PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PC,PC,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH],
    // rows 13-18 — middle section
    [TR, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,TR],
    [TR, G, G, G, G, G, G, G, G,FL, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G,FL, G, G, G, G, G, G, G, G, G, G,TR],
    [TR, G,RF,RF,RF,RF,RF, G, G, G, G, G, G, G,TR, G, G, G, G,PV,PV, G, G, G, G,TR, G, G, G, G, G,RF,RF,RF,RF,RF, G, G, G,TR],
    [TR, G,WS,WF,WF,WF,WS, G, G, G, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G, G, G, G,WS,WF,WF,WF,WS, G, G, G,TR],
    [TR, G,WS,WN,DR,WN,WS, G, G, G,FL, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G,FL, G, G,WS,WN,DR,WN,WS, G, G, G,TR],
    [TR, G,WS,WF,WF,WF,WS, G, G, G, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G, G, G, G,WS,WF,WF,WF,WS, G, G, G,TR],
    [TR, G, G, G, G, G, G, G, G, G, G, G,TR, G, G, G, G, G, G,PV,PV, G, G, G, G, G,TR, G, G, G, G, G, G, G, G, G, G, G, G,TR],
    [TR, G, G,FL, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G, G, G, G, G,FL, G, G, G, G, G, G,TR],
    [TR, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,TR],
    // rows 22-23 — second horizontal road
    [PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PC,PC,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH],
    [PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PC,PC,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH,PH],
    // rows 24-29 — lower section with buildings
    [TR, G, G, G, G, G, G, G, G, G, G,TR, G, G, G,FL, G, G, G,PV,PV, G, G, G,FL, G, G,TR, G, G, G, G, G, G, G, G, G, G, G,TR],
    [TR, G,RF,RF,RF,RF, G, G, G, G, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G, G, G, G,RF,RF,RF,RF, G, G, G, G,TR],
    [TR, G,WS,WF,WF,WS, G, G, G, G, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G, G, G, G,WS,WF,WF,WS, G, G, G, G,TR],
    [TR, G,WS,DR,WN,WS, G, G, G,FL, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G,FL, G, G, G,WS,DR,WN,WS, G, G, G, G,TR],
    [TR, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,TR],
    // rows 30-32 — extra grass buffer before bottom border
    [TR, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,TR],
    [TR, G, G, G, G,FL, G, G, G, G, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G, G, G, G, G, G,FL, G, G, G, G, G,TR],
    [TR, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,TR],
    [TR, G, G, G, G, G, G, G,TR, G, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G,TR, G, G, G, G, G, G, G, G, G, G,TR],
    [TR, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,TR],
    [TR, G, G,FL, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G, G, G, G, G, G, G,FL, G, G, G, G,TR],
    [TR, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,PV,PV, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G, G,TR],
    // row 37 — bottom tree border
    [TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR,TR],
];

#[derive(Component)]
pub struct TileMap;

pub fn spawn_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("tileset.png");

    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(TILE_SOURCE, TILE_SOURCE),
        16,
        1,
        None,
        None,
    );
    let atlas_layout = texture_atlas_layouts.add(layout);

    let origin_x = -(MAP_W as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
    let origin_y =  (MAP_H as f32 * TILE_SIZE) / 2.0 - TILE_SIZE / 2.0;

    for row in 0..MAP_H {
        for col in 0..MAP_W {
            let tile_id = MAP[row][col] as usize;
            let x = origin_x + col as f32 * TILE_SIZE;
            let y = origin_y - row as f32 * TILE_SIZE;
            let z = get_tile_layer(MAP[row][col]);

            commands.spawn((
                Sprite {
                    image: texture.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: atlas_layout.clone(),
                        index: tile_id,
                    }),
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
                    ..default()
                },
                Transform::from_xyz(x, y, z),
                TileMap,
            ));
        }
    }
}