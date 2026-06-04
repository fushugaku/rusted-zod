use bevy::prelude::*;
use pathfinding::prelude::astar;

use crate::{
    components::PassabilityGrid,
    constants::TILE_SIZE,
    original::{
        map::{MapObjectType, ZMap},
        objects::{BuildingType, ItemType, ObjectKind},
        tileinfo::PaletteTileInfo,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RouteFootprint {
    Robot,
    Vehicle,
}

impl PassabilityGrid {
    pub(crate) fn build(map: &ZMap, tile_info: &[PaletteTileInfo]) -> Self {
        let mut walkable = vec![true; map.basics.width as usize * map.basics.height as usize];
        let mut vehicle_walkable =
            vec![true; map.basics.width as usize * map.basics.height as usize];
        let mut walk_speed = vec![1.0; map.basics.width as usize * map.basics.height as usize];

        for (i, tile) in map.tiles.iter().enumerate() {
            let info = tile_info.get(tile.tile as usize).copied();
            let speed = info.map_or(0.0, |info| info.walk_speed());
            walkable[i] = speed > 0.0;
            vehicle_walkable[i] = speed > 0.0 && !info.is_some_and(|info| info.is_water);
            walk_speed[i] = speed;
        }

        for object in &map.objects {
            match object.object_type {
                MapObjectType::MapItem if object.object_id == ItemType::Rock as u8 => {
                    Self::block_tile(
                        &mut walkable,
                        &mut vehicle_walkable,
                        map.basics.width,
                        map.basics.height,
                        object.x,
                        object.y.saturating_add(2),
                    );
                }
                MapObjectType::MapItem
                    if object.object_id == ItemType::Hut as u8
                        || object.object_id >= ItemType::MapObjectStart as u8 =>
                {
                    Self::block_tile(
                        &mut walkable,
                        &mut vehicle_walkable,
                        map.basics.width,
                        map.basics.height,
                        object.x,
                        object.y,
                    );
                }
                MapObjectType::Bridge => {
                    if let Ok(building) = BuildingType::try_from(object.object_id) {
                        Self::apply_bridge_impassables(
                            &mut walkable,
                            &mut vehicle_walkable,
                            map.basics.width,
                            map.basics.height,
                            object.x,
                            object.y,
                            building,
                            object.extra_links,
                            object.health_percent <= 0,
                        );
                    }
                }
                MapObjectType::Building => {
                    if let Ok(building) = BuildingType::try_from(object.object_id) {
                        Self::apply_building_impassables(
                            &mut walkable,
                            &mut vehicle_walkable,
                            map.basics.width,
                            map.basics.height,
                            object.x,
                            object.y,
                            building,
                        );
                    }
                }
                MapObjectType::Cannon => {
                    Self::block_tile(
                        &mut walkable,
                        &mut vehicle_walkable,
                        map.basics.width,
                        map.basics.height,
                        object.x,
                        object.y,
                    );
                }
                _ => {}
            }
        }

        Self {
            width: map.basics.width,
            height: map.basics.height,
            walkable,
            vehicle_walkable,
            walk_speed,
        }
    }

    pub(crate) fn set_bridge_destroyed(
        &mut self,
        x: u16,
        y: u16,
        building: BuildingType,
        extra_links: u16,
    ) {
        Self::apply_bridge_center_impassables(
            &mut self.walkable,
            &mut self.vehicle_walkable,
            self.width,
            self.height,
            x,
            y,
            building,
            extra_links,
            true,
        );
    }

    pub(crate) fn set_bridge_repaired(
        &mut self,
        x: u16,
        y: u16,
        building: BuildingType,
        extra_links: u16,
    ) {
        Self::apply_bridge_center_impassables(
            &mut self.walkable,
            &mut self.vehicle_walkable,
            self.width,
            self.height,
            x,
            y,
            building,
            extra_links,
            false,
        );
    }

    pub(crate) fn set_walkable_tile(&mut self, x: u16, y: u16, value: bool) {
        Self::set_walkable(
            &mut self.walkable,
            &mut self.vehicle_walkable,
            self.width,
            self.height,
            x,
            y,
            value,
        );
    }

    pub(crate) fn bridge_bounds(
        x: u16,
        y: u16,
        building: BuildingType,
        extra_links: u16,
    ) -> Option<(u16, u16, u16, u16)> {
        let (width, height) = Self::bridge_dimensions(building, extra_links)?;
        Some((x, y, width, height))
    }

    fn apply_bridge_impassables(
        walkable: &mut [bool],
        vehicle_walkable: &mut [bool],
        map_width: u16,
        map_height: u16,
        x: u16,
        y: u16,
        building: BuildingType,
        extra_links: u16,
        destroyed: bool,
    ) {
        let Some((bridge_width, bridge_height)) = Self::bridge_dimensions(building, extra_links)
        else {
            return;
        };

        match building {
            BuildingType::BridgeVert => {
                for ty in y..y.saturating_add(bridge_height) {
                    Self::block_tile(walkable, vehicle_walkable, map_width, map_height, x, ty);
                    Self::block_tile(
                        walkable,
                        vehicle_walkable,
                        map_width,
                        map_height,
                        x.saturating_add(3),
                        ty,
                    );
                }
            }
            BuildingType::BridgeHorz => {
                for tx in x..x.saturating_add(bridge_width) {
                    Self::block_tile(walkable, vehicle_walkable, map_width, map_height, tx, y);
                    Self::block_tile(
                        walkable,
                        vehicle_walkable,
                        map_width,
                        map_height,
                        tx,
                        y.saturating_add(3),
                    );
                }
            }
            _ => {}
        }

        Self::apply_bridge_center_impassables(
            walkable,
            vehicle_walkable,
            map_width,
            map_height,
            x,
            y,
            building,
            extra_links,
            destroyed,
        );
    }

    fn apply_bridge_center_impassables(
        walkable: &mut [bool],
        vehicle_walkable: &mut [bool],
        map_width: u16,
        map_height: u16,
        x: u16,
        y: u16,
        building: BuildingType,
        extra_links: u16,
        impassable: bool,
    ) {
        let Some((bridge_width, bridge_height)) = Self::bridge_dimensions(building, extra_links)
        else {
            return;
        };

        match building {
            BuildingType::BridgeVert => {
                for ty in y..y.saturating_add(bridge_height) {
                    Self::set_walkable(
                        walkable,
                        vehicle_walkable,
                        map_width,
                        map_height,
                        x.saturating_add(1),
                        ty,
                        !impassable,
                    );
                    Self::set_walkable(
                        walkable,
                        vehicle_walkable,
                        map_width,
                        map_height,
                        x.saturating_add(2),
                        ty,
                        !impassable,
                    );
                }
            }
            BuildingType::BridgeHorz => {
                for tx in x..x.saturating_add(bridge_width) {
                    Self::set_walkable(
                        walkable,
                        vehicle_walkable,
                        map_width,
                        map_height,
                        tx,
                        y.saturating_add(1),
                        !impassable,
                    );
                    Self::set_walkable(
                        walkable,
                        vehicle_walkable,
                        map_width,
                        map_height,
                        tx,
                        y.saturating_add(2),
                        !impassable,
                    );
                }
            }
            _ => {}
        }
    }

    fn bridge_dimensions(building: BuildingType, extra_links: u16) -> Option<(u16, u16)> {
        match building {
            BuildingType::BridgeVert => Some((4, 5u16.saturating_add(extra_links))),
            BuildingType::BridgeHorz => Some((5u16.saturating_add(extra_links), 4)),
            _ => None,
        }
    }

    fn apply_building_impassables(
        walkable: &mut [bool],
        vehicle_walkable: &mut [bool],
        map_width: u16,
        map_height: u16,
        x: u16,
        y: u16,
        building: BuildingType,
    ) {
        match building {
            BuildingType::Radar => {
                Self::block_rect(
                    walkable,
                    vehicle_walkable,
                    map_width,
                    map_height,
                    x,
                    y,
                    4,
                    3,
                );
                Self::set_walkable(
                    walkable,
                    vehicle_walkable,
                    map_width,
                    map_height,
                    x.saturating_add(3),
                    y.saturating_add(2),
                    true,
                );
            }
            BuildingType::Repair => {
                Self::block_rect(
                    walkable,
                    vehicle_walkable,
                    map_width,
                    map_height,
                    x,
                    y,
                    5,
                    4,
                );
            }
            BuildingType::RobotFactory | BuildingType::VehicleFactory => {
                Self::block_rect(
                    walkable,
                    vehicle_walkable,
                    map_width,
                    map_height,
                    x,
                    y,
                    4,
                    5,
                );
            }
            BuildingType::FortFront => {
                Self::apply_building_mask(
                    walkable,
                    vehicle_walkable,
                    map_width,
                    map_height,
                    x,
                    y,
                    &[
                        ".##....##.",
                        ".########.",
                        ".########.",
                        "##########",
                        "##########",
                        "##########",
                        "####..####",
                        ".###..###.",
                        "..##..##..",
                        "...#..#...",
                    ],
                );
            }
            BuildingType::FortBack => {
                Self::apply_building_mask(
                    walkable,
                    vehicle_walkable,
                    map_width,
                    map_height,
                    x,
                    y,
                    &[
                        ".##....##.",
                        ".###..###.",
                        ".###..###.",
                        "##########",
                        "##########",
                        "##########",
                        "##########",
                        ".########.",
                        "..#....#..",
                        "..........",
                    ],
                );
            }
            BuildingType::BridgeVert | BuildingType::BridgeHorz => {}
        }
    }

    fn apply_building_mask(
        walkable: &mut [bool],
        vehicle_walkable: &mut [bool],
        map_width: u16,
        map_height: u16,
        x: u16,
        y: u16,
        rows: &[&str],
    ) {
        for (dy, row) in rows.iter().enumerate() {
            for (dx, cell) in row.as_bytes().iter().enumerate() {
                if *cell == b'#' {
                    Self::block_tile(
                        walkable,
                        vehicle_walkable,
                        map_width,
                        map_height,
                        x.saturating_add(dx as u16),
                        y.saturating_add(dy as u16),
                    );
                }
            }
        }
    }

    fn block_rect(
        walkable: &mut [bool],
        vehicle_walkable: &mut [bool],
        map_width: u16,
        map_height: u16,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
    ) {
        for tx in x..x.saturating_add(width) {
            for ty in y..y.saturating_add(height) {
                Self::block_tile(walkable, vehicle_walkable, map_width, map_height, tx, ty);
            }
        }
    }

    fn block_tile(
        walkable: &mut [bool],
        vehicle_walkable: &mut [bool],
        width: u16,
        height: u16,
        x: u16,
        y: u16,
    ) {
        Self::set_walkable(walkable, vehicle_walkable, width, height, x, y, false);
    }

    fn set_walkable(
        walkable: &mut [bool],
        vehicle_walkable: &mut [bool],
        width: u16,
        height: u16,
        x: u16,
        y: u16,
        value: bool,
    ) {
        if x >= width || y >= height {
            return;
        }

        let index = y as usize * width as usize + x as usize;
        walkable[index] = value;
        vehicle_walkable[index] = value;
    }

    pub(crate) fn is_walkable(&self, tile: IVec2) -> bool {
        self.tile_passable(tile, RouteFootprint::Robot)
    }

    #[cfg(test)]
    pub(crate) fn is_walkable_for(&self, tile: IVec2, footprint: RouteFootprint) -> bool {
        self.can_occupy(tile, footprint)
    }

    fn tile_passable(&self, tile: IVec2, footprint: RouteFootprint) -> bool {
        if tile.x < 0 || tile.y < 0 || tile.x >= self.width as i32 || tile.y >= self.height as i32 {
            return false;
        }

        let index = tile.y as usize * self.width as usize + tile.x as usize;
        match footprint {
            RouteFootprint::Robot => self.walkable[index],
            RouteFootprint::Vehicle => self.vehicle_walkable[index],
        }
    }

    fn can_occupy(&self, tile: IVec2, footprint: RouteFootprint) -> bool {
        match footprint {
            RouteFootprint::Robot => self.tile_passable(tile, footprint),
            RouteFootprint::Vehicle => {
                self.tile_passable(tile, footprint)
                    && self.tile_passable(IVec2::new(tile.x + 1, tile.y), footprint)
                    && self.tile_passable(IVec2::new(tile.x, tile.y + 1), footprint)
                    && self.tile_passable(IVec2::new(tile.x + 1, tile.y + 1), footprint)
            }
        }
    }

    pub(crate) fn walk_speed_at_world(&self, world: Vec2) -> f32 {
        let tile = self.world_to_tile(world);
        if tile.x < 0 || tile.y < 0 || tile.x >= self.width as i32 || tile.y >= self.height as i32 {
            return 0.0;
        }

        self.walk_speed[tile.y as usize * self.width as usize + tile.x as usize]
    }

    pub(crate) fn world_to_tile(&self, world: Vec2) -> IVec2 {
        IVec2::new(
            (world.x / TILE_SIZE).floor() as i32,
            (-world.y / TILE_SIZE).floor() as i32,
        )
    }

    fn world_to_route_tile(&self, world: Vec2, footprint: RouteFootprint) -> IVec2 {
        match footprint {
            RouteFootprint::Robot => self.world_to_tile(world),
            RouteFootprint::Vehicle => IVec2::new(
                ((world.x - TILE_SIZE * 0.5) / TILE_SIZE).floor() as i32,
                ((-world.y - TILE_SIZE * 0.5) / TILE_SIZE).floor() as i32,
            ),
        }
    }

    pub(crate) fn tile_center(&self, tile: IVec2) -> Vec2 {
        Vec2::new(
            tile.x as f32 * TILE_SIZE + TILE_SIZE * 0.5,
            -(tile.y as f32 * TILE_SIZE + TILE_SIZE * 0.5),
        )
    }

    fn route_point(&self, tile: IVec2, footprint: RouteFootprint) -> Vec2 {
        match footprint {
            RouteFootprint::Robot => self.tile_center(tile),
            RouteFootprint::Vehicle => Vec2::new(
                tile.x as f32 * TILE_SIZE + TILE_SIZE,
                -(tile.y as f32 * TILE_SIZE + TILE_SIZE),
            ),
        }
    }

    pub(crate) fn route(&self, start_world: Vec2, end_world: Vec2) -> Option<Vec<Vec2>> {
        self.route_with_footprint(start_world, end_world, RouteFootprint::Robot)
    }

    pub(crate) fn route_for_object_kind(
        &self,
        start_world: Vec2,
        end_world: Vec2,
        kind: ObjectKind,
    ) -> Option<Vec<Vec2>> {
        self.route_with_footprint(start_world, end_world, route_footprint_for_kind(kind))
    }

    pub(crate) fn route_with_footprint(
        &self,
        start_world: Vec2,
        end_world: Vec2,
        footprint: RouteFootprint,
    ) -> Option<Vec<Vec2>> {
        let start = self.world_to_route_tile(start_world, footprint);
        let goal = self.world_to_route_tile(end_world, footprint);
        if !self.can_occupy(start, footprint) || !self.can_occupy(goal, footprint) {
            return None;
        }

        let result = astar(
            &start,
            |tile| {
                [
                    IVec2::new(tile.x - 1, tile.y - 1),
                    IVec2::new(tile.x - 1, tile.y),
                    IVec2::new(tile.x - 1, tile.y + 1),
                    IVec2::new(tile.x, tile.y - 1),
                    IVec2::new(tile.x, tile.y + 1),
                    IVec2::new(tile.x + 1, tile.y - 1),
                    IVec2::new(tile.x + 1, tile.y),
                    IVec2::new(tile.x + 1, tile.y + 1),
                ]
                .into_iter()
                .filter(|next| self.can_step(*tile, *next, footprint))
                .map(|next| {
                    let diagonal = next.x != tile.x && next.y != tile.y;
                    (next, if diagonal { 14 } else { 10 })
                })
                .collect::<Vec<_>>()
            },
            |tile| octile_heuristic(*tile, goal),
            |tile| *tile == goal,
        )?;

        let mut points: Vec<Vec2> = simplify_collinear_tiles(result.0)
            .into_iter()
            .skip(1)
            .map(|tile| self.route_point(tile, footprint))
            .collect();
        if points
            .last()
            .is_none_or(|point| point.distance_squared(end_world) > 0.01)
        {
            points.push(end_world);
        }

        Some(points)
    }

    fn can_step(&self, from: IVec2, to: IVec2, footprint: RouteFootprint) -> bool {
        if !self.can_occupy(to, footprint) {
            return false;
        }

        let delta = to - from;
        if delta.x == 0 || delta.y == 0 {
            return true;
        }

        match footprint {
            RouteFootprint::Robot => {
                self.tile_passable(IVec2::new(from.x + delta.x, from.y), footprint)
                    && self.tile_passable(IVec2::new(from.x, from.y + delta.y), footprint)
            }
            RouteFootprint::Vehicle => {
                if delta.x > 0 && delta.y < 0 {
                    self.tile_passable(IVec2::new(from.x, from.y - 1), footprint)
                        && self.tile_passable(IVec2::new(from.x + 2, from.y + 1), footprint)
                } else if delta.x < 0 && delta.y < 0 {
                    self.tile_passable(IVec2::new(from.x + 1, from.y - 1), footprint)
                        && self.tile_passable(IVec2::new(from.x - 1, from.y + 1), footprint)
                } else if delta.x > 0 && delta.y > 0 {
                    self.tile_passable(IVec2::new(from.x + 2, from.y), footprint)
                        && self.tile_passable(IVec2::new(from.x, from.y + 2), footprint)
                } else {
                    self.tile_passable(IVec2::new(from.x - 1, from.y), footprint)
                        && self.tile_passable(IVec2::new(from.x + 1, from.y + 2), footprint)
                }
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn route_to_attack_range(
        &self,
        start_world: Vec2,
        target_world: Vec2,
        attack_radius: f32,
    ) -> Option<Vec<Vec2>> {
        self.route_to_attack_range_with_footprint(
            start_world,
            target_world,
            attack_radius,
            RouteFootprint::Robot,
        )
    }

    pub(crate) fn route_to_attack_range_for_object_kind(
        &self,
        start_world: Vec2,
        target_world: Vec2,
        attack_radius: f32,
        kind: ObjectKind,
    ) -> Option<Vec<Vec2>> {
        self.route_to_attack_range_with_footprint(
            start_world,
            target_world,
            attack_radius,
            route_footprint_for_kind(kind),
        )
    }

    pub(crate) fn route_to_attack_range_with_footprint(
        &self,
        start_world: Vec2,
        target_world: Vec2,
        attack_radius: f32,
        footprint: RouteFootprint,
    ) -> Option<Vec<Vec2>> {
        if start_world.distance(target_world) <= attack_radius {
            return Some(Vec::new());
        }

        let mut candidates = Vec::new();
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let tile = IVec2::new(x, y);
                if !self.can_occupy(tile, footprint) {
                    continue;
                }

                let center = self.route_point(tile, footprint);
                if center.distance(target_world) <= attack_radius {
                    candidates.push(center);
                }
            }
        }

        candidates.sort_by(|a, b| {
            a.distance_squared(start_world)
                .total_cmp(&b.distance_squared(start_world))
        });

        candidates
            .into_iter()
            .take(16)
            .filter_map(|candidate| self.route_with_footprint(start_world, candidate, footprint))
            .min_by(|a, b| a.len().cmp(&b.len()))
    }
}

pub(crate) fn route_footprint_for_kind(kind: ObjectKind) -> RouteFootprint {
    match kind {
        ObjectKind::Robot(_) => RouteFootprint::Robot,
        _ => RouteFootprint::Vehicle,
    }
}

fn octile_heuristic(tile: IVec2, goal: IVec2) -> u32 {
    let dx = (tile.x - goal.x).unsigned_abs();
    let dy = (tile.y - goal.y).unsigned_abs();
    14 * dx.min(dy) + 10 * dx.max(dy).saturating_sub(dx.min(dy))
}

fn simplify_collinear_tiles(mut path: Vec<IVec2>) -> Vec<IVec2> {
    if path.len() < 3 {
        return path;
    }

    let mut index = 1;
    while index + 1 < path.len() {
        let previous_dir = direction_delta(path[index - 1], path[index]);
        let current_dir = direction_delta(path[index], path[index + 1]);
        if previous_dir == current_dir {
            path.remove(index);
        } else {
            index += 1;
        }
    }

    path
}

fn direction_delta(from: IVec2, to: IVec2) -> IVec2 {
    (to - from).clamp(IVec2::splat(-1), IVec2::splat(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::{
        map::{MapBasics, MapObject, MapTile, ZMap},
        tileinfo::PaletteTileInfo,
        types::{PlanetType, TeamType},
    };

    fn passable_tile_info() -> Vec<PaletteTileInfo> {
        vec![PaletteTileInfo {
            is_water: false,
            is_passable: true,
            is_usable: false,
            is_road: false,
            is_effect: false,
            is_water_effect: false,
            next_tile_in_effect: 0,
            takes_tank_tracks: false,
            crater_type: 0,
            is_starter_tile: false,
        }]
    }

    fn map_with_bridge(building: BuildingType, health_percent: i32) -> ZMap {
        let width = 8;
        let height = 8;
        ZMap {
            basics: MapBasics {
                width,
                height,
                map_name: "bridge_test".to_string(),
                player_count: 2,
                object_count: 1,
                terrain_type: PlanetType::Desert,
                zone_count: 0,
            },
            zones: Vec::new(),
            objects: vec![MapObject {
                x: 1,
                y: 1,
                owner: TeamType::Null,
                object_type: MapObjectType::Bridge,
                object_id: building as u8,
                building_level: 0,
                extra_links: 1,
                health_percent,
            }],
            tiles: vec![MapTile { tile: 0 }; width as usize * height as usize],
        }
    }

    fn map_with_building(building: BuildingType, x: u16, y: u16) -> ZMap {
        let width = 10;
        let height = 10;
        ZMap {
            basics: MapBasics {
                width,
                height,
                map_name: "building_test".to_string(),
                player_count: 2,
                object_count: 1,
                terrain_type: PlanetType::Desert,
                zone_count: 0,
            },
            zones: Vec::new(),
            objects: vec![MapObject {
                x,
                y,
                owner: TeamType::Red,
                object_type: MapObjectType::Building,
                object_id: building as u8,
                building_level: 0,
                extra_links: 0,
                health_percent: 100,
            }],
            tiles: vec![MapTile { tile: 0 }; width as usize * height as usize],
        }
    }

    #[test]
    fn live_vertical_bridge_blocks_edges_but_leaves_center_passable() {
        let map = map_with_bridge(BuildingType::BridgeVert, 100);
        let grid = PassabilityGrid::build(&map, &passable_tile_info());

        assert!(!grid.is_walkable(IVec2::new(1, 1)));
        assert!(!grid.is_walkable(IVec2::new(4, 1)));
        assert!(grid.is_walkable(IVec2::new(2, 1)));
        assert!(grid.is_walkable(IVec2::new(3, 6)));
    }

    #[test]
    fn destroyed_horizontal_bridge_closes_center_lanes() {
        let map = map_with_bridge(BuildingType::BridgeHorz, 0);
        let grid = PassabilityGrid::build(&map, &passable_tile_info());

        assert!(!grid.is_walkable(IVec2::new(1, 1)));
        assert!(!grid.is_walkable(IVec2::new(1, 4)));
        assert!(!grid.is_walkable(IVec2::new(1, 2)));
        assert!(!grid.is_walkable(IVec2::new(6, 3)));
    }

    #[test]
    fn bridge_destroy_update_closes_previously_open_center() {
        let map = map_with_bridge(BuildingType::BridgeVert, 100);
        let mut grid = PassabilityGrid::build(&map, &passable_tile_info());

        assert!(grid.is_walkable(IVec2::new(2, 3)));
        grid.set_bridge_destroyed(1, 1, BuildingType::BridgeVert, 1);
        assert!(!grid.is_walkable(IVec2::new(2, 3)));
    }

    #[test]
    fn bridge_repair_update_reopens_destroyed_center_like_original() {
        let map = map_with_bridge(BuildingType::BridgeVert, 0);
        let mut grid = PassabilityGrid::build(&map, &passable_tile_info());

        grid.set_bridge_destroyed(1, 1, BuildingType::BridgeVert, 0);
        assert!(!grid.is_walkable(IVec2::new(2, 3)));
        grid.set_bridge_repaired(1, 1, BuildingType::BridgeVert, 0);
        assert!(grid.is_walkable(IVec2::new(2, 3)));
        assert!(!grid.is_walkable(IVec2::new(1, 3)));
        assert!(!grid.is_walkable(IVec2::new(4, 3)));
    }

    #[test]
    fn rock_impassable_matches_original_bottom_tile_only() {
        let mut map = map_with_bridge(BuildingType::BridgeVert, 100);
        map.objects = vec![MapObject {
            x: 2,
            y: 2,
            owner: TeamType::Null,
            object_type: MapObjectType::MapItem,
            object_id: ItemType::Rock as u8,
            building_level: 0,
            extra_links: 0,
            health_percent: 100,
        }];

        let mut grid = PassabilityGrid::build(&map, &passable_tile_info());

        assert!(grid.is_walkable(IVec2::new(2, 2)));
        assert!(grid.is_walkable(IVec2::new(2, 3)));
        assert!(!grid.is_walkable(IVec2::new(2, 4)));
        grid.set_walkable_tile(2, 4, true);
        assert!(grid.is_walkable(IVec2::new(2, 4)));
    }

    #[test]
    fn hut_and_generic_map_objects_block_but_pickups_do_not() {
        let mut map = map_with_bridge(BuildingType::BridgeVert, 100);
        map.objects = vec![
            MapObject {
                x: 1,
                y: 1,
                owner: TeamType::Null,
                object_type: MapObjectType::MapItem,
                object_id: ItemType::Hut as u8,
                building_level: 0,
                extra_links: 0,
                health_percent: 100,
            },
            MapObject {
                x: 2,
                y: 1,
                owner: TeamType::Null,
                object_type: MapObjectType::MapItem,
                object_id: ItemType::MapObjectStart as u8,
                building_level: 0,
                extra_links: 0,
                health_percent: 100,
            },
            MapObject {
                x: 3,
                y: 1,
                owner: TeamType::Null,
                object_type: MapObjectType::MapItem,
                object_id: ItemType::Grenades as u8,
                building_level: 0,
                extra_links: 0,
                health_percent: 100,
            },
            MapObject {
                x: 4,
                y: 1,
                owner: TeamType::Red,
                object_type: MapObjectType::MapItem,
                object_id: ItemType::Flag as u8,
                building_level: 0,
                extra_links: 0,
                health_percent: 100,
            },
            MapObject {
                x: 5,
                y: 1,
                owner: TeamType::Null,
                object_type: MapObjectType::MapItem,
                object_id: ItemType::Rockets as u8,
                building_level: 0,
                extra_links: 0,
                health_percent: 100,
            },
        ];

        let grid = PassabilityGrid::build(&map, &passable_tile_info());

        assert!(!grid.is_walkable(IVec2::new(1, 1)));
        assert!(!grid.is_walkable(IVec2::new(2, 1)));
        assert!(grid.is_walkable(IVec2::new(3, 1)));
        assert!(grid.is_walkable(IVec2::new(4, 1)));
        assert!(grid.is_walkable(IVec2::new(5, 1)));
    }

    #[test]
    fn vehicle_tiles_reject_water_but_robot_tiles_accept_it_like_original() {
        let mut map = map_with_bridge(BuildingType::BridgeVert, 100);
        map.objects.clear();
        map.tiles[1] = MapTile { tile: 1 };
        let mut info = passable_tile_info();
        info.push(PaletteTileInfo {
            is_water: true,
            ..info[0]
        });

        let grid = PassabilityGrid::build(&map, &info);

        assert!(grid.is_walkable_for(IVec2::new(1, 0), RouteFootprint::Robot));
        assert!(!grid.is_walkable_for(IVec2::new(1, 0), RouteFootprint::Vehicle));
    }

    #[test]
    fn vehicle_footprint_requires_original_two_by_two_clearance() {
        let mut grid = PassabilityGrid {
            width: 4,
            height: 4,
            walkable: vec![true; 16],
            vehicle_walkable: vec![true; 16],
            walk_speed: vec![1.0; 16],
        };
        grid.vehicle_walkable[1 + 2 * grid.width as usize] = false;

        assert!(grid.is_walkable_for(IVec2::new(1, 1), RouteFootprint::Robot));
        assert!(!grid.is_walkable_for(IVec2::new(1, 1), RouteFootprint::Vehicle));
    }

    #[test]
    fn vehicle_route_rejects_one_tile_gap_that_robot_can_use() {
        let mut walkable = vec![true; 15];
        for tile in [IVec2::new(2, 0), IVec2::new(2, 2)] {
            walkable[tile.y as usize * 5 + tile.x as usize] = false;
        }
        let grid = PassabilityGrid {
            width: 5,
            height: 3,
            vehicle_walkable: walkable.clone(),
            walkable,
            walk_speed: vec![1.0; 15],
        };

        assert!(
            grid.route_with_footprint(
                grid.tile_center(IVec2::new(0, 1)),
                grid.tile_center(IVec2::new(4, 1)),
                RouteFootprint::Robot,
            )
            .is_some()
        );
        assert!(
            grid.route_with_footprint(
                grid.route_point(IVec2::new(0, 1), RouteFootprint::Vehicle),
                grid.route_point(IVec2::new(3, 1), RouteFootprint::Vehicle),
                RouteFootprint::Vehicle,
            )
            .is_none()
        );
    }

    #[test]
    fn vehicle_diagonal_corner_checks_match_original_astar_tile_ok() {
        let mut grid = PassabilityGrid {
            width: 4,
            height: 4,
            walkable: vec![true; 16],
            vehicle_walkable: vec![true; 16],
            walk_speed: vec![1.0; 16],
        };

        assert!(grid.can_step(IVec2::new(1, 1), IVec2::new(2, 2), RouteFootprint::Vehicle));

        grid.vehicle_walkable[1 * grid.width as usize + 3] = false;
        assert!(!grid.can_step(IVec2::new(1, 1), IVec2::new(2, 2), RouteFootprint::Vehicle));

        grid.vehicle_walkable[1 * grid.width as usize + 3] = true;
        grid.vehicle_walkable[3 * grid.width as usize + 1] = false;
        assert!(!grid.can_step(IVec2::new(1, 1), IVec2::new(2, 2), RouteFootprint::Vehicle));
    }

    #[test]
    fn route_simplifies_collinear_tiles_and_appends_exact_endpoint() {
        let grid = PassabilityGrid {
            width: 6,
            height: 3,
            walkable: vec![true; 18],
            vehicle_walkable: vec![true; 18],
            walk_speed: vec![1.0; 18],
        };
        let exact_end = Vec2::new(75.0, -24.0);

        let route = grid
            .route_with_footprint(
                grid.tile_center(IVec2::new(0, 1)),
                exact_end,
                RouteFootprint::Robot,
            )
            .expect("straight route");

        assert_eq!(route.len(), 2);
        assert_eq!(route[0], grid.tile_center(IVec2::new(4, 1)));
        assert_eq!(route[1], exact_end);
    }

    #[test]
    fn radar_impassables_leave_original_bottom_right_open() {
        let map = map_with_building(BuildingType::Radar, 1, 1);
        let grid = PassabilityGrid::build(&map, &passable_tile_info());

        assert!(!grid.is_walkable(IVec2::new(1, 1)));
        assert!(!grid.is_walkable(IVec2::new(4, 1)));
        assert!(!grid.is_walkable(IVec2::new(1, 3)));
        assert!(grid.is_walkable(IVec2::new(4, 3)));
        assert!(grid.is_walkable(IVec2::new(5, 3)));
    }

    #[test]
    fn repair_impassables_block_full_original_footprint() {
        let map = map_with_building(BuildingType::Repair, 1, 1);
        let grid = PassabilityGrid::build(&map, &passable_tile_info());

        assert!(!grid.is_walkable(IVec2::new(1, 1)));
        assert!(!grid.is_walkable(IVec2::new(5, 4)));
        assert!(grid.is_walkable(IVec2::new(6, 4)));
        assert!(grid.is_walkable(IVec2::new(5, 5)));
    }

    #[test]
    fn factory_impassables_block_full_original_footprints() {
        for building in [BuildingType::RobotFactory, BuildingType::VehicleFactory] {
            let map = map_with_building(building, 1, 1);
            let grid = PassabilityGrid::build(&map, &passable_tile_info());

            assert!(!grid.is_walkable(IVec2::new(1, 1)));
            assert!(!grid.is_walkable(IVec2::new(4, 5)));
            assert!(grid.is_walkable(IVec2::new(5, 5)));
            assert!(grid.is_walkable(IVec2::new(4, 6)));
        }
    }

    #[test]
    fn fort_front_impassables_match_original_entrance_mask() {
        let map = map_with_building(BuildingType::FortFront, 0, 0);
        let grid = PassabilityGrid::build(&map, &passable_tile_info());

        assert!(grid.is_walkable(IVec2::new(0, 0)));
        assert!(!grid.is_walkable(IVec2::new(1, 0)));
        assert!(grid.is_walkable(IVec2::new(4, 6)));
        assert!(grid.is_walkable(IVec2::new(5, 7)));
        assert!(!grid.is_walkable(IVec2::new(3, 9)));
        assert!(grid.is_walkable(IVec2::new(2, 9)));
    }

    #[test]
    fn fort_back_impassables_match_original_entrance_mask() {
        let map = map_with_building(BuildingType::FortBack, 0, 0);
        let grid = PassabilityGrid::build(&map, &passable_tile_info());

        assert!(grid.is_walkable(IVec2::new(0, 0)));
        assert!(!grid.is_walkable(IVec2::new(1, 0)));
        assert!(grid.is_walkable(IVec2::new(4, 1)));
        assert!(grid.is_walkable(IVec2::new(5, 2)));
        assert!(!grid.is_walkable(IVec2::new(2, 8)));
        assert!(grid.is_walkable(IVec2::new(3, 8)));
    }
}
