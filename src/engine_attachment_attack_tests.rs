use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 10] = [
    "Baseball Bat",
    "Captain America's Shield",
    "Stormbeacon Blade",
    "Thunder Lasso",
    "Ultima Weapon",
    "Candlestick",
    "Diamond Pick-Axe",
    "Medic's Kitesail",
    "Pirate Hat",
    "Web-Shooters",
];
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    g.state.players[p.index()].mana = [30; 6];
    g
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let actor = g.actor().unwrap();
    let rev = g.revision();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(fork.snapshot(), expected);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(actor, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn reject(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), rev);
}
fn choose(g: &mut Game, target: Option<CardId>) {
    let actions = g.legal_actions();
    let a = target
        .and_then(|id| {
            actions.iter().find(|a| {
                **a == Action::ChooseTarget {
                    target: Target::Card(id),
                }
            })
        })
        .or_else(|| {
            actions.iter().find(|a| {
                matches!(
                    a,
                    Action::PayCosts { .. }
                        | Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::ChooseTarget { .. }
                        | Action::ArrangeLibrary { .. }
                        | Action::Discard { .. }
                )
            })
        })
        .expect("continuation")
        .clone();
    replay(g, a);
}
fn announce(g: &mut Game, target: Option<CardId>) {
    g.prepare_triggers();
    for _ in 0..100 {
        if g.state.announcing.is_none()
            && g.state.ordering_triggers.is_empty()
            && g.state.pending == Pending::Priority
        {
            return;
        }
        choose(g, target)
    }
    panic!("unannounced")
}
fn settle(g: &mut Game, target: Option<CardId>) {
    g.prepare_triggers();
    for _ in 0..200 {
        if g.state.stack.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
            && g.state.ordering_triggers.is_empty()
            && g.state.pending == Pending::Priority
        {
            return;
        }
        choose(g, target)
    }
    panic!("unsettled")
}
fn attach(g: &mut Game, e: CardId, c: CardId) {
    assert!(g.attach_equipment(g.object_ref(e), g.object_ref(c)));
}
fn attack(g: &mut Game, p: Player, ids: &[CardId]) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    for &id in ids {
        replay(g, Action::Attack { card: id })
    }
    replay(g, Action::FinishAttackers);
}
fn packet(g: &Game) -> SpellView {
    g.state
        .announcing
        .as_ref()
        .map(|a| &a.item)
        .or_else(|| g.state.stack.last())
        .or_else(|| g.state.ordering_triggers.last())
        .or_else(|| g.state.pending_triggers.last())
        .unwrap()
        .clone()
}
fn paid_equip(g: &mut Game, p: Player, e: CardId, c: CardId, index: usize) {
    g.state.players[p.index()].mana = [30; 6];
    let cost = g.state.objects[e as usize].card.activations()[index].cost;
    let mut payment = cost.colored;
    payment[5] += cost.generic;
    replay(
        g,
        Action::Activate {
            card: e,
            ability: index as u16,
            target: Some(Target::Card(c)),
            payment,
            counter_payment: Counters::default(),
        },
    );
    settle(g, Some(c));
    assert_eq!(g.object_view(e).attached_to, Some(g.object_ref(c)));
}
#[test]
fn ten_complete_paid_casts_equip_static_characteristics_attack_effects_and_residuals_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let extras = [
                named(&mut g, p, "Bear Cub", Zone::Battlefield),
                named(&mut g, p, "Bear Cub", Zone::Battlefield),
            ];
            let e = named(&mut g, p, name, Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: e });
            settle(&mut g, Some(bear));
            assert_eq!(g.object_view(e).zone, Zone::Battlefield);
            if i == 0 || i == 3 {
                assert_eq!(g.object_view(e).attached_to, Some(g.object_ref(bear)));
            } else {
                assert_eq!(g.object_view(e).attached_to, None);
                let index = g.state.objects[e as usize].card.activations().len() - 1;
                paid_equip(&mut g, p, e, bear, index);
            }
            let (power, toughness) = [
                (1, 1),
                (0, 8),
                (3, 0),
                (1, 1),
                (7, 7),
                (1, 1),
                (1, 1),
                (1, 0),
                (1, 1),
                (1, 1),
            ][i];
            assert_eq!(
                g.stats(bear),
                Stats {
                    power: 2 + power,
                    toughness: 2 + toughness
                }
            );
            if i == 1 {
                assert!(g.keywords(bear).contains(Keywords::VIGILANCE));
            }
            if i == 7 {
                assert!(g.keywords(bear).contains(Keywords::FLYING));
            }
            if i == 9 {
                assert!(g.keywords(bear).contains(Keywords::REACH));
            }
            if i == 1 || i == 6 {
                assert!(g.keywords(e).contains(Keywords::INDESTRUCTIBLE));
            }
            let life = g.state.players[p.index()].life;
            let drawn = g.state.players[p.index()].cards_drawn_this_turn;
            let discarded = g.state.players[p.index()].cards_discarded_this_turn;
            attack(
                &mut g,
                p,
                &if i == 2 {
                    vec![bear, extras[0], extras[1]]
                } else {
                    vec![bear]
                },
            );
            let item = packet(&g);
            let event = item.attachment_trigger.as_ref().unwrap();
            assert_eq!(event.grantor, g.object_ref(e));
            assert_eq!(event.attacker, g.object_ref(bear));
            assert_eq!(event.granted, i >= 5);
            assert_eq!(event.defender, p.other());
            assert_eq!(
                item.source_card,
                g.state.objects[if i >= 5 { bear } else { e } as usize].card
            );
            assert_eq!(item.object, g.object_ref(if i >= 5 { bear } else { e }));
            assert_eq!(item.controller, p);
            assert_eq!(g.object_view(bear).tapped, i != 1);
            settle(&mut g, Some(foe));
            match i {
                0 | 1 | 3 | 9 => assert!(g.object_view(foe).tapped),
                2 => assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, drawn + 1),
                4 => assert_eq!(g.object_view(foe).zone, Zone::Graveyard),
                5 => assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, drawn),
                6 => assert!(g.state.objects.iter().any(|o| {
                    o.owner == p
                        && o.zone == Zone::Battlefield
                        && o.card
                            .token_info()
                            .is_some_and(|t| t.subtypes == "Treasure")
                })),
                7 => assert_eq!(g.state.players[p.index()].life, life + 1),
                8 => {
                    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, drawn + 1);
                    assert_eq!(
                        g.state.players[p.index()].cards_discarded_this_turn,
                        discarded + 1
                    )
                }
                _ => unreachable!(),
            }
            if i == 6 {
                let treasure = g
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .find(|(_, o)| {
                        o.owner == p
                            && o.zone == Zone::Battlefield
                            && o.card
                                .token_info()
                                .is_some_and(|t| t.subtypes == "Treasure")
                    })
                    .unwrap()
                    .0 as CardId;
                let mana = g.state.players[p.index()]
                    .mana
                    .iter()
                    .map(|&n| u64::from(n))
                    .sum::<u64>();
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| {
                        matches!(a,
                    Action::ActivateMana { card, .. } if *card == treasure)
                    })
                    .unwrap();
                replay(&mut g, action);
                assert_ne!(g.object_view(treasure).zone, Zone::Battlefield);
                assert_eq!(
                    g.state.players[p.index()]
                        .mana
                        .iter()
                        .map(|&n| u64::from(n))
                        .sum::<u64>(),
                    mana + 1
                );
            }
            if i == 5 {
                g.state.players[p.index()].mana = [30; 6];
                let index = g.state.objects[e as usize]
                    .card
                    .activations()
                    .iter()
                    .position(|a| a.cost.sacrifice)
                    .unwrap();
                replay(
                    &mut g,
                    Action::Activate {
                        card: e,
                        ability: index as u16,
                        target: None,
                        payment: [0, 0, 0, 0, 0, 2],
                        counter_payment: Counters::default(),
                    },
                );
                settle(&mut g, None);
                assert_eq!(g.object_view(e).zone, Zone::Graveyard);
                assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, drawn + 1);
            }
        }
    }
}
#[test]
fn attachment_qualification_happens_once_at_complete_declaration_not_tap_or_partial_selection() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let e = named(&mut g, p, "Medic's Kitesail", Zone::Battlefield);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        attach(&mut g, e, a);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        replay(&mut g, Action::Attack { card: a });
        assert!(g.state.pending_triggers.is_empty());
        attach(&mut g, e, b);
        replay(&mut g, Action::FinishAttackers);
        assert!(g.state.stack.is_empty());
        assert!(g.state.pending_triggers.is_empty());
        g.state.attackers.clear();
        g.state.objects[a as usize].tapped = false;
        g.state.objects[b as usize].tapped = false;
        attack(&mut g, p, &[b]);
        assert!(packet(&g).attachment_trigger.is_some());
        attach(&mut g, e, a);
        settle(&mut g, None);
        assert_eq!(g.state.players[p.index()].life, 21);
    }
}
#[test]
fn duplicate_grants_have_truthful_creature_sources_independent_ids_and_survive_both_departures() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        for _ in 0..2 {
            let e = named(&mut g, p, "Medic's Kitesail", Zone::Battlefield);
            attach(&mut g, e, c);
        }
        attack(&mut g, p, &[c]);
        announce(&mut g, None);
        assert_eq!(g.state.stack.len(), 2);
        assert_ne!(g.state.stack[0].id, g.state.stack[1].id);
        assert_ne!(
            g.state.stack[0]
                .attachment_trigger
                .as_ref()
                .unwrap()
                .grantor,
            g.state.stack[1]
                .attachment_trigger
                .as_ref()
                .unwrap()
                .grantor
        );
        for item in &g.state.stack {
            assert_eq!(item.source_card, Card::from_name("Bear Cub").unwrap());
            assert_eq!(item.object, g.object_ref(c));
        }
        let sources = g.state.attachment_sources.clone();
        let replacement = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        for e in sources {
            let old = g.object_ref(e);
            g.move_object(e, Zone::Graveyard);
            g.move_object_raw(e, Zone::Battlefield);
            attach(&mut g, e, replacement);
            assert_ne!(g.object_ref(e), old);
            assert!(g.state.stack.iter().any(|item| {
                item.attachment_trigger
                    .as_ref()
                    .is_some_and(|event| event.grantor == old)
            }));
        }
        g.move_object(c, Zone::Graveyard);
        g.move_object_raw(c, Zone::Battlefield);
        settle(&mut g, None);
        assert_eq!(g.state.players[p.index()].life, 22);
    }
}
#[test]
fn defender_is_captured_independently_of_equipment_controller_and_later_links() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let e = named(
            &mut g,
            p.other(),
            "Captain America's Shield",
            Zone::Battlefield,
        );
        let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        attach(&mut g, e, c);
        let kite = named(&mut g, p, "Medic's Kitesail", Zone::Battlefield);
        attach(&mut g, kite, c);
        attack(&mut g, p, &[c]);
        assert_eq!(packet(&g).controller, p.other());
        assert!(g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(target)
        }));
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(c),
            },
        );
        announce(&mut g, Some(target));
        assert_eq!(g.state.stack.len(), 2);
        assert_eq!(g.state.stack[0].controller, p);
        assert_eq!(g.state.stack[1].controller, p.other());
        g.move_object(e, Zone::Graveyard);
        settle(&mut g, Some(target));
        assert!(g.object_view(target).tapped);
        assert_eq!(g.state.players[p.index()].life, 21);
    }
}

