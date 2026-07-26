use bevy::{
    camera::{Viewport, visibility::RenderLayers},
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};

use crate::{
    components::{CurrentMap, MainCamera, StartupScreenshot},
    constants::{GAME_LAYER, HUD_HEIGHT, HUD_LAYER, HUD_WIDTH, TILE_SIZE},
    original::map::ZMap,
};

impl StartupScreenshot {
    pub(crate) fn from_env() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(path) = std::env::var("ZOD_SCREENSHOT") {
                return Self {
                    path: Some(path),
                    frames_remaining: std::env::var("ZOD_SCREENSHOT_FRAMES")
                        .ok()
                        .and_then(|value| value.parse().ok())
                        .unwrap_or(30),
                    requested: false,
                };
            }
        }

        Self::default()
    }
}

pub(crate) fn setup_camera(mut commands: Commands, map: Res<CurrentMap>) {
    let map_size = map_pixel_size(&map.0);

    commands.spawn((
        Camera2d,
        Camera::default(),
        Transform::from_xyz(map_size.x * 0.5, -map_size.y * 0.5, 1000.0),
        RenderLayers::layer(GAME_LAYER),
        MainCamera,
    ));

    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1000.0),
        RenderLayers::layer(HUD_LAYER),
        crate::components::HudCamera,
    ));
}

pub(crate) fn map_pixel_size(map: &ZMap) -> Vec2 {
    Vec2::new(
        map.basics.width as f32 * TILE_SIZE,
        map.basics.height as f32 * TILE_SIZE,
    )
}

pub(crate) fn game_view_size(window: &Window) -> Vec2 {
    Vec2::new(
        (window.width() - HUD_WIDTH).max(1.0),
        (window.height() - HUD_HEIGHT).max(1.0),
    )
}

pub(crate) fn clamp_camera_center(center: Vec2, map_size: Vec2, view_size: Vec2) -> Vec2 {
    let x = if map_size.x <= view_size.x {
        map_size.x * 0.5
    } else {
        center
            .x
            .clamp(view_size.x * 0.5, map_size.x - view_size.x * 0.5)
    };

    let y = if map_size.y <= view_size.y {
        -map_size.y * 0.5
    } else {
        center
            .y
            .clamp(-(map_size.y - view_size.y * 0.5), -view_size.y * 0.5)
    };

    Vec2::new(x, y)
}

pub(crate) fn focus_camera_to_world_point(
    transform: &mut Transform,
    map: &ZMap,
    window: &Window,
    world_point: Vec2,
) {
    let clamped = clamp_camera_center(world_point, map_pixel_size(map), game_view_size(window));
    transform.translation.x = clamped.x;
    transform.translation.y = clamped.y;
}

pub(crate) fn cursor_world_position(
    windows: &Query<&Window>,
    camera_query: &Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> Option<Vec2> {
    let Ok(window) = windows.single() else {
        return None;
    };
    let cursor = window.cursor_position()?;
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return None;
    };
    camera.viewport_to_world_2d(camera_transform, cursor).ok()
}

pub(crate) fn capture_startup_screenshot(
    mut commands: Commands,
    mut request: ResMut<StartupScreenshot>,
) {
    if request.requested {
        return;
    }

    let Some(path) = request.path.clone() else {
        return;
    };

    if request.frames_remaining > 0 {
        request.frames_remaining -= 1;
        return;
    };

    request.requested = true;
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
}

pub(crate) fn sync_game_camera_viewport(
    windows: Query<&Window>,
    map: Res<CurrentMap>,
    mut camera_query: Query<(&mut Camera, &mut Transform), With<MainCamera>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((mut camera, mut transform)) = camera_query.single_mut() else {
        return;
    };

    let view_size = game_view_size(window);
    let scale = window.scale_factor();
    camera.viewport = Some(Viewport {
        physical_position: UVec2::ZERO,
        physical_size: UVec2::new(
            (view_size.x * scale).round().max(1.0) as u32,
            (view_size.y * scale).round().max(1.0) as u32,
        ),
        depth: 0.0..1.0,
    });

    let clamped = clamp_camera_center(
        transform.translation.truncate(),
        map_pixel_size(&map.0),
        view_size,
    );
    transform.translation.x = clamped.x;
    transform.translation.y = clamped.y;
}

pub(crate) fn camera_controls(
    time: Res<Time<Real>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<MainCamera>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        dir.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        dir.y -= 1.0;
    }

    if dir != Vec2::ZERO {
        let speed = 450.0;
        let delta = dir.normalize() * speed * time.delta_secs();
        transform.translation.x += delta.x;
        transform.translation.y += delta.y;
    }
}
