use bevy::prelude::*;

use crate::components::{RockAtlas, RockRenderPiece};
use crate::constants::TILE_SIZE;
use crate::original::map::{MapObjectType, ZMap};
use crate::original::objects::ItemType;
use crate::original::types::PlanetType;

#[derive(Clone, Copy)]
struct RockRender {
    top: [Option<usize>; 3],
    shadow: [Option<usize>; 3],
}

pub(crate) fn spawn_rocks(commands: &mut Commands, map: &ZMap, atlas: &RockAtlas) {
    let map_w = map.basics.width as usize;
    let map_h = map.basics.height as usize;
    let mut rock_list = vec![vec![false; map_h]; map_w];
    let mut rock_coords = Vec::new();

    for object in &map.objects {
        if object.object_type != MapObjectType::MapItem || object.object_id != ItemType::Rock as u8
        {
            continue;
        }

        let tx = object.x as usize;
        let ty = object.y as usize;
        if tx < map_w && ty < map_h {
            rock_list[tx][ty] = true;
            rock_coords.push((object.x, object.y));
        }
    }

    let image = match map.basics.terrain_type {
        PlanetType::Desert => atlas.desert.clone(),
        PlanetType::Volcanic => atlas.volcanic.clone(),
        PlanetType::Arctic => atlas.arctic.clone(),
        PlanetType::Jungle => atlas.jungle.clone(),
        PlanetType::City => atlas.city.clone(),
    };

    let mut pieces = 0;
    for &(tx, ty) in &rock_coords {
        let render = rock_render_for(&rock_list, tx as usize, ty as usize, map_w, map_h);
        for (j, maybe_index) in render.shadow.into_iter().enumerate() {
            if let Some(index) = maybe_index {
                spawn_rock_piece(
                    commands,
                    image.clone(),
                    atlas.layout.clone(),
                    tx,
                    ty,
                    tx + 1,
                    ty + j as u16,
                    index,
                    3.0,
                    "rock_shadow",
                );
                pieces += 1;
            }
        }

        for (j, maybe_index) in render.top.into_iter().enumerate() {
            if let Some(index) = maybe_index {
                spawn_rock_piece(
                    commands,
                    image.clone(),
                    atlas.layout.clone(),
                    tx,
                    ty,
                    tx,
                    ty + j as u16,
                    index,
                    4.0 + j as f32 * 0.01,
                    "rock",
                );
                pieces += 1;
            }
        }
    }

    println!(
        "Rock render: {} rocks, {pieces} atlas pieces",
        rock_coords.len()
    );
}

fn spawn_rock_piece(
    commands: &mut Commands,
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    rock_x: u16,
    rock_y: u16,
    tile_x: u16,
    tile_y: u16,
    index: usize,
    z: f32,
    name: &'static str,
) {
    commands.spawn((
        Sprite {
            image,
            texture_atlas: Some(TextureAtlas { layout, index }),
            ..default()
        },
        Transform::from_xyz(
            tile_x as f32 * TILE_SIZE + TILE_SIZE * 0.5,
            -(tile_y as f32 * TILE_SIZE + TILE_SIZE * 0.5),
            z,
        ),
        RockRenderPiece {
            x: rock_x,
            y: rock_y,
        },
        Name::new(name),
    ));
}

fn rock_render_for(
    rock_list: &[Vec<bool>],
    tx: usize,
    ty: usize,
    map_w: usize,
    map_h: usize,
) -> RockRender {
    let l = tx == 0 || rock_list[tx - 1][ty];
    let up = ty == 0 || rock_list[tx][ty - 1];
    let r = tx == map_w - 1 || rock_list[tx + 1][ty];
    let dn = ty == map_h - 1 || rock_list[tx][ty + 1];

    let dl = tx > 0 && ty < map_h - 1 && rock_list[tx - 1][ty + 1];
    let ddn = ty >= map_h - 2 || rock_list[tx][ty + 2];
    let uup = ty < 2 || rock_list[tx][ty - 2];
    let uur = ty >= 2 && tx < map_w - 1 && rock_list[tx + 1][ty - 2];
    let ur = ty >= 1 && tx < map_w - 1 && rock_list[tx + 1][ty - 1];
    let dr = ty < map_h - 1 && tx < map_w - 1 && rock_list[tx + 1][ty + 1];
    let ddr = ty < map_h - 2 && tx < map_w - 1 && rock_list[tx + 1][ty + 2];

    let mut top = [None, None, Some(rock_index(3, 4))];
    let mut shadow = [None, None, None];

    top[0] = Some(if r && l && up && dn {
        rock_index(1, 1)
    } else if r && !l && !up && dn {
        rock_index(0, 0)
    } else if !r && l && !up && dn {
        rock_index(2, 0)
    } else if !r && l && up && !dn {
        rock_index(2, 2)
    } else if r && !l && up && !dn {
        rock_index(0, 2)
    } else if r && l && !up && dn {
        rock_index(1, 0)
    } else if r && l && up && !dn {
        rock_index(1, 2)
    } else if !r && l && up && dn {
        rock_index(2, 1)
    } else if r && !l && up && dn {
        rock_index(0, 1)
    } else if !r && !l && !up && dn {
        rock_index(3, 0)
    } else if !r && !l && up && dn {
        rock_index(3, 1)
    } else if !r && !l && up && !dn {
        rock_index(3, 2)
    } else if r && !l && !up && !dn {
        rock_index(0, 5)
    } else if r && l && !up && !dn {
        rock_index(1, 5)
    } else if !r && l && !up && !dn {
        rock_index(2, 5)
    } else {
        rock_index(3, 2)
    });

    if ty + 1 < map_h {
        top[1] = if dn {
            None
        } else if dl {
            Some(if r {
                rock_index(5, 0)
            } else {
                rock_index(4, 0)
            })
        } else {
            Some(if r && l {
                rock_index(1, 3)
            } else if r && !l {
                rock_index(0, 3)
            } else if !r && l {
                rock_index(2, 3)
            } else {
                rock_index(3, 3)
            })
        };
    }

    if ty + 2 < map_h {
        top[2] = if ddn || dn {
            None
        } else if dl {
            Some(if r {
                rock_index(5, 1)
            } else {
                rock_index(4, 1)
            })
        } else {
            Some(if r && l {
                rock_index(1, 4)
            } else if r && !l {
                rock_index(0, 4)
            } else if !r && l {
                rock_index(2, 4)
            } else {
                rock_index(3, 4)
            })
        };
    }

    if tx < map_w - 1 {
        if !(uur || ur || r) {
            if up && !uup {
                shadow[0] = Some(rock_index(4, 2));
            } else if up || uup {
                shadow[0] = Some(rock_index(4, 3));
            }
        }

        if !dn && !(ur || r || dr) {
            if !up {
                shadow[1] = Some(rock_index(4, 2));
            } else {
                shadow[1] = Some(rock_index(4, 3));
            }
        }

        if !dn && !ddn && !(r || dr || ddr) {
            shadow[2] = Some(rock_index(4, 4));
        }
    }

    RockRender { top, shadow }
}

fn rock_index(x: usize, y: usize) -> usize {
    y * 6 + x
}
