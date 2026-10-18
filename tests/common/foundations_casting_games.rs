//! Whole casting-permission consumers in natural, set-only legal games.
use super::{deck, run_matrix_actions_observed};
use rustmage::{CastMethod, GraveyardCastKind, Observation, StackKind, Step, Zone};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn foundations_three_casting_cards_natural_games_and_owned_permissions() {
    let blue = deck(
        "Island",
        &[
            "Sphinx of Forgotten Lore",
            "Omniscience",
            "Hedron Archive",
            "Gilded Lotus",
            "Solemn Simulacrum",
            "Think Twice",
            "Unsummon",
            "Fog Bank",
            "Opt",
        ],
    );
    let black = deck(
        "Swamp",
        &[
            "Zul Ashur, Lich Lord",
            "Diregraf Ghoul",
            "Crow of Dark Tidings",
            "Hungry Ghoul",
            "Maalfeld Twins",
            "Dread Summons",
            "Solemn Simulacrum",
            "Hedron Archive",
            "Gilded Lotus",
        ],
    );
    let mut paid_consumers = BTreeSet::new();
    let mut pending_permissions = BTreeMap::new();
    let mut granted_flashback_seats = BTreeSet::new();
    let mut zul_cast_seats = BTreeSet::new();
    let mut free_cast_seats = BTreeSet::new();
    let mut owned_sphinx_replays = 0;
    let mut owned_zul_replays = 0;
    let mut observe = |before: &Observation, after: &Observation, owned_replay: bool| {
        if before.step == Step::Pregame {
            pending_permissions.clear();
        }
        if owned_replay {
            for permission in &before.graveyard_cast_permissions {
                match permission.kind {
                    GraveyardCastKind::Cast => owned_zul_replays += 1,
                    GraveyardCastKind::Flashback => owned_sphinx_replays += 1,
                }
            }
        }
        // Announcing moves the card off the graveyard before its costs are paid.
        // Capture the actual old permission here, then require the finished cast.
        if let Some(item) = after.announcing.as_ref().filter(|item| {
            before
                .announcing
                .as_ref()
                .is_none_or(|old| old.id != item.id)
        }) && let Some(cast) = item.cast.filter(|cast| cast.origin == Zone::Graveyard)
        {
            let kind = match cast.method {
                CastMethod::Normal => Some(GraveyardCastKind::Cast),
                CastMethod::GrantedFlashback => Some(GraveyardCastKind::Flashback),
                _ => None,
            };
            if let Some(kind) = kind {
                let permission = before
                    .graveyard_cast_permissions
                    .iter()
                    .find(|permission| {
                        permission.object.card == item.card
                            && permission.player == item.controller
                            && permission.kind == kind
                            && permission.expires_turn >= before.turn
                    })
                    .expect("cast announcement must consume its exact graveyard permission");
                assert_ne!(permission.object, item.object);
                assert!(
                    !after
                        .graveyard_cast_permissions
                        .iter()
                        .any(|p| p.object == permission.object)
                );
                pending_permissions.insert((item.id, item.object), kind);
            }
        }
        for item in after.stack.iter().filter(|item| {
            item.kind == StackKind::Spell && !before.stack.iter().any(|old| old.id == item.id)
        }) {
            let cast = item.cast.unwrap();
            let name = item.source_card.name();
            if [
                "Sphinx of Forgotten Lore",
                "Zul Ashur, Lich Lord",
                "Omniscience",
            ]
            .contains(&name)
                && cast.origin == Zone::Hand
                && cast.method == CastMethod::Normal
            {
                assert!(cast.payment.into_iter().any(|amount| amount > 0));
                paid_consumers.insert((name, item.controller.index()));
            }
            match pending_permissions.remove(&(item.id, item.object)) {
                Some(GraveyardCastKind::Cast) => {
                    assert_eq!(cast.method, CastMethod::Normal);
                    assert_eq!(cast.origin, Zone::Graveyard);
                    assert!(item.source_card.is_creature());
                    zul_cast_seats.insert(item.controller.index());
                }
                Some(GraveyardCastKind::Flashback) => {
                    assert_eq!(cast.method, CastMethod::GrantedFlashback);
                    assert_eq!(cast.origin, Zone::Graveyard);
                    granted_flashback_seats.insert(item.controller.index());
                }
                None => {}
            }
            if cast.method == CastMethod::Free {
                assert_eq!(cast.origin, Zone::Hand);
                assert_eq!(cast.payment, [0; 6]);
                free_cast_seats.insert(item.controller.index());
            }
        }
    };
    run_matrix_actions_observed(
        &[blue, black],
        &[
            "Sphinx of Forgotten Lore",
            "Zul Ashur, Lich Lord",
            "Omniscience",
        ],
        &[],
        &["Hedron Archive", "Gilded Lotus"],
        &[],
        &[0, 1, 2, 3, 7, 13, 41, 73],
        Some(&mut observe),
    );
    for name in [
        "Sphinx of Forgotten Lore",
        "Zul Ashur, Lich Lord",
        "Omniscience",
    ] {
        for seat in [0, 1] {
            assert!(
                paid_consumers.contains(&(name, seat)),
                "missing paid {name} for seat {seat}"
            );
        }
    }
    assert_eq!(
        granted_flashback_seats.len(),
        2,
        "no granted flashback cast for both seats"
    );
    assert_eq!(
        zul_cast_seats.len(),
        2,
        "no Zul graveyard cast for both seats"
    );
    assert_eq!(
        free_cast_seats.len(),
        2,
        "no Omniscience free cast for both seats"
    );
    assert!(owned_sphinx_replays > 0 && owned_zul_replays > 0);
    println!(
        "FDN three casting consumers: paid all three, consumed Sphinx/Zul permissions and cast freely with Omniscience in both seats; {owned_sphinx_replays} owned Sphinx-state and {owned_zul_replays} owned Zul-state replays"
    );
}
