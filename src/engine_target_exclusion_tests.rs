use super::tests::{scenario, spawn};
use super::*;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn checked(g: &mut Game, action: Action) {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let before = g.snapshot();
    assert_eq!(
        g.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(g.snapshot(), before);
    let mut branch = {
        let parent = g.clone();
        parent.clone()
    };
    branch.apply_index(actor, revision, index).unwrap();
    g.apply(actor, action.clone()).unwrap();
    assert_eq!(g.snapshot(), branch.snapshot());
    for player in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(player)).unwrap(),
            serde_json::to_value(branch.observe(player)).unwrap()
        );
    }
    let after = g.snapshot();
    g.restore(&before).unwrap();
    assert_eq!(
        g.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    g.apply(actor, action).unwrap();
    assert_eq!(g.snapshot(), after);
}
fn settle_on(g: &mut Game, chosen: Option<Target>) {
    g.prepare_triggers();
    for _ in 0..100 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let action = if let Some(target) =
            chosen.filter(|t| actions.contains(&Action::ChooseTarget { target: *t }))
        {
            Action::ChooseTarget { target }
        } else if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .or_else(|| actions.iter().find(|a| matches!(a, Action::FinishTargets)))
                .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                .unwrap()
                .clone()
        };
        checked(g, action);
    }
    panic!("unsettled {:?}", g.state.pending);
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    g.state.players[p.index()].mana = [30; 6];
    g
}

