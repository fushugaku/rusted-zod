use bevy::prelude::{Color, Vec2};

use crate::{
    components::{CombatRng, DamageCrater, DamageMissileVisual, Selectable},
    original::{
        map::MapObjectType,
        objects::{ItemType, ObjectKind},
        types::TeamType,
    },
    render::atlas::MobileSpriteRole,
    units::{
        self, DamageMissileImpactEffectProfile, DamageMissileLaunchEffectProfile,
        DamageMissileVisualGeometry, RocketImpactProfile, UnitImpactSound,
    },
};

pub(crate) fn combat_object_default_size(kind: ObjectKind) -> Vec2 {
    match kind {
        ObjectKind::Robot(robot) => units::robots::default_selection_size(robot),
        ObjectKind::Vehicle(vehicle) => units::vehicles::default_selection_size(vehicle),
        ObjectKind::Cannon(cannon) => units::cannons::default_selection_size(cannon),
        ObjectKind::Building(building) | ObjectKind::Bridge(building) => {
            units::buildings::default_selection_size(building)
        }
        ObjectKind::Rock => units::items::rock_ui::default_selection_size(),
        ObjectKind::Animal(_) => units::items::animal_ui::default_selection_size(),
        ObjectKind::MapItem(item_id) => units::items::default_selection_size(item_id),
    }
}

pub(crate) fn selected_hud_base_name(kind: ObjectKind) -> Option<&'static str> {
    match kind {
        ObjectKind::Robot(robot) => Some(units::robots::hud_name(robot)),
        ObjectKind::Vehicle(vehicle) => Some(units::vehicles::hud_name(vehicle)),
        ObjectKind::Cannon(cannon) => Some(units::cannons::hud_name(cannon)),
        _ => None,
    }
}

pub(crate) fn selected_hud_icon_asset_name(kind: ObjectKind, team: TeamType) -> Option<String> {
    let name = selected_hud_base_name(kind)?;
    Some(format!(
        "icon_{}_{}.png",
        name,
        team.atlas_team().asset_name()
    ))
}

pub(crate) fn selected_hud_label_asset_name(kind: ObjectKind) -> Option<String> {
    selected_hud_base_name(kind).map(|name| format!("label_{name}.png"))
}

pub(crate) fn queue_item_text(unit: ObjectKind) -> String {
    match unit {
        ObjectKind::Robot(robot) => format!("{robot:?}"),
        ObjectKind::Vehicle(vehicle) => format!("{vehicle:?}"),
        ObjectKind::Cannon(cannon) => format!("{cannon:?}"),
        ObjectKind::Building(building) => format!("{building:?}"),
        ObjectKind::Bridge(bridge) => format!("{bridge:?}"),
        ObjectKind::Animal(id) | ObjectKind::MapItem(id) => id.to_string(),
        ObjectKind::Rock => "Rock".to_string(),
    }
}

pub(crate) fn object_kind_to_map_parts(kind: ObjectKind) -> Option<(MapObjectType, u8)> {
    match kind {
        ObjectKind::Bridge(building) => Some((MapObjectType::Bridge, building as u8)),
        ObjectKind::Building(building) => Some((MapObjectType::Building, building as u8)),
        ObjectKind::Cannon(cannon) => Some((MapObjectType::Cannon, cannon as u8)),
        ObjectKind::Vehicle(vehicle) => Some((MapObjectType::Vehicle, vehicle as u8)),
        ObjectKind::Robot(robot) => Some((MapObjectType::Robot, robot as u8)),
        ObjectKind::Animal(id) => Some((MapObjectType::Animal, id)),
        ObjectKind::MapItem(id) => Some((MapObjectType::MapItem, id)),
        ObjectKind::Rock => Some((MapObjectType::MapItem, ItemType::Rock as u8)),
    }
}

pub(crate) fn mobile_frame_time(role: MobileSpriteRole) -> f32 {
    match role {
        MobileSpriteRole::Robot => units::robots::mobile_frame_time(),
        MobileSpriteRole::VehicleBase => units::vehicles::VEHICLE_BASE_FRAME_TIME,
        MobileSpriteRole::VehicleTop => 0.0,
    }
}

pub(crate) fn mobile_frame_count(kind: ObjectKind, role: MobileSpriteRole) -> usize {
    match (kind, role) {
        (ObjectKind::Robot(_), MobileSpriteRole::Robot) => units::robots::mobile_frame_count(),
        (ObjectKind::Vehicle(vehicle), role) => {
            units::vehicles::mobile_frame_count(vehicle, role).unwrap_or(1)
        }
        _ => 1,
    }
}

pub(crate) fn mobile_frame_delta_seconds(
    kind: ObjectKind,
    role: MobileSpriteRole,
    delta_secs: f32,
    speed_offset_percent: f32,
) -> f32 {
    match (kind, role) {
        (ObjectKind::Robot(_), MobileSpriteRole::Robot) => {
            units::robots::common_process_delta_seconds(delta_secs, speed_offset_percent)
        }
        _ => delta_secs.max(0.0),
    }
}

