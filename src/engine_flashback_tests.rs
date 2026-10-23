use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 9] = [
    "Auron's Inspiration",
    "Bestial Incursion",
    "Dig Site Inventory",
    "Dreams of Laguna",
    "Laughing Mad",
    "Predictive Preparations",
    "Revenge of the Rats",
    "Think Twice",
    "Tome Blast",
];
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    spawn(g, p, Card::from_name(n).unwrap(), z)
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let rev = g.revision();
    let actor = g.actor().unwrap();
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
    let old = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(old, g.snapshot());
    assert_eq!(rev, g.revision());
}
fn finish(g: &mut Game) {
    while g.state.announcing.is_some() {
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| matches!(a, Action::ChooseTarget { .. }))
            .or_else(|| {
                actions.iter().find(|a| {
                    matches!(
                        a,
                        Action::FinishTargets
                            | Action::ChooseAdditionalCost { .. }
                            | Action::PayCosts { .. }
                    )
                })
            })
            .unwrap();
        replay(g, a.clone());
    }
}
fn resolve(g: &mut Game) {
    for _ in 0..100 {
        if g.state.stack.is_empty() && g.state.resolving.is_none() {
            return;
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass | Action::ArrangeLibrary { .. } | Action::PutTrigger { .. }
                )
            })
            .unwrap();
        replay(g, a.clone());
    }
    panic!("resolution did not finish")
}
#[test]
fn all_nine_complete_bodies_cast_from_both_origins_and_have_independent_effects() {
    for (i, n) in NAMES.iter().enumerate() {
        for flashback in [false, true] {
            let mut g = scenario();
            g.state.players[0].mana = [30; 6];
            let spell = named(
                &mut g,
                Player::P0,
                n,
                if flashback {
                    Zone::Graveyard
                } else {
                    Zone::Hand
                },
            );
            let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
            let other = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
            named(&mut g, Player::P0, "Bear Cub", Zone::Graveyard);
            named(&mut g, Player::P0, "Forest", Zone::Hand);
            g.state.attackers.push(Attacker {
                card: bear,
                defender: Some(CombatDefender::Player(Player::P1)),
                blocked: false,
            });
            let olddraw = g.state.players[0].cards_drawn_this_turn;
            let life = g.state.players[0].life;
            replay(
                &mut g,
                if flashback {
                    Action::AnnounceFlashback { card: spell }
                } else {
                    Action::AnnounceCast { card: spell }
                },
            );
            finish(&mut g);
            let root = &g.state.stack[0];
            let actual = root.cast.unwrap();
            let cost = if flashback {
                root.source_card.flashback_cost().unwrap()
            } else {
                let (colored, generic) = root.source_card.cost();
                ManaCost { colored, generic }
            };
            assert_eq!(
                actual.mana_spent(),
                cost.colored.into_iter().map(u64::from).sum::<u64>() + u64::from(cost.generic)
            );
            assert_eq!(
                root.cast,
                Some(CastInfo {
                    sneak: None,
                    origin_permissions: CastOriginPermissions::Neither,
                    gift: None,
                    physical_card: None,
                    flash_surcharge: false,
                    face: CastFace::Normal,
                    payment: root.cast.unwrap().payment,
                    offspring: None,
                    kicker: None,
                    bargain: None,
                    evidence: None,
                    optional_additional: None,
                    evoked: false,
                    origin: if flashback {
                        Zone::Graveyard
                    } else {
                        Zone::Hand
                    },
                    method: if flashback {
                        CastMethod::Flashback
                    } else {
                        CastMethod::Normal
                    }
                })
            );
            assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
            resolve(&mut g);
            assert_eq!(
                g.object_view(spell).zone,
                if flashback {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                },
                "{n}"
            );
            assert!(!g.state.objects[spell as usize].flashback);
            match i {
                0 => {
                    assert_eq!((g.stats(bear).power, g.stats(other).power), (4, 2));
                }
                1 => {
                    let token = g
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .find(|(_, o)| o.card.is_token())
                        .unwrap()
                        .0 as u32;
                    assert_eq!((g.stats(token).power, g.stats(token).toughness), (4, 4));
                    assert!(g.keywords(token).contains(Keywords::TRAMPLE));
                }
                2 => {
                    assert_eq!(g.object_view(bear).counters.plus_one, 1);
                    assert!(g.keywords(bear).contains(Keywords::VIGILANCE));
                }
                3 | 7 => assert_eq!(g.state.players[0].cards_drawn_this_turn, olddraw + 1),
                4 => {
                    assert_eq!(g.state.players[0].cards_drawn_this_turn, olddraw + 2);
                    assert_eq!(g.state.players[0].cards_discarded_this_turn, 1);
                }
                5 => {
                    assert_eq!(g.object_view(bear).counters.plus_one, 1);
                    assert_eq!(g.object_view(other).counters.plus_one, 1);
                }
                6 => {
                    let rats: Vec<_> = g
                        .state
                        .objects
                        .iter()
                        .filter(|o| o.card.is_token())
                        .collect();
                    assert_eq!(rats.len(), 1);
                    assert!(rats[0].tapped);
                    assert_eq!(rats[0].card.info().type_line, "Creature — Rat");
                }
                8 => assert_eq!(g.state.players[0].life, life - 2),
                _ => unreachable!(),
            }
        }
    }
}
#[test]
fn ownership_origin_timing_and_direct_cast_bypasses_are_enforced_atomically() {
    let mut g = scenario();
    g.state.players[0].mana = [30; 6];
    let spell = named(&mut g, Player::P0, "Think Twice", Zone::Graveyard);
    reject(&mut g, Action::AnnounceCast { card: spell });
    reject(
        &mut g,
        Action::Cast {
            card: spell,
            target: None,
            payment: [0, 3, 0, 0, 0, 0],
        },
    );
    reject(
        &mut g,
        Action::BeginCast {
            card: spell,
            payment: [0, 3, 0, 0, 0, 0],
        },
    );
    for (owner, zone) in [
        (Player::P1, Zone::Graveyard),
        (Player::P0, Zone::Hand),
        (Player::P0, Zone::Exile),
        (Player::P0, Zone::Library),
    ] {
        let id = named(&mut g, owner, "Think Twice", zone);
        reject(&mut g, Action::AnnounceFlashback { card: id });
    }
    let ordinary = named(&mut g, Player::P0, "Divination", Zone::Graveyard);
    reject(&mut g, Action::AnnounceFlashback { card: ordinary });
    let sorcery = named(&mut g, Player::P0, "Tome Blast", Zone::Graveyard);
    g.state.step = Step::End;
    reject(&mut g, Action::AnnounceFlashback { card: sorcery });
    assert!(
        g.legal_actions()
            .contains(&Action::AnnounceFlashback { card: spell })
    );
    g.state.active = Player::P1;
    g.state.step = Step::Main1;
    reject(&mut g, Action::AnnounceFlashback { card: sorcery });
}
#[test]
fn laughing_mad_requires_another_hand_card_and_pays_it_before_cast_from_either_origin() {
    for zone in [Zone::Hand, Zone::Graveyard] {
        let mut g = scenario();
        g.state.players[0].mana = [30; 6];
        let id = named(&mut g, Player::P0, "Laughing Mad", zone);
        let action = if zone == Zone::Hand {
            Action::AnnounceCast { card: id }
        } else {
            Action::AnnounceFlashback { card: id }
        };
        reject(&mut g, action.clone());
        let cost = named(&mut g, Player::P0, "Forest", Zone::Hand);
        replay(&mut g, action);
        let source_ref = g.object_ref(id);
        reject(&mut g, Action::ChooseAdditionalCost { object: source_ref });
        assert!(g.observe(Player::P1).additional_cost.is_none());
        let cost_ref = g.object_ref(cost);
        replay(&mut g, Action::ChooseAdditionalCost { object: cost_ref });
        assert_eq!(g.object_view(cost).zone, Zone::Hand);
        finish(&mut g);
        assert_eq!(g.object_view(cost).zone, Zone::Graveyard);
        assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
    }
}
#[test]
fn flashback_exiles_on_every_requested_stack_departure_without_packet_lookup() {
    for destination in [
        Zone::Hand,
        Zone::Graveyard,
        Zone::Library,
        Zone::Exile,
        Zone::Battlefield,
    ] {
        let mut g = scenario();
        g.state.players[0].mana = [30; 6];
        let id = named(&mut g, Player::P0, "Think Twice", Zone::Graveyard);
        act(&mut g, Action::AnnounceFlashback { card: id });
        finish(&mut g);
        let packet = g.state.stack.pop().unwrap();
        let before = packet.object;
        let movement = g.move_object(id, destination);
        assert_eq!(movement.destination, Zone::Exile);
        assert_eq!(movement.before.reference, before);
        assert_eq!(movement.after.generation, before.generation + 1);
        assert_eq!(g.object_view(id).zone, Zone::Exile);
        assert!(!g.state.objects[id as usize].flashback);
        g.move_object(id, Zone::Hand);
        let ordinary = g.cast_item(Player::P0, id, [0; 6]);
        assert_eq!(ordinary.cast.unwrap().method, CastMethod::Normal);
        assert_eq!(
            g.move_object(id, Zone::Graveyard).destination,
            Zone::Graveyard
        );
    }
}
#[test]
fn all_illegal_targets_fizzle_to_exile_and_partial_targets_still_resolve() {
    for partial in [false, true] {
        let mut g = scenario();
        g.state.players[0].mana = [30; 6];
        let id = named(
            &mut g,
            Player::P0,
            "Predictive Preparations",
            Zone::Graveyard,
        );
        let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        let other = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        act(&mut g, Action::AnnounceFlashback { card: id });
        finish(&mut g);
        g.move_object(bear, Zone::Hand);
        if !partial {
            g.move_object(other, Zone::Hand);
        }
        resolve(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Exile);
        assert_eq!(g.object_view(other).counters.plus_one, u64::from(partial));
    }
}
#[test]
fn counterspell_exiles_flashback_but_ordinary_cast_goes_to_graveyard() {
    for flashback in [false, true] {
        let mut g = scenario();
        g.state.players[0].mana = [30; 6];
        g.state.players[1].mana = [30; 6];
        let id = named(
            &mut g,
            Player::P0,
            "Think Twice",
            if flashback {
                Zone::Graveyard
            } else {
                Zone::Hand
            },
        );
        let counter = named(&mut g, Player::P1, "Counterspell", Zone::Hand);
        act(
            &mut g,
            if flashback {
                Action::AnnounceFlashback { card: id }
            } else {
                Action::AnnounceCast { card: id }
            },
        );
        finish(&mut g);
        act(&mut g, Action::Pass);
        act(&mut g, Action::AnnounceCast { card: counter });
        finish(&mut g);
        resolve(&mut g);
        assert_eq!(
            g.object_view(id).zone,
            if flashback {
                Zone::Exile
            } else {
                Zone::Graveyard
            }
        );
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
    }
}
#[test]
fn chosen_alternative_cost_reductions_preserve_printed_mana_value() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Stormcatch Mentor", Zone::Battlefield);
    g.state.players[0].mana = [30; 6];
    let lizards = named(&mut g, Player::P0, "Lurking Lizards", Zone::Battlefield);
    let tome = named(&mut g, Player::P0, "Tome Blast", Zone::Graveyard);
    act(&mut g, Action::AnnounceFlashback { card: tome });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    let payment = g.observe(Player::P0).payment_request.unwrap();
    assert_eq!((payment.generic, payment.required), (3, [0, 0, 0, 1, 0, 0]));
    assert_eq!(
        g.state
            .announcing
            .as_ref()
            .unwrap()
            .item
            .source_card
            .fixed_mana_value(),
        2
    );
    finish(&mut g);
    resolve(&mut g);
    assert_eq!(g.object_view(lizards).counters.plus_one, 0);
    let zero = named(&mut g, Player::P0, "Dig Site Inventory", Zone::Graveyard);
    assert_eq!(g.state.objects[zero as usize].card.cost().1, 0);
    assert_eq!(
        g.reduced_spell_cost(
            Player::P0,
            zero,
            ManaCost {
                colored: [1, 0, 0, 0, 0, 0],
                generic: 4
            }
        )
        .generic,
        3
    );
    let target = named(&mut g, Player::P1, "Shivan Dragon", Zone::Battlefield);
    let prowler = named(&mut g, Player::P0, "Stormkeld Prowler", Zone::Battlefield);
    let current = named(&mut g, Player::P0, "Defibrillating Current", Zone::Hand);
    let c = g.state.objects[current as usize].card;
    assert_eq!(c.fixed_mana_value(), 6);
    let base = c.cost_options().max_by_key(|c| c.generic).unwrap();
    assert_eq!(base.generic, 6);
    act(&mut g, Action::AnnounceCast { card: current });
    act(&mut g, Action::ChooseSpellCost { cost: 0 });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    assert_eq!(g.observe(Player::P0).payment_request.unwrap().generic, 5);
    assert_eq!(
        g.state
            .announcing
            .as_ref()
            .unwrap()
            .item
            .source_card
            .fixed_mana_value(),
        6
    );
    finish(&mut g);
    resolve(&mut g);
    assert_eq!(g.object_view(prowler).counters.plus_one, 2);
}
#[test]
fn graveyard_preflight_cannot_count_the_spell_itself_as_a_future_target() {
    let mut g = scenario();
    g.state.players[0].mana = [30; 6];
    let id = named(&mut g, Player::P0, "Think Twice", Zone::Graveyard);
    let plan = TargetPlan::Single(TargetSpec {
        rule: TargetRule::Object {
            zone: TargetZone::Graveyard,
            relation: 0,
            kinds: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        },
        min: 1,
        max: 1,
    });
    assert!(g.targets_available(plan, Player::P0, g.object_ref(id)));
    assert!(!g.cast_targets_available(plan, Player::P0, g.object_ref(id)));
    named(&mut g, Player::P0, "Forest", Zone::Graveyard);
    assert!(g.cast_targets_available(plan, Player::P0, g.object_ref(id)));
}
#[test]
fn announcement_and_resolution_capacity_failures_restore_flashback_marker_and_private_frames() {
    let mut g = scenario();
    g.state.players[0].mana = [30; 6];
    let id = named(&mut g, Player::P0, "Think Twice", Zone::Graveyard);
    g.state.next_stack_id = u64::MAX;
    let saved = g.snapshot();
    assert!(matches!(
        g.apply(Player::P0, Action::AnnounceFlashback { card: id }),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), saved);
    g.state.next_stack_id = 0;
    act(&mut g, Action::AnnounceFlashback { card: id });
    finish(&mut g);
    g.state.objects[id as usize].generation = u64::MAX;
    g.state.stack[0].object = g.object_ref(id);
    act(&mut g, Action::Pass);
    let saved = g.snapshot();
    assert!(matches!(
        g.apply(Player::P1, Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), saved);
    assert!(g.state.objects[id as usize].flashback);
}

#[test]
fn flashback_funding_uses_alternative_mana_and_regular_timing_in_response() {
    let mut g = scenario();
    let spell = named(&mut g, Player::P0, "Think Twice", Zone::Graveyard);
    g.state.players[0].mana = [0, 1, 0, 0, 0, 1]; // Printed cost is payable, flashback is not.
    reject(&mut g, Action::AnnounceFlashback { card: spell });
    g.state.players[0].mana[5] = 2;
    replay(&mut g, Action::AnnounceFlashback { card: spell });
    let payment = g.observe(Player::P0).payment_request.unwrap();
    assert_eq!((payment.required, payment.generic), ([0, 1, 0, 0, 0, 0], 2));
    reject(
        &mut g,
        Action::PayCosts {
            payment: [0, 1, 0, 0, 0, 1],
        },
    );
    finish(&mut g);
    let sorcery = named(&mut g, Player::P0, "Tome Blast", Zone::Graveyard);
    let instant = named(&mut g, Player::P0, "Think Twice", Zone::Graveyard);
    g.state.players[0].mana = [30; 6];
    reject(&mut g, Action::AnnounceFlashback { card: sorcery });
    assert!(
        g.legal_actions()
            .contains(&Action::AnnounceFlashback { card: instant })
    );
}