#[test]
fn pirate_equip_alternative_requires_owned_pirate_and_ordinary_option_remains_available() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hat = named(&mut g, p, "Pirate Hat", Zone::Battlefield);
        let pirate = named(&mut g, p, "Three Tree Mascot", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Three Tree Mascot", Zone::Battlefield);
        assert_eq!(g.state.objects[hat as usize].card.activations().len(), 2);
        let cheap = |target| Action::Activate {
            card: hat,
            ability: 0,
            target: Some(Target::Card(target)),
            payment: [0, 0, 0, 0, 0, 1],
            counter_payment: Counters::default(),
        };
        reject(&mut g, cheap(bear));
        reject(&mut g, cheap(foe));
        paid_equip(&mut g, p, hat, pirate, 0);
        paid_equip(&mut g, p, hat, bear, 1);
        g.state.step = Step::BeginCombat;
        reject(&mut g, cheap(pirate));
    }
}
#[test]
fn optional_and_mandatory_attachment_targets_and_all_target_fizzles_preserve_full_semantics() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "Baseball Bat",
            "Thunder Lasso",
            "Ultima Weapon",
            "Web-Shooters",
        ] {
            let mut g = setup(p);
            let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let e = named(&mut g, p, name, Zone::Battlefield);
            attach(&mut g, e, c);
            attack(&mut g, p, &[c]);
            if name == "Baseball Bat" {
                announce(&mut g, None);
                assert!(g.state.stack[0].targets.is_empty());
                settle(&mut g, None);
            } else {
                assert!(g.state.stack.is_empty());
                assert!(g.state.announcing.is_none());
            }
            let mut g = setup(p);
            let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let e = named(&mut g, p, name, Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            attach(&mut g, e, c);
            attack(&mut g, p, &[c]);
            announce(&mut g, Some(foe));
            let old = g.object_ref(foe);
            g.move_object(foe, Zone::Graveyard);
            g.move_object_raw(foe, Zone::Battlefield);
            assert_ne!(g.object_ref(foe), old);
            settle(&mut g, None);
            assert!(!g.object_view(foe).tapped);
            assert_eq!(g.object_view(foe).zone, Zone::Battlefield);
        }
    }
}
#[test]
fn stormbeacon_qualifies_attachment_at_declaration_but_counts_owned_attackers_live() {
    for p in [Player::P0, Player::P1] {
        for depart in [false, true] {
            let mut g = setup(p);
            let cs = [
                named(&mut g, p, "Bear Cub", Zone::Battlefield),
                named(&mut g, p, "Bear Cub", Zone::Battlefield),
                named(&mut g, p, "Bear Cub", Zone::Battlefield),
            ];
            let e = named(&mut g, p, "Stormbeacon Blade", Zone::Battlefield);
            attach(&mut g, e, cs[0]);
            attack(&mut g, p, &cs);
            announce(&mut g, None);
            g.move_object(e, Zone::Graveyard);
            if depart {
                g.move_object(cs[2], Zone::Graveyard);
            }
            settle(&mut g, None);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                u64::from(!depart)
            );
        }
        let mut g = setup(p);
        let cs = [
            named(&mut g, p, "Bear Cub", Zone::Battlefield),
            named(&mut g, p, "Bear Cub", Zone::Battlefield),
            named(&mut g, p, "Bear Cub", Zone::Battlefield),
        ];
        let e = named(&mut g, p.other(), "Stormbeacon Blade", Zone::Battlefield);
        attach(&mut g, e, cs[0]);
        attack(&mut g, p, &cs);
        settle(&mut g, None);
        assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 0);
    }
}
#[test]
fn private_surveil_and_loot_keep_actual_recipient_actor_after_grantor_or_recipient_departure() {
    for p in [Player::P0, Player::P1] {
        for name in ["Candlestick", "Pirate Hat"] {
            let mut g = setup(p);
            let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let e = named(&mut g, p.other(), name, Zone::Battlefield);
            attach(&mut g, e, c);
            attack(&mut g, p, &[c]);
            announce(&mut g, None);
            g.move_object(e, Zone::Graveyard);
            g.move_object(c, Zone::Graveyard);
            replay(&mut g, Action::Pass);
            replay(&mut g, Action::Pass);
            assert_eq!(g.actor(), Some(p));
            let private = g.observe(p);
            let hidden = g.observe(p.other());
            assert!(hidden.actions.is_empty());
            assert!(
                !hidden
                    .objects
                    .iter()
                    .any(|o| o.zone == Zone::Hand && o.owner == p)
            );
            if name == "Candlestick" {
                assert!(private.library_choice.is_some());
                assert!(hidden.library_choice.is_none());
                assert_eq!(private.library_choice.as_ref().unwrap().remaining.len(), 2);
            } else {
                assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
                assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 0);
                assert!(matches!(g.state.pending, Pending::EffectDiscard { .. }));
            }
            let saved = g.snapshot();
            let action = g.legal_actions()[0].clone();
            let mut fork = g.clone();
            replay(&mut fork, action);
            assert_eq!(g.snapshot(), saved);
            settle(&mut g, None);
            assert_eq!(
                g.state.players[p.index()].cards_discarded_this_turn,
                u64::from(name == "Pirate Hat")
            );
        }
    }
}
#[test]
fn attack_stack_capacity_and_late_granted_life_overflow_restore_all_owned_packets_and_rng() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let e = named(&mut g, p, "Medic's Kitesail", Zone::Battlefield);
        attach(&mut g, e, c);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        replay(&mut g, Action::Attack { card: c });
        g.state.next_stack_id = u64::MAX;
        let saved = g.snapshot();
        let rev = g.revision();
        assert!(matches!(
            g.apply(p, Action::FinishAttackers),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), rev);
        let mut g = setup(p);
        let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let e = named(&mut g, p, "Medic's Kitesail", Zone::Battlefield);
        attach(&mut g, e, c);
        attack(&mut g, p, &[c]);
        announce(&mut g, None);
        g.state.players[p.index()].life = i128::MAX;
        replay(&mut g, Action::Pass);
        let saved = g.snapshot();
        let rev = g.revision();
        assert!(matches!(
            g.apply(g.actor().unwrap(), Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), rev);
    }
}
#[test]
fn indestructible_is_intrinsic_to_the_equipment_and_not_granted_to_the_creature() {
    for p in [Player::P0, Player::P1] {
        for name in ["Captain America's Shield", "Diamond Pick-Axe"] {
            let mut g = setup(p);
            let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let e = named(&mut g, p, name, Zone::Battlefield);
            attach(&mut g, e, c);
            assert!(!g.keywords(c).contains(Keywords::INDESTRUCTIBLE));
            let d = named(&mut g, p, "Disenchant", Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: d });
            settle(&mut g, Some(e));
            assert_eq!(g.object_view(e).zone, Zone::Battlefield);
            assert_eq!(g.object_view(e).attached_to, Some(g.object_ref(c)));
        }
    }
}

