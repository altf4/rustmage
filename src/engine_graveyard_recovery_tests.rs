//! Whole normal recovery bodies use existing typed runtime and owned rollback.
use super::tests::{scenario, spawn};
use super::*;
const NAMES: [&str; 19] = [
    "Archenemy's Charm",
    "Auroral Procession",
    "Barkform Harvester",
    "Coati Scavenger",
    "Dundoolin Weaver",
    "Elvish Regrower",
    "Fateshaper Aspirant",
    "Fight On!",
    "Hazel's Nocturne",
    "Macabre Waltz",
    "Morcant's Loyalist",
    "Oltec Archaeologists",
    "Pull from the Grave",
    "Restore with Empathy",
    "Rydia's Return",
    "Tomb Trawler",
    "Tombstone, Career Criminal",
    "True Ancestry",
    "White Widow, Free Agent",
];
fn act(g: &mut Game, a: Action) {
    let actor = g.actor().unwrap();
    let saved = g.snapshot();
    let rev = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|x| *x == a)
        .unwrap_or_else(|| panic!("illegal {a:?}, {:?}", g.observe(actor)));
    assert_eq!(g.apply(actor.other(), a.clone()), Err(Error::WrongActor));
    assert_eq!(saved, g.snapshot());
    let mut child = {
        let parent = g.clone();
        let c = parent.clone();
        drop(parent);
        c
    };
    child.apply_index(actor, rev, index).unwrap();
    g.apply(actor, a.clone()).unwrap();
    assert_eq!(g.snapshot(), child.snapshot());
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(viewer)).unwrap(),
            serde_json::to_value(child.observe(viewer)).unwrap()
        );
    }
    let done = g.snapshot();
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(actor, rev, index), Err(Error::StaleRevision));
    g.apply(actor, a).unwrap();
    assert_eq!(g.snapshot(), done);
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for player in &mut g.state.players {
        player.mana = [30; 6];
    }
    g
}
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn drive(g: &mut Game, target: CardId, mode: u16) {
    for _ in 0..200 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.announcing.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let a = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .iter()
                .find(|a| **a == Action::ChooseMode { mode })
                .or_else(|| {
                    actions.iter().find(|a| {
                        **a == Action::ChooseTarget {
                            target: Target::Card(target),
                        }
                    })
                })
                 .or_else(|| actions.iter().find(|a| matches!(a, Action::Discard { card } if g.state.objects[*card as usize].card.name()=="Forest")))
                .or_else(|| actions.iter().find(|a| matches!(a, Action::FinishTargets)))
                .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                .expect("continuation")
                .clone()
        };
        act(g, a);
    }
    panic!("unsettled {:?}", g.observe(g.actor().unwrap()));
}
fn priority(g: &mut Game, p: Player) {
    for _ in 0..10 {
        if g.actor() == Some(p) && g.state.pending == Pending::Priority {
            return;
        }
        act(g, Action::Pass);
    }
    panic!("priority");
}
#[test]
fn eighteen_whole_paid_normal_bodies_recover_typed_targets_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            if i == 10 {
                assert!(!Card::from_name(name).unwrap().is_supported());
                continue;
            }
            let mut g = setup(p);
            let target_name = if i == 6 {
                "Tombstone, Career Criminal"
            } else {
                "Barkform Harvester"
            };
            let target = named(&mut g, p, target_name, Zone::Graveyard);
            let before = g.object_ref(target);
            for _ in 0..4 {
                named(&mut g, p, "Forest", Zone::Graveyard);
            }
            for _ in 0..3 {
                named(&mut g, p, "Bear Cub", Zone::Battlefield);
            }
            named(&mut g, p, "Forest", Zone::Hand);
            named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            let source = named(&mut g, p, name, Zone::Hand);
            let mana: u32 = g.state.players[p.index()].mana.iter().sum();
            let (colored, generic) = Card::from_name(name).unwrap().cost();
            let cost: u32 = colored.iter().sum::<u32>() + generic;
            let mode = if [0, 14, 18].contains(&i) { 1 } else { 0 };
            let life = g.state.players[p.index()].life;
            let other_life = g.state.players[p.other().index()].life;
            act(&mut g, Action::AnnounceCast { card: source });
            drive(&mut g, target, mode);
            assert_eq!(
                mana - g.state.players[p.index()].mana.iter().sum::<u32>(),
                cost,
                "{name}"
            );
            if [2, 15].contains(&i) {
                priority(&mut g, p);
                let mana: u32 = g.state.players[p.index()].mana.iter().sum();
                act(
                    &mut g,
                    Action::AnnounceActivate {
                        card: source,
                        ability: 0,
                        counter_payment: Counters::default(),
                    },
                );
                drive(&mut g, target, 0);
                assert_eq!(
                    mana - g.state.players[p.index()].mana.iter().sum::<u32>(),
                    2
                );
                assert_eq!(g.state.players[p.index()].library.first(), Some(&target));
                assert_eq!(g.state.objects[target as usize].zone, Zone::Library);
            } else {
                assert_eq!(g.state.objects[target as usize].zone, Zone::Hand, "{name}");
            }
            assert_ne!(before, g.object_ref(target), "{name}");
            if [8, 12].contains(&i) {
                assert_eq!(g.state.players[p.index()].life, life + 2);
            }
            if i == 8 {
                assert_eq!(g.state.players[p.other().index()].life, other_life - 2);
            }
            if i == 13 {
                assert_eq!(g.state.players[p.index()].life, life + 4);
            }
            if i == 17 {
                assert!(g.state.objects.iter().any(|o| o.owner == p
                    && o.zone == Zone::Battlefield
                    && o.card.is_token()
                    && o.card.has_subtype("Clue")));
            }
        }
    }
}
fn rejected(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
#[test]
fn all_new_modal_alternatives_resolve_complete_paid_bodies_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (name, mode) in [
            ("Archenemy's Charm", 0),
            ("Archenemy's Charm", 2),
            ("Fateshaper Aspirant", 1),
            ("Oltec Archaeologists", 1),
            ("Rydia's Return", 0),
            ("White Widow, Free Agent", 0),
        ] {
            let mut g = setup(p);
            let target = named(
                &mut g,
                if name == "Archenemy's Charm" && mode == 0 {
                    p.other()
                } else {
                    p
                },
                "Bear Cub",
                Zone::Battlefield,
            );
            let source = named(&mut g, p, name, Zone::Hand);
            let mana: u32 = g.state.players[p.index()].mana.iter().sum();
            act(&mut g, Action::AnnounceCast { card: source });
            drive(&mut g, target, mode);
            assert!(g.state.players[p.index()].mana.iter().sum::<u32>() < mana);
            match name {
                "Archenemy's Charm" if mode == 0 => {
                    assert_eq!(g.state.objects[target as usize].zone, Zone::Exile)
                }
                "Archenemy's Charm" => {
                    assert_eq!(g.state.objects[target as usize].counters.plus_one, 2);
                    assert!(g.keywords(target).contains(Keywords::LIFELINK));
                }
                "Fateshaper Aspirant" => {
                    assert_eq!(g.state.objects[target as usize].counters.plus_one, 1);
                    assert!(g.keywords(target).contains(Keywords::VIGILANCE));
                    assert!(g.keywords(target).contains(Keywords::INDESTRUCTIBLE));
                }
                "Oltec Archaeologists" => assert!(g.observe(p).library_choice.is_none()),
                "Rydia's Return" => {
                    assert_eq!((g.stats(target).power, g.stats(target).toughness), (5, 5))
                }
                "White Widow, Free Agent" => {
                    assert_eq!(g.state.objects[target as usize].counters.plus_one, 1)
                }
                _ => unreachable!(),
            }
        }
    }
}
fn announce(g: &mut Game, source: CardId, mode: u16, targets: &[CardId]) {
    act(g, Action::AnnounceCast { card: source });
    let mut selected = 0;
    for _ in 0..40 {
        if g.state.announcing.is_none()
            && g.state
                .stack
                .iter()
                .any(|s| s.kind == StackKind::Spell && s.card == source)
        {
            return;
        }
        let actions = g.legal_actions();
        let target = targets.get(selected).map(|&card| Action::ChooseTarget {
            target: Target::Card(card),
        });
        let a = actions
            .iter()
            .find(|a| **a == Action::ChooseMode { mode })
            .or_else(|| actions.iter().find(|a| Some(*a) == target.as_ref()))
            .or_else(|| actions.iter().find(|a| matches!(a, Action::FinishTargets)))
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .expect("announcement")
            .clone();
        if matches!(a, Action::ChooseTarget { .. }) {
            selected += 1;
        }
        act(g, a);
    }
    panic!("announcement");
}
#[test]
fn optional_two_targets_partially_resolve_and_stale_incarnations_do_not_recover() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let b = named(&mut g, p, "Barkform Harvester", Zone::Graveyard);
        let source = named(&mut g, p, "Hazel's Nocturne", Zone::Hand);
        let life = g.state.players[p.index()].life;
        announce(&mut g, source, 0, &[a, b]);
        let stale = g.object_ref(a);
        g.move_object(a, Zone::Hand);
        g.move_object(a, Zone::Graveyard);
        assert_ne!(g.object_ref(a), stale);
        drive(&mut g, b, 0);
        assert_eq!(g.state.objects[a as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[b as usize].zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].life, life + 2);
    }
}
#[test]
fn macabre_zero_targets_still_discards_but_all_illegal_targets_fizzle_every_effect() {
    for p in [Player::P0, Player::P1] {
        for count in [0, 1] {
            let mut g = setup(p);
            let target = named(&mut g, p, "Bear Cub", Zone::Graveyard);
            let discard = named(&mut g, p, "Forest", Zone::Hand);
            let source = named(&mut g, p, "Macabre Waltz", Zone::Hand);
            announce(
                &mut g,
                source,
                0,
                if count == 0 {
                    &[]
                } else {
                    std::slice::from_ref(&target)
                },
            );
            if count == 1 {
                g.move_object(target, Zone::Exile);
            }
            drive(&mut g, target, 0);
            assert_eq!(
                g.state.objects[discard as usize].zone,
                if count == 0 {
                    Zone::Graveyard
                } else {
                    Zone::Hand
                }
            );
        }
    }
}
#[test]
fn graveyard_target_relations_cardinality_and_exact_types_are_enforced() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let foreign = named(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
        let battlefield = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let instant = named(&mut g, p, "Shock", Zone::Graveyard);
        let a = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let b = named(&mut g, p, "Barkform Harvester", Zone::Graveyard);
        let c = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let source = named(&mut g, p, "Fight On!", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: source });
        for _ in 0..20 {
            if g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseTarget { .. }))
            {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap();
            act(&mut g, action);
        }
        for card in [foreign, battlefield, instant] {
            rejected(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(card),
                },
            );
        }
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(a),
            },
        );
        rejected(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(a),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(b),
            },
        );
        rejected(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(c),
            },
        );
        drive(&mut g, b, 0);
        assert_eq!(g.state.objects[a as usize].zone, Zone::Hand);
        assert_eq!(g.state.objects[b as usize].zone, Zone::Hand);
        assert_eq!(g.state.objects[c as usize].zone, Zone::Graveyard);
    }
}
