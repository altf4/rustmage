//! Seed Suture's two prepared creatures and hybrid spell.
use super::big_helpers::{checked, settle, setup};
use super::*;

fn announce(g: &mut Game, action: Action, target: Option<Target>) {
    checked(g, action);
    let mut target = target;
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let actions = g.legal_actions();
        let choice = target.map(|target| Action::ChooseTarget { target });
        let action = if choice.as_ref().is_some_and(|a| actions.contains(a)) {
            target = None;
            choice.unwrap()
        } else {
            actions
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::ChooseSpellCost { .. }
                            | Action::FinishTargets
                            | Action::PayCosts { .. }
                    )
                })
                .unwrap()
        };
        checked(g, action);
    }
    panic!("announcement did not complete");
}
#[test]
fn seed_suture_both_seats_and_hybrid_colors_with_invalidated_target() {
    for p in [Player::P0, Player::P1] {
        for (name, flying) in [
            ("Emergency Phytomedic // Seed Suture", false),
            ("Blossom-Blessed Angel // Seed Suture", true),
        ] {
            for green in [false, true] {
                for invalid in [false, true] {
                    let mut g = setup(p);
                    let source = put(&mut g, p, "Forest", Zone::Hand);
                    g.state.objects[source as usize].card = Card::from_name(name).unwrap();
                    announce(&mut g, Action::AnnounceCast { card: source }, None);
                    settle(&mut g);
                    assert_eq!(g.keywords(source).contains(Keywords::FLYING), flying);
                    assert_eq!(g.keywords(source).contains(Keywords::VIGILANCE), flying);
                    let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                    let copy = g
                        .state
                        .preparations
                        .as_ref()
                        .unwrap()
                        .iter()
                        .find(|r| r.permanent == g.object_ref(source))
                        .unwrap()
                        .spell_copy
                        .card;
                    g.state.players[p.index()].mana = if green {
                        [0, 0, 0, 0, 1, 0]
                    } else {
                        [1, 0, 0, 0, 0, 0]
                    };
                    let life = g.state.players[p.index()].life;
                    if p == Player::P0 && !green && !invalid {
                        super::contracts::branch_contract(g.clone());
                    }
                    announce(
                        &mut g,
                        Action::AnnounceExile { card: copy },
                        Some(Target::Card(target)),
                    );
                    assert!(!g.is_prepared(g.object_ref(source)));
                    if invalid {
                        g.move_object(target, Zone::Exile);
                    }
                    settle(&mut g);
                    assert_eq!(
                        g.counter_count(target, CounterKind::PlusOne),
                        u64::from(!invalid)
                    );
                    assert_eq!(g.state.players[p.index()].life, life + i128::from(!invalid));
                    assert_eq!(g.object_view(copy).zone, Zone::Outside);
                }
            }
        }
    }
}

#[test]
fn seed_suture_timing_invalid_target_and_owned_announcement() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = put(&mut g, p, "Forest", Zone::Hand);
        g.state.objects[source as usize].card =
            Card::from_name("Emergency Phytomedic // Seed Suture").unwrap();
        announce(&mut g, Action::AnnounceCast { card: source }, None);
        settle(&mut g);
        let source_ref = g.object_ref(source);
        let copy = g
            .state
            .preparations
            .as_ref()
            .unwrap()
            .iter()
            .find(|r| r.permanent == source_ref)
            .unwrap()
            .spell_copy;
        let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        g.state.active = p.other();
        g.state.priority = p;
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceExile { card: copy.card })
        );
        reject(&mut g, Action::AnnounceExile { card: copy.card });
        assert!(g.is_prepared(source_ref));
        g.state.active = p;
        checked(&mut g, Action::AnnounceExile { card: copy.card });
        if let Some(action) = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseSpellCost { .. }))
        {
            checked(&mut g, action);
        }
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        assert!(g.is_prepared(source_ref));
        let announced_copy = g
            .state
            .preparations
            .as_ref()
            .unwrap()
            .iter()
            .find(|r| r.permanent == source_ref)
            .unwrap()
            .spell_copy;
        let checkpoint = g.snapshot();
        super::contracts::branch_contract(g.clone());
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        assert!(!g.is_prepared(source_ref));
        settle(&mut g);
        assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 1);
        g.restore(&checkpoint).unwrap();
        assert!(g.is_prepared(source_ref));
        assert_eq!(
            g.state
                .preparations
                .as_ref()
                .unwrap()
                .iter()
                .find(|r| r.permanent == source_ref)
                .unwrap()
                .spell_copy,
            announced_copy
        );
    }
}