pub(crate) fn mobile_sprite_role(kind: ObjectKind, layer_index: usize) -> Option<MobileSpriteRole> {
    match kind {
        ObjectKind::Robot(_) => units::robots::mobile_sprite_role(layer_index),
        ObjectKind::Vehicle(_) => units::vehicles::mobile_sprite_role(layer_index),
        _ => None,
    }
}

pub(crate) fn selectable_for(kind: ObjectKind, selection_size: Vec2) -> Option<Selectable> {
    match kind {
        ObjectKind::Robot(_) => Some(Selectable {
            radius: 10.0,
            selection_size,
            mobile: true,
        }),
        ObjectKind::Vehicle(_) => {
            let profile = units::vehicles::selection_profile(selection_size);
            Some(Selectable {
                radius: profile.radius,
                selection_size: profile.selection_size,
                mobile: profile.mobile,
            })
        }
        ObjectKind::Cannon(_) => Some(Selectable {
            radius: 18.0,
            selection_size,
            mobile: false,
        }),
        ObjectKind::Building(_) | ObjectKind::Bridge(_) => Some(Selectable {
            radius: 42.0,
            selection_size,
            mobile: false,
        }),
        ObjectKind::Rock | ObjectKind::Animal(_) | ObjectKind::MapItem(_) => None,
    }
}

pub(crate) fn fallback_marker_size(kind: ObjectKind) -> Option<Vec2> {
    match kind {
        ObjectKind::Rock => None,
        ObjectKind::Building(_) | ObjectKind::Bridge(_) => Some(Vec2::splat(18.0)),
        ObjectKind::Vehicle(_) => Some(units::vehicles::fallback_marker_size()),
        ObjectKind::Robot(_) => Some(Vec2::splat(7.0)),
        ObjectKind::Cannon(_) => Some(Vec2::splat(12.0)),
        ObjectKind::MapItem(item_id) => {
            units::items::map_item_display_policy(item_id, TeamType::Null).fallback_marker_size
        }
        ObjectKind::Animal(_) => Some(Vec2::splat(6.0)),
    }
}

pub(crate) fn fallback_collision_size(kind: ObjectKind) -> Vec2 {
    match kind {
        ObjectKind::Building(building) | ObjectKind::Bridge(building) => {
            units::buildings::fallback_collision_size(building)
        }
        ObjectKind::Cannon(_) => units::cannons::fallback_collision_size(),
        ObjectKind::Rock | ObjectKind::MapItem(_) => {
            units::items::fallback_collision_size(kind).unwrap_or(Vec2::ZERO)
        }
        ObjectKind::Vehicle(_) | ObjectKind::Robot(_) | ObjectKind::Animal(_) => Vec2::ZERO,
    }
}

pub(crate) fn fallback_marker_color(kind: ObjectKind, owner: TeamType) -> Color {
    if owner != TeamType::Null {
        return owner.color();
    }

    match kind {
        ObjectKind::Building(_) | ObjectKind::Bridge(_) => Color::srgb(0.75, 0.75, 0.65),
        ObjectKind::MapItem(item_id) => {
            units::items::map_item_display_policy(item_id, owner).fallback_marker_color
        }
        ObjectKind::Rock => {
            units::items::map_item_display_policy(ItemType::Rock as u8, owner).fallback_marker_color
        }
        _ => Color::srgb(0.9, 0.9, 0.9),
    }
}

pub(crate) fn damage_missile_frame_paths(visual: DamageMissileVisual) -> Vec<String> {
    match visual {
        DamageMissileVisual::Generic => Vec::new(),
        DamageMissileVisual::Grenade => units::items::grenades_ui::projectile_frame_paths(),
        DamageMissileVisual::ToughRocket => units::robots::tough_ui::damage_missile_frame_paths(),
        DamageMissileVisual::LightRocket { .. } => {
            units::vehicles::light_ui::damage_missile_frame_paths()
        }
        DamageMissileVisual::MissileCannon => {
            units::cannons::missile_cannon_ui::damage_missile_frame_paths()
        }
        DamageMissileVisual::MissileLauncher => {
            units::vehicles::missile_launcher_ui::damage_missile_frame_paths()
        }
        DamageMissileVisual::MapObjectTurrent(object_i) => {
            units::items::map_object_ui::turrent_frame_paths(object_i)
        }
    }
}

#[cfg(test)]
pub(crate) fn light_rocket_init_fire_frame_path(frame: usize) -> String {
    units::vehicles::light_ui::init_fire_frame_path(frame)
}

pub(crate) fn damage_crater_for_visual(visual: DamageMissileVisual) -> Option<DamageCrater> {
    match visual {
        DamageMissileVisual::MissileLauncher => {
            Some(units::vehicles::missile_launcher_ui::damage_crater())
        }
        DamageMissileVisual::MissileCannon => {
            Some(units::cannons::missile_cannon_ui::damage_crater())
        }
        DamageMissileVisual::ToughRocket => Some(units::robots::tough_ui::damage_crater()),
        DamageMissileVisual::LightRocket {
            extra_large,
            xx_large,
            ..
        } => Some(units::vehicles::light_ui::damage_crater(
            extra_large,
            xx_large,
        )),
        _ => Some(DamageCrater {
            is_big: false,
            chance: 0.75,
            big_chance: None,
        }),
    }
}