#[test]
fn printed_and_multiple_granted_abilities_keep_distinct_immutable_program_owners() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = named(&mut g, p, "Pulse Tracker", Zone::Battlefield);
        for _ in 0..2 {
            let e = named(&mut g, p, "Medic's Kitesail", Zone::Battlefield);
            attach(&mut g, e, c);
        }
        attack(&mut g, p, &[c]);
        announce(&mut g, None);
        assert_eq!(g.state.stack.len(), 3);
        assert_eq!(
            g.state
                .stack
                .iter()
                .filter(|s| s.attachment_trigger.is_some())
                .count(),
            2
        );
        for item in &g.state.stack {
            assert_eq!(item.source_card, Card::from_name("Pulse Tracker").unwrap());
            assert_eq!(item.object, g.object_ref(c));
        }
        settle(&mut g, None);
        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.state.players[p.other().index()].life, 19);
    }
}
#[test]
fn granted_tap_targets_obey_ward_hexproof_wrong_actors_and_late_target_legality() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let e = named(&mut g, p, "Web-Shooters", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Hoverstone Pilgrim", Zone::Battlefield);
            let protected = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[protected as usize].gained_keywords = Keywords::HEXPROOF;
            attach(&mut g, e, c);
            attack(&mut g, p, &[c]);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(protected),
                },
            );
            let saved = g.snapshot();
            let rev = g.revision();
            assert_eq!(
                g.apply(
                    p.other(),
                    Action::ChooseTarget {
                        target: Target::Card(foe)
                    }
                ),
                Err(Error::WrongActor)
            );
            assert_eq!(g.snapshot(), saved);
            assert_eq!(g.revision(), rev);
            announce(&mut g, Some(foe));
            for _ in 0..30 {
                if g.observe(g.actor().unwrap()).resolution_payment.is_some() {
                    break;
                }
                choose(&mut g, None);
            }
            assert!(g.observe(g.actor().unwrap()).resolution_payment.is_some());
            if pay {
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayResolutionMana { .. }))
                    .unwrap();
                replay(&mut g, action);
            } else {
                replay(&mut g, Action::DeclineResolutionMana);
            }
            settle(&mut g, None);
            assert_eq!(g.object_view(foe).tapped, pay);
        }
        let mut g = setup(p);
        let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let e = named(&mut g, p, "Web-Shooters", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        attach(&mut g, e, c);
        attack(&mut g, p, &[c]);
        announce(&mut g, Some(foe));
        g.state.objects[foe as usize].gained_keywords = Keywords::HEXPROOF;
        settle(&mut g, None);
        assert!(!g.object_view(foe).tapped);
    }
}