fn settle(g: &mut Game) {
    settle_on(g, None);
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
fn cast(g: &mut Game, id: CardId, mode: Option<u16>, targets: &[Target]) {
    checked(g, Action::AnnounceCast { card: id });
    if let Some(mode) = mode {
        reject(g, Action::ChooseMode { mode: 3 });
        checked(g, Action::ChooseMode { mode });
    }
    for &target in targets {
        for _ in 0..3 {
            if g.legal_actions().contains(&Action::ChooseTarget { target }) {
                break;
            }
            checked(g, Action::FinishTargets);
        }
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets))
            .or_else(|| {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. }))
            })
            .unwrap();
        checked(g, a);
    }
    panic!("unpaid");
}
#[test]
fn exclusion_tappers_pay_and_recheck_live_types_and_changeling() {
    for p in [Player::P0, Player::P1] {
        for name in ["Coeurl", "Sterling Keykeeper"] {
            let mut g = setup(p);
            let source = named(&mut g, p, name, Zone::Hand);
            cast(&mut g, source, None, &[]);
            settle(&mut g);
            g.state.objects[source as usize].summoning_sick = false;
            let good = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let forbidden = named(
                &mut g,
                p.other(),
                if name == "Coeurl" {
                    "Fear of Surveillance"
                } else {
                    "Alacrian Jaguar"
                },
                Zone::Battlefield,
            );
            assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::Activate{card,target:Some(Target::Card(t)),..}if *card==source&&*t==forbidden)));
            if name == "Sterling Keykeeper" {
                let changeling = named(&mut g, p.other(), "Three Tree Mascot", Zone::Battlefield);
                assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::Activate{card,target:Some(Target::Card(t)),..}if *card==source&&*t==changeling)));
            }
            let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,target:Some(Target::Card(t)),..}if *card==source&&*t==good)).unwrap();
            let mut underpaid = a.clone();
            if let Action::Activate { payment, .. } = &mut underpaid {
                *payment = [0; 6];
            }
            reject(&mut g, underpaid);
            let mana = g.state.players[p.index()].mana;
            checked(&mut g, a);
            assert!(g.state.objects[source as usize].tapped);
            assert_ne!(g.state.players[p.index()].mana, mana);
            g.move_object(good, Zone::Hand);
            g.move_object(good, Zone::Battlefield);
            settle(&mut g);
            assert!(!g.state.objects[good as usize].tapped);
        }
    }
}
#[test]
fn rock_optional_entry_and_gorilla_actual_sacrifice_respect_noncreature_kinds() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let artifact = named(&mut g, p.other(), "Basilisk Collar", Zone::Battlefield);
        let rock = named(&mut g, p, "Rock Soldiers", Zone::Hand);
        cast(&mut g, rock, None, &[]);
        settle_on(&mut g, Some(Target::Card(artifact)));
        assert_eq!(g.state.objects[artifact as usize].zone, Zone::Graveyard);
        let gorilla = named(&mut g, p, "Guerrilla Gorilla", Zone::Hand);
        cast(&mut g, gorilla, None, &[]);
        settle(&mut g);
        let vehicle = named(&mut g, p.other(), "Air Response Unit", Zone::Battlefield);
        let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,target:Some(Target::Card(t)),..}if *card==gorilla&&*t==vehicle)).unwrap();
        checked(&mut g, a);
        assert_eq!(g.state.objects[gorilla as usize].zone, Zone::Graveyard);
        g.state.crew = Some(Box::new(super::crew::CrewState {
            animated: vec![g.object_ref(vehicle)],
            ..Default::default()
        }));
        settle(&mut g);
        assert!(g.on_battlefield(vehicle));
        let rock2 = named(&mut g, p, "Rock Soldiers", Zone::Hand);
        cast(&mut g, rock2, None, &[]);
        settle(&mut g);
        assert!(g.on_battlefield(rock2));
    }
}
#[test]
fn surge_each_mode_and_both_pay_once_with_partial_resolution() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            for animate in [false, true] {
                let mut g = setup(p);
                let creature = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
                let artifact = named(&mut g, p.other(), "Air Response Unit", Zone::Battlefield);
                let spell = named(&mut g, p, "Overwhelming Surge", Zone::Hand);
                let targets = match mode {
                    0 => vec![Target::Card(creature)],
                    1 => vec![Target::Card(artifact)],
                    _ => vec![Target::Card(creature), Target::Card(artifact)],
                };
                let before = g.state.players[p.index()].mana;
                cast(&mut g, spell, Some(mode), &targets);
                assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
                assert_ne!(before, g.state.players[p.index()].mana);
                if animate {
                    g.state.crew = Some(Box::new(super::crew::CrewState {
                        animated: vec![g.object_ref(artifact)],
                        ..Default::default()
                    }));
                }
                settle(&mut g);
                assert_eq!(
                    g.state.objects[creature as usize].zone,
                    if mode != 1 {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    }
                );
                assert_eq!(
                    g.state.objects[artifact as usize].zone,
                    if mode != 0 && !animate {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    }
                );
            }
        }
    }
}
#[test]
fn decoy_all_combinations_return_exact_subtypes_and_allow_same_changeling_twice() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = setup(p);
            let villain = named(&mut g, p, "Common Crook", Zone::Graveyard);
            let hero = named(&mut g, p, "Agent of Atlas", Zone::Graveyard);
            let spell = named(&mut g, p, "Decoy Ploy", Zone::Hand);
            let targets = match mode {
                0 => vec![Target::Card(villain)],
                1 => vec![Target::Card(hero)],
                _ => vec![Target::Card(villain), Target::Card(hero)],
            };
            cast(&mut g, spell, Some(mode), &targets);
            settle(&mut g);
            assert_eq!(
                g.state.objects[villain as usize].zone,
                if mode != 1 {
                    Zone::Hand
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.state.objects[hero as usize].zone,
                if mode != 0 {
                    Zone::Hand
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let both = named(&mut g, p, "Three Tree Mascot", Zone::Graveyard);
        let spell = named(&mut g, p, "Decoy Ploy", Zone::Hand);
        let old = g.object_ref(both);
        cast(
            &mut g,
            spell,
            Some(2),
            &[Target::Card(both), Target::Card(both)],
        );
        settle(&mut g);
        assert_eq!(g.state.objects[both as usize].zone, Zone::Hand);
        assert_eq!(g.object_ref(both).generation, old.generation + 1);
    }
}
#[test]
fn acrobatics_counter_and_tap_modes_have_independent_targets() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = setup(p.other());
            let body = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let other = named(&mut g, p.other(), "Grizzly Bears", Zone::Hand);
            cast(&mut g, other, None, &[]);
            g.state.priority = p;
            g.state.players[p.index()].mana = [30; 6];
            let spell = named(&mut g, p, "Amazing Acrobatics", Zone::Hand);
            let targets = match mode {
                0 => vec![Target::Spell(other)],
                1 => vec![Target::Card(body)],
                _ => vec![Target::Spell(other), Target::Card(body)],
            };
            cast(&mut g, spell, Some(mode), &targets);
            settle(&mut g);
            assert_eq!(
                g.state.objects[other as usize].zone,
                if mode != 1 {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(g.state.objects[body as usize].tapped, mode != 0);
        }
    }
}

#[test]
fn gorilla_sorcery_timing_and_opponent_hexproof_are_enforced_before_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Guerrilla Gorilla", Zone::Battlefield);
        let target = named(&mut g, p.other(), "Basilisk Collar", Zone::Battlefield);
        let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,target:Some(Target::Card(t)),..}if *card==source&&*t==target)).unwrap();
        g.state.active = p.other();
        reject(&mut g, action.clone());
        assert!(g.on_battlefield(source));
        g.state.active = p;
        g.state.objects[target as usize].gained_keywords = Keywords::HEXPROOF;
        reject(&mut g, action.clone());
        assert!(g.on_battlefield(source));
        g.state.objects[target as usize].controller = p;
        checked(&mut g, action);
        assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.objects[target as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[target as usize].owner, p.other());
    }
}
