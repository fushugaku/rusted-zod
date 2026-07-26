use crate::{
    components::{CombatRng, PortraitAnimationKind},
    units::{UnitAttackSound, UnitImpactSound},
};

pub(crate) fn attack_sound_asset_path(sound: UnitAttackSound) -> &'static str {
    match sound {
        UnitAttackSound::Rifle => "sounds/RIFLE3.wav",
        UnitAttackSound::Psycho => "sounds/MACHGUN2.wav",
        UnitAttackSound::Tough => "sounds/MOBIMISS.wav",
        UnitAttackSound::Pyro => "sounds/FLAMER.wav",
        UnitAttackSound::Laser => "sounds/LASERGUN.wav",
        UnitAttackSound::Gun => "sounds/LTGUN.wav",
        UnitAttackSound::Gatling => "sounds/GATTGUN.wav",
        UnitAttackSound::Jeep => "sounds/JEEPMGUN.wav",
        UnitAttackSound::Light => "sounds/LTANKGUN.wav",
        UnitAttackSound::Medium => "sounds/MTANKGUN.wav",
        UnitAttackSound::Heavy => "sounds/HTANKGUN.wav",
        UnitAttackSound::MobileMissile => "sounds/MOBIMIS2.wav",
        UnitAttackSound::ThrowGrenade => "sounds/GRENLOBX.wav",
    }
}

pub(crate) fn impact_sound_asset_path(
    sound: UnitImpactSound,
    rng: Option<&mut CombatRng>,
) -> String {
    match sound {
        UnitImpactSound::RandomExplosion => {
            let index = rng.map_or(0, |rng| rng.index(5));
            format!("sounds/explosion_{index:02}.wav")
        }
        UnitImpactSound::TurrentExplosion => "sounds/METGRND.wav".to_string(),
    }
}

pub(crate) fn selected_common_portrait_animation(rng: &mut CombatRng) -> PortraitAnimationKind {
    PortraitAnimationKind::SelectedCommon(rng.index(4) as u8)
}

pub(crate) fn acknowledge_portrait_animation(
    no_way: bool,
    rng: &mut CombatRng,
) -> PortraitAnimationKind {
    if no_way {
        PortraitAnimationKind::AcknowledgeNoWay(rng.index(3) as u8)
    } else {
        PortraitAnimationKind::Acknowledge(rng.index(12) as u8)
    }
}

pub(crate) fn selected_common_voice_asset_path(
    anim_index: u8,
    rng: Option<&mut CombatRng>,
) -> String {
    let variant = if matches!(anim_index % 4, 0 | 3) {
        rng.map_or(0, |rng| rng.index(2))
    } else {
        0
    };
    selected_common_voice_asset_path_for_choice(anim_index, variant).to_string()
}

pub(crate) fn selected_common_voice_asset_path_for_choice(
    anim_index: u8,
    variant: usize,
) -> &'static str {
    match anim_index % 4 {
        0 if variant % 2 == 0 => "sounds/ROB01.wav",
        0 => "sounds/ROB02.wav",
        1 => "sounds/ROB03.wav",
        2 => "sounds/ROB04.wav",
        _ if variant % 2 == 0 => "sounds/ROB05.wav",
        _ => "sounds/ROB06.wav",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_generic_voice_assets_match_original_object_portrait_sounds() {
        assert_eq!(
            selected_common_voice_asset_path_for_choice(0, 0),
            "sounds/ROB01.wav"
        );
        assert_eq!(
            selected_common_voice_asset_path_for_choice(0, 1),
            "sounds/ROB02.wav"
        );
        assert_eq!(
            selected_common_voice_asset_path_for_choice(1, 0),
            "sounds/ROB03.wav"
        );
        assert_eq!(
            selected_common_voice_asset_path_for_choice(2, 0),
            "sounds/ROB04.wav"
        );
        assert_eq!(
            selected_common_voice_asset_path_for_choice(3, 0),
            "sounds/ROB05.wav"
        );
        assert_eq!(
            selected_common_voice_asset_path_for_choice(3, 1),
            "sounds/ROB06.wav"
        );
    }

    #[test]
    fn selected_common_portrait_sound_uses_existing_anim_choice() {
        assert_eq!(
            selected_common_voice_asset_path(1, Some(&mut CombatRng(0))),
            "sounds/ROB03.wav"
        );
        assert_eq!(
            selected_common_voice_asset_path(2, Some(&mut CombatRng(0))),
            "sounds/ROB04.wav"
        );
    }

    #[test]
    fn acknowledge_portrait_animation_matches_source_random_ranges() {
        assert_eq!(
            acknowledge_portrait_animation(false, &mut CombatRng(0)),
            PortraitAnimationKind::Acknowledge(0)
        );
        assert_eq!(
            acknowledge_portrait_animation(false, &mut CombatRng(1)),
            PortraitAnimationKind::Acknowledge(8)
        );
        assert_eq!(
            acknowledge_portrait_animation(true, &mut CombatRng(0)),
            PortraitAnimationKind::AcknowledgeNoWay(0)
        );
        assert_eq!(
            acknowledge_portrait_animation(true, &mut CombatRng(1)),
            PortraitAnimationKind::AcknowledgeNoWay(2)
        );
    }

    #[test]
    fn attack_and_impact_sound_assets_match_original_unit_sounds() {
        assert_eq!(
            attack_sound_asset_path(UnitAttackSound::Rifle),
            "sounds/RIFLE3.wav"
        );
        assert_eq!(
            attack_sound_asset_path(UnitAttackSound::MobileMissile),
            "sounds/MOBIMIS2.wav"
        );
        assert_eq!(
            attack_sound_asset_path(UnitAttackSound::ThrowGrenade),
            "sounds/GRENLOBX.wav"
        );
        assert_eq!(
            impact_sound_asset_path(UnitImpactSound::TurrentExplosion, None),
            "sounds/METGRND.wav"
        );
        assert_eq!(
            impact_sound_asset_path(UnitImpactSound::RandomExplosion, None),
            "sounds/explosion_00.wav"
        );
    }
}