#[test]
fn initialized_bytes_include_every_owned_attack_context_and_public_snapshots_keep_provenance() {
    let mut g = setup(Player::P0);
    let c = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    for name in ["Medic's Kitesail", "Baseball Bat"] {
        let e = named(&mut g, Player::P0, name, Zone::Battlefield);
        attach(&mut g, e, c);
    }
    let foe = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    attack(&mut g, Player::P0, &[c]);
    announce(&mut g, Some(foe));
    let mut without = g.clone();
    let count = without
        .state
        .stack
        .iter_mut()
        .filter_map(|s| s.attachment_trigger.take())
        .count();
    assert_eq!(count, 2);
    assert_eq!(
        g.state_bytes() - without.state_bytes(),
        count * std::mem::size_of::<AttachmentTriggerView>()
    );
    for p in [Player::P0, Player::P1] {
        let o = g.observe(p);
        assert_eq!(o.schema, 59);
        assert_eq!(o.stack, g.state.stack);
        let json = serde_json::to_value(o).unwrap();
        assert_eq!(
            json["stack"][0]["attachment_trigger"]["attacker"]["card"],
            c
        );
    }
    println!(
        "attachment context {} bytes; SpellView {} bytes",
        std::mem::size_of::<AttachmentTriggerView>(),
        std::mem::size_of::<SpellView>()
    );
}
