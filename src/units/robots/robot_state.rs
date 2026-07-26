use crate::{
    components::CombatRng,
    original::objects::{ObjectKind, RobotType},
};

use super::robot_ui::{
    GRENADE_PICKUP_FRAME_COUNT, GRENADE_PICKUP_FRAME_TIME, GRENADE_THROW_FRAME_COUNT,
    GRENADE_THROW_FRAME_TIME, IDLE_ACTION_FRAME_TIME,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RobotIdleActionKind {
    Cigarette,
    Beer,
    FullScan,
    HeadStretch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RobotIdleProcessChoice {
    None,
    Turn(usize),
    Action(RobotIdleActionKind),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RobotFireAnimationReset {
    pub(crate) frame: usize,
    pub(crate) elapsed: f32,
    pub(crate) delay: f32,
}

pub(crate) fn grenade_throw_start_frame() -> usize {
    0
}

pub(crate) fn grenade_pickup_start_frame() -> usize {
    0
}

pub(crate) fn grenade_pickup_uses_upward_frames(rotation: u16) -> bool {
    matches!(rotation % 360, 0 | 45 | 90 | 135)
}

pub(crate) fn grenade_ready_attack_pose_active(
    kind: ObjectKind,
    has_throwable_grenade: bool,
    target_attacked_only_by_explosives: bool,
) -> bool {
    matches!(kind, ObjectKind::Robot(robot) if robot != RobotType::Tough)
        && (has_throwable_grenade || target_attacked_only_by_explosives)
}

pub(crate) fn grenade_ready_attack_pose_frame() -> usize {
    0
}

pub(crate) fn idle_action_start_frame() -> usize {
    0
}

pub(crate) fn idle_action_default_direction() -> usize {
    6
}

pub(crate) fn idle_action_frame_count(kind: RobotIdleActionKind) -> usize {
    match kind {
        RobotIdleActionKind::Cigarette => 11,
        RobotIdleActionKind::Beer => 10,
        RobotIdleActionKind::FullScan => 12,
        RobotIdleActionKind::HeadStretch => 11,
    }
}

pub(crate) fn idle_process_choice(
    activity_roll: usize,
    turn_roll: usize,
    direction_roll: usize,
    action_roll: usize,
) -> RobotIdleProcessChoice {
    if activity_roll % 10 != 0 {
        return RobotIdleProcessChoice::None;
    }

    if turn_roll % 3 != 0 {
        return RobotIdleProcessChoice::Turn(direction_roll % 8);
    }

    RobotIdleProcessChoice::Action(match action_roll % 4 {
        0 => RobotIdleActionKind::Cigarette,
        1 => RobotIdleActionKind::Beer,
        2 => RobotIdleActionKind::FullScan,
        _ => RobotIdleActionKind::HeadStretch,
    })
}

pub(crate) fn common_process_delta_seconds(delta_secs: f32, speed_offset_percent: f32) -> f32 {
    delta_secs.max(0.0) * speed_offset_percent.max(0.0)
}

pub(crate) fn fire_animation_start_frame() -> usize {
    0
}

pub(crate) fn fire_animation_start_delay(robot: RobotType) -> f32 {
    if robot == RobotType::Tough { 0.0 } else { 0.1 }
}

pub(crate) fn fire_animation_reset_for_attack_assignment(
    robot: RobotType,
) -> RobotFireAnimationReset {
    RobotFireAnimationReset {
        frame: fire_animation_start_frame(),
        elapsed: 0.0,
        delay: fire_animation_start_delay(robot),
    }
}

pub(crate) fn fire_animation_projectile_start_frame(robot: RobotType) -> usize {
    if robot == RobotType::Tough { 1 } else { 0 }
}

pub(crate) fn fire_animation_projectile_frame(robot: RobotType) -> usize {
    match robot {
        RobotType::Grunt | RobotType::Sniper => 4,
        RobotType::Psycho | RobotType::Tough => 1,
        RobotType::Pyro | RobotType::Laser => 2,
    }
}

pub(crate) fn fire_animation_frame_count(robot: RobotType) -> usize {
    match robot {
        RobotType::Grunt | RobotType::Sniper => 5,
        RobotType::Psycho => 2,
        RobotType::Tough | RobotType::Pyro | RobotType::Laser => 3,
    }
}

pub(crate) fn fire_animation_delay_after_frame(
    robot: RobotType,
    frame: usize,
    rng: &mut CombatRng,
) -> f32 {
    let roll = rng.index(100) as f32;
    match robot {
        RobotType::Grunt => {
            if frame >= 3 {
                0.40 + roll * 0.002
            } else {
                0.02
            }
        }
        RobotType::Sniper => {
            if frame >= 3 {
                0.30 + roll * 0.0018
            } else {
                0.02
            }
        }
        RobotType::Psycho => 0.07 + roll * 0.0003,
        RobotType::Pyro => {
            if frame % fire_animation_frame_count(robot) == 0 {
                0.07 + roll * 0.0003
            } else {
                0.05 + roll * 0.0003
            }
        }
        RobotType::Laser => {
            if frame % fire_animation_frame_count(robot) == 0 {
                0.30 + roll * 0.002
            } else {
                0.05 + roll * 0.0003
            }
        }
        RobotType::Tough => {
            if frame % fire_animation_frame_count(robot) == 0 {
                0.70 + roll * 0.003
            } else {
                0.05 + roll * 0.0003
            }
        }
    }
}

pub(crate) fn advance_fire_animation(
    robot: RobotType,
    frame: &mut usize,
    elapsed: &mut f32,
    delay: &mut f32,
    delta_secs: f32,
    rng: &mut CombatRng,
) {
    if robot == RobotType::Tough && *frame == 0 {
        *elapsed = 0.0;
        *delay = 0.0;
        return;
    }

    *elapsed += delta_secs.max(0.0);
    while *delay > 0.0 && *elapsed >= *delay {
        *elapsed -= *delay;
        *frame = next_fire_animation_frame(robot, *frame);
        *delay = fire_animation_delay_after_frame(robot, *frame, rng);
        if robot == RobotType::Tough && *frame == 0 {
            *elapsed = 0.0;
            *delay = 0.0;
            break;
        }
    }
}

fn next_fire_animation_frame(robot: RobotType, frame: usize) -> usize {
    match robot {
        RobotType::Grunt | RobotType::Sniper => {
            let next = frame + 1;
            if next >= fire_animation_frame_count(robot) {
                3
            } else {
                next
            }
        }
        RobotType::Psycho | RobotType::Pyro | RobotType::Laser | RobotType::Tough => {
            (frame + 1) % fire_animation_frame_count(robot)
        }
    }
}

pub(crate) fn advance_grenade_throw_animation(
    frame: &mut usize,
    elapsed: &mut f32,
    delta_secs: f32,
) -> bool {
    *elapsed += delta_secs.max(0.0);
    while *elapsed >= GRENADE_THROW_FRAME_TIME {
        *elapsed -= GRENADE_THROW_FRAME_TIME;
        *frame += 1;
        if *frame >= GRENADE_THROW_FRAME_COUNT {
            *frame = 0;
            *elapsed = 0.0;
            return true;
        }
    }

    false
}

pub(crate) fn advance_grenade_pickup_animation(
    frame: &mut usize,
    elapsed: &mut f32,
    delta_secs: f32,
) -> bool {
    *elapsed += delta_secs.max(0.0);
    while *elapsed >= GRENADE_PICKUP_FRAME_TIME {
        *elapsed -= GRENADE_PICKUP_FRAME_TIME;
        *frame += 1;
        if *frame >= GRENADE_PICKUP_FRAME_COUNT {
            *frame = 0;
            *elapsed = 0.0;
            return true;
        }
    }

    false
}

pub(crate) fn advance_idle_action_animation(
    kind: RobotIdleActionKind,
    frame: &mut usize,
    elapsed: &mut f32,
    delta_secs: f32,
) -> bool {
    *elapsed += delta_secs.max(0.0);
    while *elapsed >= IDLE_ACTION_FRAME_TIME {
        *elapsed -= IDLE_ACTION_FRAME_TIME;
        *frame += 1;
        if *frame >= idle_action_frame_count(kind) {
            *frame = 0;
            *elapsed = 0.0;
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grenade_throw_animation_matches_original_tick_timing() {
        let mut frame = grenade_throw_start_frame();
        let mut elapsed = 0.0;

        assert!(!advance_grenade_throw_animation(
            &mut frame,
            &mut elapsed,
            0.149
        ));
        assert_eq!(frame, 0);
        assert!(!advance_grenade_throw_animation(
            &mut frame,
            &mut elapsed,
            0.001
        ));
        assert_eq!(frame, 1);

        assert!(!advance_grenade_throw_animation(
            &mut frame,
            &mut elapsed,
            0.15
        ));
        assert_eq!(frame, 2);
        assert!(!advance_grenade_throw_animation(
            &mut frame,
            &mut elapsed,
            0.15
        ));
        assert_eq!(frame, 3);
        assert!(advance_grenade_throw_animation(
            &mut frame,
            &mut elapsed,
            0.15
        ));
        assert_eq!(frame, 0);
        assert_eq!(elapsed, 0.0);
    }

    #[test]
    fn grenade_pickup_animation_matches_original_common_process_frames() {
        assert_eq!(grenade_pickup_start_frame(), 0);
        assert!(grenade_pickup_uses_upward_frames(0));
        assert!(grenade_pickup_uses_upward_frames(135));
        assert!(!grenade_pickup_uses_upward_frames(180));
        assert!(!grenade_pickup_uses_upward_frames(315));

        let mut frame = grenade_pickup_start_frame();
        let mut elapsed = 0.0;
        assert!(!advance_grenade_pickup_animation(
            &mut frame,
            &mut elapsed,
            0.299
        ));
        assert_eq!(frame, 0);
        assert!(!advance_grenade_pickup_animation(
            &mut frame,
            &mut elapsed,
            0.002
        ));
        assert_eq!(frame, 1);
        assert!(!advance_grenade_pickup_animation(
            &mut frame,
            &mut elapsed,
            0.6
        ));
        assert_eq!(frame, 3);
        assert!(advance_grenade_pickup_animation(
            &mut frame,
            &mut elapsed,
            0.3
        ));
        assert_eq!(frame, 0);
        assert_eq!(elapsed, 0.0);
    }

    #[test]
    fn idle_process_choice_matches_original_random_branches() {
        assert_eq!(
            idle_process_choice(1, 0, 0, 0),
            RobotIdleProcessChoice::None
        );
        assert_eq!(
            idle_process_choice(0, 1, 5, 0),
            RobotIdleProcessChoice::Turn(5)
        );
        assert_eq!(
            idle_process_choice(0, 0, 5, 0),
            RobotIdleProcessChoice::Action(RobotIdleActionKind::Cigarette)
        );
        assert_eq!(
            idle_process_choice(0, 0, 5, 1),
            RobotIdleProcessChoice::Action(RobotIdleActionKind::Beer)
        );
        assert_eq!(
            idle_process_choice(0, 0, 5, 2),
            RobotIdleProcessChoice::Action(RobotIdleActionKind::FullScan)
        );
        assert_eq!(
            idle_process_choice(0, 0, 5, 3),
            RobotIdleProcessChoice::Action(RobotIdleActionKind::HeadStretch)
        );
        assert_eq!(idle_action_default_direction(), 6);
    }

    #[test]
    fn idle_action_animation_profiles_match_original_frame_counts() {
        assert_eq!(idle_action_frame_count(RobotIdleActionKind::Cigarette), 11);
        assert_eq!(idle_action_frame_count(RobotIdleActionKind::Beer), 10);
        assert_eq!(idle_action_frame_count(RobotIdleActionKind::FullScan), 12);
        assert_eq!(
            idle_action_frame_count(RobotIdleActionKind::HeadStretch),
            11
        );

        let mut frame = idle_action_start_frame();
        let mut elapsed = 0.0;
        assert!(!advance_idle_action_animation(
            RobotIdleActionKind::Beer,
            &mut frame,
            &mut elapsed,
            0.302
        ));
        assert_eq!(frame, 1);
        assert!(!advance_idle_action_animation(
            RobotIdleActionKind::Beer,
            &mut frame,
            &mut elapsed,
            2.4
        ));
        assert_eq!(frame, 9);
        assert!(advance_idle_action_animation(
            RobotIdleActionKind::Beer,
            &mut frame,
            &mut elapsed,
            0.3
        ));
        assert_eq!(frame, 0);
        assert_eq!(elapsed, 0.0);
    }

    #[test]
    fn common_process_delta_scales_with_original_speed_offset() {
        assert_eq!(common_process_delta_seconds(0.1, 1.0), 0.1);
        assert!((common_process_delta_seconds(0.1, 1.8) - 0.18).abs() < f32::EPSILON);
        assert_eq!(common_process_delta_seconds(-1.0, 1.8), 0.0);
        assert_eq!(common_process_delta_seconds(0.1, -1.0), 0.0);
    }

    #[test]
    fn grenade_ready_attack_pose_matches_original_robot_exclusions() {
        assert!(grenade_ready_attack_pose_active(
            ObjectKind::Robot(RobotType::Grunt),
            true,
            false
        ));
        assert!(grenade_ready_attack_pose_active(
            ObjectKind::Robot(RobotType::Laser),
            false,
            true
        ));
        assert!(!grenade_ready_attack_pose_active(
            ObjectKind::Robot(RobotType::Tough),
            true,
            true
        ));
        assert!(!grenade_ready_attack_pose_active(
            ObjectKind::Vehicle(crate::original::objects::VehicleType::Jeep),
            true,
            true
        ));
        assert_eq!(grenade_ready_attack_pose_frame(), 0);
    }

    #[test]
    fn fire_animation_profiles_match_original_process_timing() {
        assert_eq!(fire_animation_frame_count(RobotType::Grunt), 5);
        assert_eq!(fire_animation_frame_count(RobotType::Sniper), 5);
        assert_eq!(fire_animation_frame_count(RobotType::Psycho), 2);
        assert_eq!(fire_animation_frame_count(RobotType::Pyro), 3);
        assert_eq!(fire_animation_frame_count(RobotType::Laser), 3);
        assert_eq!(fire_animation_frame_count(RobotType::Tough), 3);
        assert_eq!(fire_animation_start_delay(RobotType::Grunt), 0.1);
        assert_eq!(fire_animation_start_delay(RobotType::Tough), 0.0);
        assert_eq!(fire_animation_projectile_start_frame(RobotType::Tough), 1);
        assert_eq!(fire_animation_projectile_frame(RobotType::Grunt), 4);
        assert_eq!(fire_animation_projectile_frame(RobotType::Sniper), 4);
        assert_eq!(fire_animation_projectile_frame(RobotType::Psycho), 1);
        assert_eq!(fire_animation_projectile_frame(RobotType::Pyro), 2);
        assert_eq!(fire_animation_projectile_frame(RobotType::Laser), 2);
        assert_eq!(fire_animation_projectile_frame(RobotType::Tough), 1);

        let mut rng = CombatRng::default();
        for _ in 0..16 {
            assert!((0.40..=0.598).contains(&fire_animation_delay_after_frame(
                RobotType::Grunt,
                4,
                &mut rng
            )));
            assert!((0.30..=0.4782).contains(&fire_animation_delay_after_frame(
                RobotType::Sniper,
                4,
                &mut rng
            )));
            assert!((0.07..=0.0997).contains(&fire_animation_delay_after_frame(
                RobotType::Psycho,
                1,
                &mut rng
            )));
            assert!((0.05..=0.0797).contains(&fire_animation_delay_after_frame(
                RobotType::Pyro,
                2,
                &mut rng
            )));
            assert!((0.30..=0.498).contains(&fire_animation_delay_after_frame(
                RobotType::Laser,
                0,
                &mut rng
            )));
            assert!((0.70..=0.997).contains(&fire_animation_delay_after_frame(
                RobotType::Tough,
                0,
                &mut rng
            )));
        }
    }

    #[test]
    fn attack_assignment_reset_matches_original_action_start() {
        let reset = fire_animation_reset_for_attack_assignment(RobotType::Grunt);
        assert_eq!(reset.frame, 0);
        assert_eq!(reset.elapsed, 0.0);
        assert_eq!(reset.delay, 0.1);

        let tough_reset = fire_animation_reset_for_attack_assignment(RobotType::Tough);
        assert_eq!(tough_reset.frame, 0);
        assert_eq!(tough_reset.elapsed, 0.0);
        assert_eq!(tough_reset.delay, 0.0);
    }

    #[test]
    fn fire_animation_advances_like_original_loops() {
        let mut rng = CombatRng::default();
        let mut frame = fire_animation_start_frame();
        let mut elapsed = 0.0;
        let mut delay = fire_animation_start_delay(RobotType::Grunt);

        advance_fire_animation(
            RobotType::Grunt,
            &mut frame,
            &mut elapsed,
            &mut delay,
            0.1,
            &mut rng,
        );
        assert_eq!(frame, 1);
        assert_eq!(delay, 0.02);

        advance_fire_animation(
            RobotType::Grunt,
            &mut frame,
            &mut elapsed,
            &mut delay,
            0.06,
            &mut rng,
        );
        assert_eq!(frame, 3);
        assert!((0.40..=0.598).contains(&delay));

        advance_fire_animation(
            RobotType::Grunt,
            &mut frame,
            &mut elapsed,
            &mut delay,
            0.6,
            &mut rng,
        );
        assert_eq!(frame, 4);

        let mut tough_frame = fire_animation_start_frame();
        let mut tough_elapsed = 0.0;
        let mut tough_delay = fire_animation_start_delay(RobotType::Tough);
        advance_fire_animation(
            RobotType::Tough,
            &mut tough_frame,
            &mut tough_elapsed,
            &mut tough_delay,
            1.0,
            &mut rng,
        );
        assert_eq!(tough_frame, 0);
        assert_eq!(tough_delay, 0.0);
    }

    #[test]
    fn grenade_throw_animation_large_delta_finishes_without_leaking_frame() {
        let mut frame = 0;
        let mut elapsed = 0.0;

        assert!(advance_grenade_throw_animation(
            &mut frame,
            &mut elapsed,
            10.0
        ));
        assert_eq!(frame, 0);
        assert_eq!(elapsed, 0.0);
    }
}
