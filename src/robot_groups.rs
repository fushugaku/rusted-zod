#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RobotGroupMemberSnapshot {
    pub(crate) ref_id: u32,
    pub(crate) leader_ref_id: u32,
    pub(crate) destroyed: bool,
    pub(crate) grenade_amount: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RobotGroupPromotion {
    pub(crate) old_leader_ref_id: u32,
    pub(crate) new_leader_ref_id: u32,
    pub(crate) grenade_amount: u8,
}

pub(crate) fn robot_group_promotions_for_removed_refs(
    removed_refs: &[u32],
    members: &[RobotGroupMemberSnapshot],
) -> Vec<RobotGroupPromotion> {
    removed_refs
        .iter()
        .filter_map(|ref_id| {
            robot_group_promotion_for_removed_leader_excluding(*ref_id, removed_refs, members)
        })
        .collect()
}

#[cfg(test)]
fn robot_group_promotion_for_removed_leader(
    removed_ref_id: u32,
    members: &[RobotGroupMemberSnapshot],
) -> Option<RobotGroupPromotion> {
    robot_group_promotion_for_removed_leader_excluding(removed_ref_id, &[removed_ref_id], members)
}

fn robot_group_promotion_for_removed_leader_excluding(
    removed_ref_id: u32,
    removed_refs: &[u32],
    members: &[RobotGroupMemberSnapshot],
) -> Option<RobotGroupPromotion> {
    let removed_leader = members
        .iter()
        .find(|member| member.ref_id == removed_ref_id && member.leader_ref_id == removed_ref_id)?;

    let new_leader_ref_id = members
        .iter()
        .filter(|member| {
            member.ref_id != removed_ref_id
                && member.leader_ref_id == removed_ref_id
                && !member.destroyed
                && !removed_refs.contains(&member.ref_id)
        })
        .map(|member| member.ref_id)
        .min()?;

    Some(RobotGroupPromotion {
        old_leader_ref_id: removed_ref_id,
        new_leader_ref_id,
        grenade_amount: removed_leader.grenade_amount,
    })
}

pub(crate) fn remap_selected_refs_for_group_promotions(
    selected_refs: &mut Vec<u32>,
    promotions: &[RobotGroupPromotion],
) {
    for selected_ref in selected_refs.iter_mut() {
        if let Some(promotion) = promotions
            .iter()
            .find(|promotion| *selected_ref == promotion.old_leader_ref_id)
        {
            *selected_ref = promotion.new_leader_ref_id;
        }
    }

    let mut seen = Vec::new();
    selected_refs.retain(|ref_id| {
        if seen.contains(ref_id) {
            false
        } else {
            seen.push(*ref_id);
            true
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removed_robot_group_leader_promotes_first_live_minion_and_transfers_grenades() {
        let members = [
            RobotGroupMemberSnapshot {
                ref_id: 10,
                leader_ref_id: 10,
                destroyed: true,
                grenade_amount: 7,
            },
            RobotGroupMemberSnapshot {
                ref_id: 11,
                leader_ref_id: 10,
                destroyed: false,
                grenade_amount: 0,
            },
            RobotGroupMemberSnapshot {
                ref_id: 12,
                leader_ref_id: 10,
                destroyed: false,
                grenade_amount: 0,
            },
        ];

        assert_eq!(
            robot_group_promotion_for_removed_leader(10, &members),
            Some(RobotGroupPromotion {
                old_leader_ref_id: 10,
                new_leader_ref_id: 11,
                grenade_amount: 7,
            })
        );
    }

    #[test]
    fn removed_robot_group_minion_does_not_promote_group() {
        let members = [
            RobotGroupMemberSnapshot {
                ref_id: 10,
                leader_ref_id: 10,
                destroyed: false,
                grenade_amount: 0,
            },
            RobotGroupMemberSnapshot {
                ref_id: 11,
                leader_ref_id: 10,
                destroyed: true,
                grenade_amount: 0,
            },
        ];

        assert_eq!(robot_group_promotion_for_removed_leader(11, &members), None);
    }

    #[test]
    fn group_promotion_skips_minions_removed_in_same_tick() {
        let members = [
            RobotGroupMemberSnapshot {
                ref_id: 10,
                leader_ref_id: 10,
                destroyed: false,
                grenade_amount: 3,
            },
            RobotGroupMemberSnapshot {
                ref_id: 11,
                leader_ref_id: 10,
                destroyed: false,
                grenade_amount: 0,
            },
            RobotGroupMemberSnapshot {
                ref_id: 12,
                leader_ref_id: 10,
                destroyed: false,
                grenade_amount: 0,
            },
        ];

        assert_eq!(
            robot_group_promotions_for_removed_refs(&[10, 11], &members),
            vec![RobotGroupPromotion {
                old_leader_ref_id: 10,
                new_leader_ref_id: 12,
                grenade_amount: 3,
            }]
        );
    }

    #[test]
    fn group_promotion_remaps_selected_leader_without_duplicates() {
        let mut selected_refs = vec![4, 10, 11, 30];
        remap_selected_refs_for_group_promotions(
            &mut selected_refs,
            &[RobotGroupPromotion {
                old_leader_ref_id: 10,
                new_leader_ref_id: 11,
                grenade_amount: 0,
            }],
        );

        assert_eq!(selected_refs, vec![4, 11, 30]);
    }
}
