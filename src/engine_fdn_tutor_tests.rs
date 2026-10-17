//! Paid tutor obligations, required choices, private knowledge and owned replay.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;
use crate::{SearchDestination, SearchFilter};

const TUTORS: [&str; 4] = [
    "Fierce Empath",
    "Micromancer",
    "Rune-Scarred Demon",
    "Vile Entomber",
];
fn optional(name: &str) -> bool {
    matches!(name, "Fierce Empath" | "Micromancer")
}
fn result_name(name: &str) -> &str {
    match name {
        "Fierce Empath" => "Quakestrider Ceratops",
        "Micromancer" => "Opt",
        _ => "Driver of the Dead",
    }
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..80 {
        if stop(g) {
            return;
        }
        let actions = g.legal_actions();
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .or_else(|| {
                    actions
                        .iter()
                        .find(|a| matches!(a, Action::PutTrigger { .. }))
                })
                .unwrap_or_else(|| panic!("unexpected {:?}: {actions:?}", g.state.pending))
                .clone()
        };
        checked(g, action);
    }
    panic!("did not reach expected choice: {:?}", g.state.pending);
}
fn reject(g: &mut Game, action: Action) {
    let snapshot = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), snapshot);
    assert_eq!(g.revision(), revision);
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let source = named(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = match name {
        "Fierce Empath" => [2, 0, 0, 0, 1, 0],
        "Micromancer" => [3, 1, 0, 0, 0, 0],
        "Rune-Scarred Demon" => [5, 0, 2, 0, 0, 0],
        "Vile Entomber" => [2, 0, 2, 0, 0, 0],
        _ => unreachable!(),
    };
    cast(g, source, None, &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    source
}
fn enter(g: &mut Game, p: Player, name: &str) -> CardId {
    let source = paid(g, p, name);
    until(g, |g| {
        matches!(
            g.state.pending,
            Pending::EffectOptional { .. } | Pending::EffectSearch { .. }
        )
    });
    let view = g.object_view(source);
    let (power, toughness) = match name {
        "Fierce Empath" => (1, 1),
        "Micromancer" => (3, 3),
        "Rune-Scarred Demon" => (6, 6),
        _ => (2, 2),
    };
    assert_eq!((view.power, view.toughness), (power, toughness));
    assert_eq!(
        matches!(g.state.pending, Pending::EffectOptional { .. }),
        optional(name)
    );
    source
}
fn begin_search(g: &mut Game, name: &str) {
    if optional(name) {
        checked(g, Action::ChooseOptional { accept: true });
    }
    assert!(matches!(g.state.pending, Pending::EffectSearch { .. }));
}
fn commit_replay(g: &mut Game) {
    let partial = g.snapshot();
    let mut branch = {
        let parent = g.clone();
        parent.clone()
    };
    checked(&mut branch, Action::FinishSearch);
    checked(g, Action::FinishSearch);
    assert_eq!(branch.snapshot(), g.snapshot());
    let after = g.snapshot();
    branch.restore(&partial).unwrap();
    checked(&mut branch, Action::FinishSearch);
    assert_eq!(branch.snapshot(), after);
}
fn private_choice(g: &Game, p: Player) {
    let other = g.observe(p.other());
    assert!(other.search_choice.is_none());
    assert!(other.actions.is_empty());
    assert!(other.objects.iter().all(|o| o.zone != Zone::Library));
}

#[test]
fn qualified_tutors_paid_boundaries_reveal_only_chosen_hand_card_and_owned_replay() {
    for p in [Player::P0, Player::P1] {
        for (name, result) in [
            ("Fierce Empath", "Quakestrider Ceratops"),
            ("Fierce Empath", "Zetalpa, Primal Dawn"),
            ("Micromancer", "Opt"),
            ("Micromancer", "Bushwhack"),
        ] {
            let mut g = setup(p);
            let chosen = named(&mut g, p, result, Zone::Library);
            let low = named(
                &mut g,
                p,
                if name == "Fierce Empath" {
                    "Serra Angel"
                } else {
                    "Negate"
                },
                Zone::Library,
            );
            let wrong_kind = named(
                &mut g,
                p,
                if name == "Fierce Empath" {
                    "River's Rebuke"
                } else {
                    "Llanowar Elves"
                },
                Zone::Library,
            );
            let wrong_owner = named(&mut g, p.other(), result, Zone::Library);
            let hand = named(&mut g, p, result, Zone::Hand);
            enter(&mut g, p, name);
            begin_search(&mut g, name);
            let view = g.observe(p).search_choice.unwrap();
            assert_eq!(view.minimum, 0);
            assert!(view.reveal);
            assert!(g.legal_actions().contains(&Action::FinishSearch));
            for bad in [low, wrong_kind, wrong_owner, hand, u32::MAX] {
                reject(&mut g, Action::ChooseSearch { card: bad });
            }
            let library = g.state.players[p.index()].library.clone();
            let rng = g.state.rng;
            checked(&mut g, Action::ChooseSearch { card: chosen });
            reject(&mut g, Action::ChooseSearch { card: chosen });
            assert_eq!(g.state.players[p.index()].library, library);
            assert_eq!(g.state.rng, rng);
            private_choice(&g, p);
            commit_replay(&mut g);
            assert_eq!(g.object_view(chosen).zone, Zone::Hand);
            let observed = g.observe(p.other());
            assert!(
                observed
                    .objects
                    .iter()
                    .any(|o| o.id == chosen && o.zone == Zone::Hand)
            );
            assert!(!observed.objects.iter().any(|o| o.id == hand));
            assert!(
                !observed
                    .objects
                    .iter()
                    .any(|o| o.id == low || o.id == wrong_kind)
            );
            let mut expected = library.clone();
            let index = expected.iter().position(|&id| id == chosen).unwrap();
            expected.remove(index);
            let mut expected_rng = rng;
            expected_rng.shuffle(&mut expected);
            assert_eq!(g.state.players[p.index()].library, expected);
            assert_eq!(g.state.rng, expected_rng);
        }
    }
}

#[test]
fn optional_decline_avoids_shuffle_but_qualified_failure_with_matches_still_shuffles() {
    for p in [Player::P0, Player::P1] {
        for name in &TUTORS[..2] {
            for accept in [false, true] {
                let mut g = setup(p);
                named(&mut g, p, result_name(name), Zone::Library);
                enter(&mut g, p, name);
                let library = g.state.players[p.index()].library.clone();
                let mut rng = g.state.rng;
                checked(&mut g, Action::ChooseOptional { accept });
                let mut expected = library.clone();
                if accept {
                    checked(&mut g, Action::FinishSearch);
                    rng.shuffle(&mut expected);
                }
                assert_eq!(g.state.players[p.index()].library, expected);
                assert_eq!(g.state.rng, rng);
                assert!(g.observe(p).search_choice.is_none());
            }
        }
    }
}

#[test]
fn qualified_no_matches_and_all_four_empty_library_searches_complete_without_drawing() {
    for p in [Player::P0, Player::P1] {
        for name in TUTORS {
            for empty in [false, true] {
                if !empty && !optional(name) {
                    continue;
                }
                let mut g = setup(p);
                if empty {
                    for id in g.state.players[p.index()].library.clone() {
                        g.move_object(id, Zone::Exile);
                    }
                }
                enter(&mut g, p, name);
                begin_search(&mut g, name);
                let view = g.observe(p).search_choice.unwrap();
                assert_eq!(view.minimum, 0);
                let actions: Vec<_> = g
                    .legal_actions()
                    .into_iter()
                    .filter(|a| *a != Action::Concede)
                    .collect();
                assert_eq!(actions, vec![Action::FinishSearch]);
                commit_replay(&mut g);
                assert!(!g.state.players[p.index()].attempted_empty_draw);
            }
        }
    }
}

#[test]
fn unqualified_tutors_must_find_and_keep_hand_private_but_graveyard_public() {
    for p in [Player::P0, Player::P1] {
        for name in &TUTORS[2..] {
            let mut g = setup(p);
            let selected = named(&mut g, p, "Driver of the Dead", Zone::Library);
            // This prior public knowledge must not identify a privately chosen card.
            let reference = g.object_ref(selected);
            g.remember_library_top(p, reference);
            let wrong_owner = g.state.players[p.other().index()].library[0];
            let hand = named(&mut g, p, "Forest", Zone::Hand);
            enter(&mut g, p, name);
            begin_search(&mut g, name);
            let view = g.observe(p).search_choice.unwrap();
            assert_eq!(view.filter, SearchFilter::Any);
            assert_eq!(
                view.destination,
                if *name == "Vile Entomber" {
                    SearchDestination::Graveyard
                } else {
                    SearchDestination::Hand
                }
            );
            assert_eq!(view.minimum, 1);
            assert!(!view.reveal);
            reject(&mut g, Action::FinishSearch);
            reject(&mut g, Action::ChooseOptional { accept: false });
            for bad in [wrong_owner, hand, u32::MAX] {
                reject(&mut g, Action::ChooseSearch { card: bad });
            }
            checked(&mut g, Action::ChooseSearch { card: selected });
            reject(&mut g, Action::ChooseSearch { card: selected });
            private_choice(&g, p);
            assert_eq!(
                g.observe(p.other()).known_library_top[p.index()]
                    .unwrap()
                    .object,
                reference
            );
            commit_replay(&mut g);
            let graveyard = *name == "Vile Entomber";
            assert_eq!(
                g.object_view(selected).zone,
                if graveyard {
                    Zone::Graveyard
                } else {
                    Zone::Hand
                }
            );
            let other = g.observe(p.other());
            assert_eq!(other.objects.iter().any(|o| o.id == selected), graveyard);
            assert!(other.known_library_top[p.index()].is_none());
            assert!(
                g.state.pending_triggers.is_empty(),
                "library-to-graveyard is not dying"
            );
        }
    }
}

#[test]
fn tutor_search_uses_captured_controller_after_source_control_change_and_departure() {
    for p in [Player::P0, Player::P1] {
        for name in TUTORS {
            let mut g = setup(p);
            let selected = named(&mut g, p, result_name(name), Zone::Library);
            let source = paid(&mut g, p, name);
            until(&mut g, |g| {
                g.state
                    .stack
                    .last()
                    .is_some_and(|s| matches!(s.kind, StackKind::TriggeredAbility { .. }))
            });
            g.state.objects[source as usize].controller = p.other();
            g.move_object(source, Zone::Exile);
            until(&mut g, |g| {
                matches!(
                    g.state.pending,
                    Pending::EffectOptional { .. } | Pending::EffectSearch { .. }
                )
            });
            assert_eq!(g.actor(), Some(p));
            begin_search(&mut g, name);
            private_choice(&g, p);
            checked(&mut g, Action::ChooseSearch { card: selected });
            commit_replay(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Exile);
            assert_eq!(g.object_view(selected).owner, p);
            assert_eq!(
                g.object_view(selected).zone,
                if name == "Vile Entomber" {
                    Zone::Graveyard
                } else {
                    Zone::Hand
                }
            );
        }
    }
}

#[test]
fn failed_tutor_commit_restores_required_choice_rng_private_knowledge_and_revision() {
    for p in [Player::P0, Player::P1] {
        for name in TUTORS {
            let mut g = setup(p);
            let selected = named(&mut g, p, result_name(name), Zone::Library);
            g.state.objects[selected as usize].generation = u64::MAX;
            let reference = g.object_ref(selected);
            g.remember_library_top(p, reference);
            enter(&mut g, p, name);
            begin_search(&mut g, name);
            checked(&mut g, Action::ChooseSearch { card: selected });
            let before = g.snapshot();
            let revision = g.revision();
            let views = [
                serde_json::to_value(g.observe(p)).unwrap(),
                serde_json::to_value(g.observe(p.other())).unwrap(),
            ];
            for _ in 0..2 {
                assert!(matches!(
                    g.apply(p, Action::FinishSearch),
                    Err(Error::CapacityExceeded(_))
                ));
                assert_eq!(g.snapshot(), before);
                assert_eq!(g.revision(), revision);
                assert_eq!(
                    views,
                    [
                        serde_json::to_value(g.observe(p)).unwrap(),
                        serde_json::to_value(g.observe(p.other())).unwrap()
                    ]
                );
            }
            let mut branch = {
                let parent = g.clone();
                parent.clone()
            };
            let index = branch
                .legal_actions()
                .iter()
                .position(|a| *a == Action::FinishSearch)
                .unwrap();
            assert!(matches!(
                branch.apply_index(p, revision, index),
                Err(Error::CapacityExceeded(_))
            ));
            assert_eq!(branch.snapshot(), before);
        }
    }
}

#[test]
fn rune_flying_blocks_ground_and_vile_deathtouch_kills_a_larger_creature_in_paid_fight() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let demon = enter(&mut g, p, "Rune-Scarred Demon");
        let selected = g.state.players[p.index()].library[0];
        checked(&mut g, Action::ChooseSearch { card: selected });
        commit_replay(&mut g);
        let ground = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let flyer = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        assert!(!g.can_block_pair(ground, demon));
        assert!(g.can_block_pair(flyer, demon));
        let mut g = setup(p);
        let vile = enter(&mut g, p, "Vile Entomber");
        let selected = g.state.players[p.index()].library[0];
        checked(&mut g, Action::ChooseSearch { card: selected });
        commit_replay(&mut g);
        let large = named(
            &mut g,
            p.other(),
            "Quakestrider Ceratops",
            Zone::Battlefield,
        );
        let fight = named(&mut g, p, "Bushwhack", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        cast(
            &mut g,
            fight,
            Some(1),
            &[Target::Card(vile), Target::Card(large)],
        );
        settle(&mut g);
        assert_eq!(g.object_view(large).zone, Zone::Graveyard);
        assert_eq!(g.object_view(vile).zone, Zone::Graveyard);
    }
}

#[test]
fn singleton_required_search_preserves_legitimately_deducible_known_hand_identity() {
    let p = Player::P0;
    let mut g = setup(p);
    let ids = g.state.players[0].library.clone();
    let selected = ids[0];
    for &id in ids.iter().skip(1) {
        g.move_object(id, Zone::Exile);
    }
    let reference = g.object_ref(selected);
    g.remember_library_top(p, reference);
    enter(&mut g, p, "Rune-Scarred Demon");
    checked(&mut g, Action::ChooseSearch { card: selected });
    commit_replay(&mut g);
    assert!(
        g.observe(p.other())
            .objects
            .iter()
            .any(|o| o.id == selected && o.zone == Zone::Hand)
    );
}