pub(crate) fn damage_missile_launch_effect_profile(
    visual: DamageMissileVisual,
    rng: &mut CombatRng,
) -> Option<DamageMissileLaunchEffectProfile> {
    match visual {
        DamageMissileVisual::LightRocket { .. } => {
            Some(units::vehicles::light_ui::launch_effect_profile(
                rng.index(units::vehicles::light_ui::INIT_FIRE_FRAME_COUNT),
            ))
        }
        _ => None,
    }
}

pub(crate) fn damage_missile_visual_geometry(
    visual: DamageMissileVisual,
    start: Vec2,
    target: Vec2,
) -> DamageMissileVisualGeometry {
    let direction = missile_direction(start, target);
    match visual {
        DamageMissileVisual::ToughRocket => DamageMissileVisualGeometry {
            primary_offset: Vec2::ZERO,
            replica_offsets: Vec::new(),
            smoke_offsets: vec![Vec2::ZERO],
        },
        DamageMissileVisual::MissileCannon => {
            units::cannons::missile_cannon_ui::visual_geometry(direction)
        }
        DamageMissileVisual::MissileLauncher => {
            units::vehicles::missile_launcher_ui::visual_geometry(direction)
        }
        _ => DamageMissileVisualGeometry {
            primary_offset: Vec2::ZERO,
            replica_offsets: Vec::new(),
            smoke_offsets: Vec::new(),
        },
    }
}

pub(crate) fn damage_missile_rotates(visual: DamageMissileVisual) -> bool {
    matches!(
        visual,
        DamageMissileVisual::ToughRocket
            | DamageMissileVisual::LightRocket { .. }
            | DamageMissileVisual::MissileCannon
            | DamageMissileVisual::MissileLauncher
    )
}

pub(crate) fn damage_missile_muzzle_offset(
    visual: DamageMissileVisual,
    direction: usize,
) -> Option<Vec2> {
    match visual {
        DamageMissileVisual::ToughRocket => {
            Some(units::robots::tough_ui::rocket_muzzle_offset(direction))
        }
        DamageMissileVisual::LightRocket { .. }
        | DamageMissileVisual::MissileCannon
        | DamageMissileVisual::MissileLauncher => {
            Some(units::vehicles::rocket_muzzle_offset(direction))
        }
        _ => None,
    }
}

#[cfg(test)]
pub(crate) fn vehicle_rocket_muzzle_offset(direction: usize) -> Vec2 {
    units::vehicles::rocket_muzzle_offset(direction)
}

#[cfg(test)]
pub(crate) fn tough_rocket_muzzle_offset(direction: usize) -> Vec2 {
    units::robots::tough_ui::rocket_muzzle_offset(direction)
}

pub(crate) fn rocket_impact_profile(visual: DamageMissileVisual) -> Option<RocketImpactProfile> {
    match visual {
        DamageMissileVisual::LightRocket {
            extra_small,
            extra_large,
            xx_large,
        } => Some(units::vehicles::light_ui::rocket_impact_profile(
            extra_small,
            extra_large,
            xx_large,
        )),
        DamageMissileVisual::MissileCannon => {
            Some(units::cannons::missile_cannon_ui::rocket_impact_profile())
        }
        DamageMissileVisual::MissileLauncher => {
            Some(units::vehicles::missile_launcher_ui::rocket_impact_profile())
        }
        _ => None,
    }
}

pub(crate) fn damage_missile_impact_effect_profile(
    visual: DamageMissileVisual,
) -> DamageMissileImpactEffectProfile {
    match visual {
        DamageMissileVisual::LightRocket { .. }
        | DamageMissileVisual::MissileCannon
        | DamageMissileVisual::MissileLauncher => DamageMissileImpactEffectProfile::Rocket(
            rocket_impact_profile(visual).expect("rocket visual has impact profile"),
        ),
        DamageMissileVisual::ToughRocket => DamageMissileImpactEffectProfile::ToughRocket,
        DamageMissileVisual::MapObjectTurrent(_) => {
            DamageMissileImpactEffectProfile::MapObjectTurrent
        }
        DamageMissileVisual::Generic | DamageMissileVisual::Grenade => {
            DamageMissileImpactEffectProfile::Generic
        }
    }
}

pub(crate) fn damage_missile_impact_sound(visual: DamageMissileVisual) -> Option<UnitImpactSound> {
    Some(match visual {
        DamageMissileVisual::MapObjectTurrent(_) => UnitImpactSound::TurrentExplosion,
        _ => UnitImpactSound::RandomExplosion,
    })
}

fn missile_direction(start: Vec2, target: Vec2) -> Vec2 {
    let delta = target - start;
    let mag = delta.length();
    if mag <= f32::EPSILON {
        Vec2::X
    } else {
        delta / mag
    }
}
