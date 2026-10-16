use super::tests::{act, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for a in &mut g.state.players {
        a.mana = [30; 6];
    }
    g
}
fn replay(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let e = g.snapshot();
    assert_eq!(fork.snapshot(), e);
    g.restore(&s).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), e);
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
fn choose(g: &mut Game, t: Option<Target>) {
    let aa = g.legal_actions();
    let a = t
        .and_then(|t| {
            aa.iter()
                .find(|a| **a == Action::ChooseTarget { target: t })
        })
        .or_else(|| {
            aa.iter().find(|a| {
                matches!(
                    a,
                    Action::PayCosts { .. }
                        | Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::ChooseTarget { .. }
                        | Action::Discard { .. }
                        | Action::PayResolutionMana { .. }
                )
            })
        })
        .unwrap_or_else(|| panic!("no continuation {:?} {aa:?}", g.state.pending))
        .clone();
    replay(g, a);
}
fn settle(g: &mut Game, t: Option<Target>) {
    g.prepare_triggers();
    for _ in 0..150 {
        if g.outcome().is_some()
            || g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.announcing.is_none()
                && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, t);
    }
    panic!("unsettled");
}
fn announce(g: &mut Game, id: CardId, x: u32, t: Option<Target>) {
    replay(g, Action::AnnounceCast { card: id });
    assert!(matches!(g.state.pending, Pending::X { .. }));
    replay(g, Action::ChooseX { value: x });
    while g.state.announcing.is_some() {
        choose(g, t);
    }
}
#[test]
fn scarlet_witch_domain_is_disconnected_and_cost_locks_selected_x_not_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 0];
        let witch = named(&mut g, p, "The Scarlet Witch", Zone::Battlefield);
        let id = named(&mut g, p, "Mind Spring", Zone::Hand);
        let expected = vec![
            XInterval {
                minimum: 0,
                maximum: 0,
            },
            XInterval {
                minimum: 2,
                maximum: 2,
            },
        ];
        let s = g.snapshot();
        assert_eq!(g.scalar_x_intervals(p, id), expected);
        assert_eq!(g.snapshot(), s);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::Cast{card,..}|Action::BeginCast{card,..} if *card==id))
        );
        replay(&mut g, Action::AnnounceCast { card: id });
        assert_eq!(g.observe(p).x_choice.unwrap().intervals, expected);
        assert!(g.observe(p.other()).x_choice.is_none());
        assert!(g.observe(p).payment_request.is_none());
        let s = g.snapshot();
        assert_eq!(
            g.apply(p.other(), Action::ChooseX { value: 0 }),
            Err(Error::WrongActor)
        );
        assert_eq!(g.snapshot(), s);
        for value in [1, 3, u32::MAX] {
            reject(&mut g, Action::ChooseX { value });
        }
        replay(&mut g, Action::ChooseX { value: 2 });
        assert!(matches!(g.state.pending, Pending::PayCosts { .. }));
        assert_eq!(g.observe(p).payment_request.unwrap().generic, 0);
        g.move_object(witch, Zone::Graveyard);
        assert_eq!(g.observe(p).payment_request.unwrap().generic, 0);
        choose(&mut g, None);
        assert_eq!(g.state.stack[0].x, Some(2));
        assert_eq!(g.state.stack[0].mana_value(), 4);
        assert_eq!(g.state.stack[0].source_card.fixed_mana_value(), 2);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let drawn = g.state.players[p.index()].cards_drawn_this_turn;
        settle(&mut g, None);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, drawn + 2);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
}
#[test]
fn billion_wide_x_domain_is_complete_without_concrete_enumeration() {
    let mut g = setup(Player::P0);
    g.state.players[0].mana = [0, 2, 0, 0, 0, u32::MAX];
    let id = named(&mut g, Player::P0, "Mind Spring", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: id });
    let o = g.observe(Player::P0);
    assert_eq!(
        o.x_choice.unwrap().intervals,
        vec![XInterval {
            minimum: 0,
            maximum: u32::MAX
        }]
    );
    assert!(g.legal_actions().len() <= 4);
    assert!(
        g.legal_action_space()
            .x_choice
            .unwrap()
            .contains(123_456_789)
    );
    replay(&mut g, Action::ChooseX { value: 123_456_789 });
    assert_eq!(
        g.observe(Player::P0).payment_request.unwrap().generic,
        123_456_789
    );
    let saved = g.snapshot();
    reject(&mut g, Action::ChooseX { value: 1 });
    assert_eq!(g.snapshot(), saved);
}
#[test]
fn scalar_raw_generic_and_reduction_arithmetic_is_widened_before_narrowing() {
    let mut g = setup(Player::P0);
    let w = named(&mut g, Player::P0, "The Scarlet Witch", Zone::Battlefield);
    let id = named(&mut g, Player::P0, "Mind Spring", Zone::Hand);
    g.state.objects[w as usize].growth.power = 18;
    let cost = g
        .scalar_x_total(
            Player::P0,
            id,
            ManaCost {
                colored: [0; 6],
                generic: 17,
            },
            u32::MAX,
        )
        .unwrap();
    assert_eq!(cost.generic, u32::MAX - 3);
    g.state.objects[w as usize].growth.power = i128::from(u32::MAX) + 18;
    assert_eq!(
        g.scalar_x_total(
            Player::P0,
            id,
            ManaCost {
                colored: [0; 6],
                generic: 17
            },
            u32::MAX
        )
        .unwrap()
        .generic,
        0
    );
}
const NAMES: [&str; 12] = [
    "Choco-Comet",
    "Mind Spring",
    "Pedal to the Metal",
    "Procrastinate",
    "Slice from the Shadows",
    "Song of Totentanz",
    "Spectral Denial",
    "Syncopate",
    "Traumatic Critique",
    "United Front",
    "Morlun, Devourer of Spiders",
    "Pterafractyl",
];
fn settle_declining(g: &mut Game, t: Option<Target>) {
    g.prepare_triggers();
    for _ in 0..150 {
        if g.outcome().is_some()
            || g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.announcing.is_none()
                && g.state.resolving.is_none()
        {
            return;
        }
        if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
            replay(g, Action::DeclineResolutionMana);
        } else {
            choose(g, t);
        }
    }
    panic!("unsettled declining");
}
#[test]
fn twelve_complete_scalar_bodies_choose_zero_or_three_and_resolve_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (i, n) in NAMES.iter().enumerate() {
            for x in [0, 3] {
                let mut g = setup(p);
                let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let foe = named(
                    &mut g,
                    p.other(),
                    "Malevolent Chandelier",
                    Zone::Battlefield,
                );
                let own_large = named(&mut g, p, "Malevolent Chandelier", Zone::Battlefield);
                let id = named(&mut g, p, n, Zone::Hand);
                let mut shock = None;
                let target = match i {
                    0 | 3 | 4 => Some(Target::Card(foe)),
                    2 => Some(Target::Card(own)),
                    6 | 7 => {
                        let s = named(&mut g, p.other(), "Shock", Zone::Hand);
                        g.state.priority = p.other();
                        replay(&mut g, Action::AnnounceCast { card: s });
                        while g.state.announcing.is_some() {
                            choose(&mut g, Some(Target::Card(own)));
                        }
                        replay(&mut g, Action::Pass);
                        shock = Some(s);
                        Some(Target::Spell(s))
                    }
                    8 => Some(Target::Player(p.other())),
                    10 => Some(Target::Player(p.other())),
                    _ => None,
                };
                let life = g.state.players[p.index()].life;
                let otherlife = g.state.players[p.other().index()].life;
                let draws = g.state.players[p.index()].cards_drawn_this_turn;
                let discards = g.state.players[p.index()].cards_discarded_this_turn;
                announce(&mut g, id, x, target);
                let packet = g.state.stack.iter().find(|v| v.card == id).unwrap();
                assert_eq!(packet.x, Some(x));
                assert_eq!(
                    packet.mana_value(),
                    packet.source_card.fixed_mana_value() + u64::from(x)
                );
                settle_declining(&mut g, target);
                match i {
                    0 => {
                        assert_eq!(g.object_view(foe).damage, i128::from(x));
                        let birds = g
                            .state
                            .objects
                            .iter()
                            .filter(|o| {
                                o.owner == p
                                    && o.zone == Zone::Battlefield
                                    && o.card.token_info().is_some_and(|t| t.subtypes == "Bird")
                            })
                            .count();
                        assert_eq!(birds, 1);
                    }
                    1 => assert_eq!(
                        g.state.players[p.index()].cards_drawn_this_turn,
                        draws + u64::from(x)
                    ),
                    2 => {
                        assert_eq!(
                            g.stats(own),
                            Stats {
                                power: 2 + i128::from(x),
                                toughness: 2
                            }
                        );
                        assert!(g.keywords(own).contains(Keywords::FIRST_STRIKE));
                    }
                    3 => {
                        assert!(g.object_view(foe).tapped);
                        assert_eq!(g.object_view(foe).counters.stun, 2 * u64::from(x));
                    }
                    4 => {
                        assert_eq!(
                            g.stats(foe),
                            Stats {
                                power: 4 - i128::from(x),
                                toughness: 4 - i128::from(x)
                            }
                        );
                        assert!(Card::from_name(n).unwrap().cannot_be_countered());
                    }
                    5 => {
                        let rats = g
                            .state
                            .objects
                            .iter()
                            .enumerate()
                            .filter(|(_, o)| {
                                o.owner == p
                                    && o.zone == Zone::Battlefield
                                    && o.card.token_info().is_some_and(|t| t.subtypes == "Rat")
                            })
                            .map(|(id, _)| id as CardId)
                            .collect::<Vec<_>>();
                        assert_eq!(rats.len(), x as usize);
                        for r in rats {
                            assert!(g.keywords(r).contains(Keywords::CANT_BLOCK));
                            assert!(g.keywords(r).contains(Keywords::HASTE));
                        }
                        assert!(g.keywords(own).contains(Keywords::HASTE));
                    }
                    6 | 7 => {
                        assert_eq!(
                            g.object_view(shock.unwrap()).zone,
                            if i == 7 { Zone::Exile } else { Zone::Graveyard }
                        );
                        assert_eq!(g.object_view(own).zone, Zone::Battlefield);
                    }
                    8 => {
                        assert_eq!(
                            g.state.players[p.other().index()].life,
                            otherlife - i128::from(x)
                        );
                        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, draws + 2);
                        assert_eq!(
                            g.state.players[p.index()].cards_discarded_this_turn,
                            discards + 1
                        );
                    }
                    9 => {
                        let allies = g
                            .state
                            .objects
                            .iter()
                            .enumerate()
                            .filter(|(_, o)| {
                                o.owner == p
                                    && o.zone == Zone::Battlefield
                                    && o.card.token_info().is_some_and(|t| t.subtypes == "Ally")
                            })
                            .map(|(id, _)| id as CardId)
                            .collect::<Vec<_>>();
                        assert_eq!(allies.len(), x as usize);
                        for a in allies {
                            assert_eq!(
                                g.stats(a),
                                Stats {
                                    power: 2,
                                    toughness: 2
                                }
                            );
                        }
                        assert_eq!(g.object_view(own).counters.plus_one, 1);
                        assert_eq!(g.object_view(own_large).counters.plus_one, 1);
                    }
                    10 => {
                        assert_eq!(g.object_view(id).counters.plus_one, u64::from(x));
                        assert_eq!(
                            g.stats(id),
                            Stats {
                                power: 2 + i128::from(x),
                                toughness: 1 + i128::from(x)
                            }
                        );
                        assert_eq!(g.state.players[p.index()].life, life + i128::from(x));
                        assert_eq!(
                            g.state.players[p.other().index()].life,
                            otherlife - i128::from(x)
                        );
                    }
                    11 => {
                        assert_eq!(g.state.players[p.index()].life, life + 2);
                        if x == 0 {
                            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
                        } else {
                            assert_eq!(g.object_view(id).counters.plus_one, u64::from(x));
                            assert_eq!(
                                g.stats(id),
                                Stats {
                                    power: 1 + i128::from(x),
                                    toughness: i128::from(x)
                                }
                            );
                            assert!(g.keywords(id).contains(Keywords::FLYING));
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}
#[test]
fn stack_counter_filters_and_cast_observers_use_selected_x_even_after_discounts() {
    for p in [Player::P0, Player::P1] {
        for x in [0, 2] {
            let mut g = setup(p);
            let rabble = named(&mut g, p, "Angry Rabble", Zone::Battlefield);
            named(&mut g, p, "The Scarlet Witch", Zone::Battlefield);
            let id = named(&mut g, p, "Mind Spring", Zone::Hand);
            announce(&mut g, id, x, None);
            let packet = g.state.stack.iter().find(|v| v.card == id).unwrap();
            assert_eq!(packet.mana_value(), 2 + u64::from(x));
            let packets = g
                .state
                .stack
                .iter()
                .chain(&g.state.pending_triggers)
                .chain(&g.state.ordering_triggers)
                .chain(g.state.announcing.iter().map(|a| &a.item));
            assert_eq!(
                packets.filter(|v| v.card == rabble).count(),
                usize::from(x == 2)
            );
            for (n, min, max, yes) in [
                ("Disdainful Stroke", 4, u64::MAX, x == 2),
                ("Spell Snare", 2, 2, x == 0),
            ] {
                let c = named(&mut g, p.other(), n, Zone::Hand);
                assert_eq!(
                    g.target_matches(
                        TargetRule::SpellCharacteristics {
                            colors: 0,
                            min_mana: min,
                            max_mana: max
                        },
                        p.other(),
                        g.object_ref(c),
                        Target::Spell(id)
                    ),
                    yes
                );
            }
            let prior = g.state.players[p.other().index()].life;
            settle(&mut g, None);
            assert_eq!(
                g.state.players[p.other().index()].life,
                prior - i128::from(x == 2)
            );
        }
    }
}
#[test]
fn exact_entry_x_survives_source_departure_and_does_not_rebind_to_reentry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Morlun, Devourer of Spiders", Zone::Hand);
        announce(&mut g, id, 3, None);
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        let old = g.object_ref(id);
        assert_eq!(g.object_view(id).counters.plus_one, 3);
        assert_eq!(g.state.announcing.as_ref().unwrap().item.x, Some(3));
        assert!(g.state.announcing.as_ref().unwrap().item.cast.is_none());
        g.move_object(id, Zone::Graveyard);
        g.move_object(id, Zone::Battlefield);
        assert_ne!(g.object_ref(id), old);
        assert_eq!(g.object_view(id).counters.plus_one, 0);
        let life = g.state.players[p.index()].life;
        let foe = g.state.players[p.other().index()].life;
        settle(&mut g, Some(Target::Player(p.other())));
        assert_eq!(g.state.players[p.other().index()].life, foe - 3);
        assert_eq!(g.state.players[p.index()].life, life + 3);
        assert_eq!(g.object_view(id).counters.plus_one, 0);
    }
}
#[test]
fn intervals_match_exhaustive_small_funding_with_real_untapped_sources_and_thresholds() {
    for p in [Player::P0, Player::P1] {
        for spare in 0..4 {
            for forests in 0..3 {
                for power in 0..5 {
                    let mut g = setup(p);
                    g.state.players[p.index()].mana = [0, 2, 0, 0, 0, spare];
                    let witch = named(&mut g, p, "The Scarlet Witch", Zone::Battlefield);
                    g.state.objects[witch as usize].growth.power = i128::from(power) - 2;
                    for _ in 0..forests {
                        named(&mut g, p, "Forest", Zone::Battlefield);
                    }
                    let id = named(&mut g, p, "Mind Spring", Zone::Hand);
                    let snap = g.snapshot();
                    let intervals = g.scalar_x_intervals(p, id);
                    for x in 0u32..24 {
                        let cost = x.saturating_sub(if x + 2 >= 4 { power } else { 0 });
                        let expected = cost <= spare + forests;
                        assert_eq!(
                            intervals.iter().any(|r| x >= r.minimum && x <= r.maximum),
                            expected,
                            "spare{spare} forests{forests} power{power} X{x}"
                        );
                    }
                    assert_eq!(g.snapshot(), snap);
                }
            }
        }
    }
}
#[test]
fn spectral_denial_counts_only_live_owned_power_four_creatures_for_locked_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
        let large = named(&mut g, p, "Malevolent Chandelier", Zone::Battlefield);
        named(
            &mut g,
            p.other(),
            "Malevolent Chandelier",
            Zone::Battlefield,
        );
        let small = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let id = named(&mut g, p, "Spectral Denial", Zone::Hand);
        assert_eq!(
            g.scalar_x_intervals(p, id),
            vec![XInterval {
                minimum: 0,
                maximum: 3
            }]
        );
        g.state.objects[large as usize].growth.power = -1;
        assert_eq!(
            g.scalar_x_intervals(p, id),
            vec![XInterval {
                minimum: 0,
                maximum: 2
            }]
        );
        g.state.objects[small as usize].counters.plus_one = 2;
        assert_eq!(
            g.scalar_x_intervals(p, id),
            vec![XInterval {
                minimum: 0,
                maximum: 3
            }]
        );
        g.move_object(small, Zone::Graveyard);
        assert_eq!(
            g.scalar_x_intervals(p, id),
            vec![XInterval {
                minimum: 0,
                maximum: 2
            }]
        );
    }
}
#[test]
fn failed_late_scaled_counter_and_cast_trigger_payments_restore_x_mana_stack_and_rng() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let foe = named(
            &mut g,
            p.other(),
            "Malevolent Chandelier",
            Zone::Battlefield,
        );
        g.state.objects[foe as usize].counters.stun = u64::MAX - 5;
        let id = named(&mut g, p, "Procrastinate", Zone::Hand);
        announce(&mut g, id, 3, Some(Target::Card(foe)));
        replay(&mut g, Action::Pass);
        let saved = g.snapshot();
        let rev = g.revision();
        assert!(matches!(
            g.apply(g.actor().unwrap(), Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), rev);
        assert!(!g.object_view(foe).tapped);
        g.state.objects[foe as usize].counters.stun = 0;
        settle(&mut g, None);
        assert_eq!(g.object_view(foe).counters.stun, 6);
        let mut g = setup(p);
        named(&mut g, p, "Angry Rabble", Zone::Battlefield);
        let id = named(&mut g, p, "Mind Spring", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: id });
        replay(&mut g, Action::ChooseX { value: 3 });
        g.state.next_stack_id = u64::MAX;
        let saved = g.snapshot();
        let rev = g.revision();
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        assert!(matches!(g.apply(p, pay), Err(Error::CapacityExceeded(_))));
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), rev);
        assert_eq!(g.state.announcing.as_ref().unwrap().item.x, Some(3));
    }
}
#[test]
fn scalar_counter_pay_zero_and_positive_preserve_or_exile_actual_target_spell() {
    for p in [Player::P0, Player::P1] {
        for name in ["Spectral Denial", "Syncopate"] {
            for x in [0, 3] {
                let mut g = setup(p);
                let own = named(&mut g, p, "Malevolent Chandelier", Zone::Battlefield);
                let shock = named(&mut g, p.other(), "Shock", Zone::Hand);
                g.state.priority = p.other();
                replay(&mut g, Action::AnnounceCast { card: shock });
                while g.state.announcing.is_some() {
                    choose(&mut g, Some(Target::Card(own)));
                }
                replay(&mut g, Action::Pass);
                let id = named(&mut g, p, name, Zone::Hand);
                announce(&mut g, id, x, Some(Target::Spell(shock)));
                let before = g.state.players[p.other().index()]
                    .mana
                    .iter()
                    .map(|&n| u64::from(n))
                    .sum::<u64>();
                for _ in 0..30 {
                    if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
                        break;
                    }
                    choose(&mut g, None);
                }
                let request = g.observe(p.other()).resolution_payment.unwrap();
                assert_eq!(request.cost.generic, x);
                let pay = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayResolutionMana { .. }))
                    .unwrap();
                replay(&mut g, pay);
                assert_eq!(
                    g.state.players[p.other().index()]
                        .mana
                        .iter()
                        .map(|&n| u64::from(n))
                        .sum::<u64>(),
                    before - u64::from(x)
                );
                assert!(g.state.stack.iter().any(|v| v.card == shock));
                settle(&mut g, None);
                assert_eq!(g.object_view(own).damage, 2);
                assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
            }
        }
    }
}
#[test]
fn numeric_x_json_rejects_out_of_bound_values_and_public_view_is_actor_only() {
    assert_eq!(
        serde_json::from_value::<Action>(serde_json::json!({"kind":"ChooseX","value":u32::MAX}))
            .unwrap(),
        Action::ChooseX { value: u32::MAX }
    );
    assert!(
        serde_json::from_value::<Action>(
            serde_json::json!({"kind":"ChooseX","value":u64::from(u32::MAX)+1})
        )
        .is_err()
    );
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P0, "Mind Spring", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: id });
    let mine = serde_json::to_value(g.observe(Player::P0)).unwrap();
    let other = serde_json::to_value(g.observe(Player::P1)).unwrap();
    assert!(mine["x_choice"]["intervals"].is_array());
    assert!(other["x_choice"].is_null());
    assert!(other["actions"].as_array().unwrap().is_empty());
}
#[test]
fn targeted_x_fizzles_whole_chocobo_tail_and_uncounterable_slice_keeps_declared_amount() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let foe = named(
            &mut g,
            p.other(),
            "Malevolent Chandelier",
            Zone::Battlefield,
        );
        let id = named(&mut g, p, "Choco-Comet", Zone::Hand);
        announce(&mut g, id, 3, Some(Target::Card(foe)));
        g.move_object(foe, Zone::Graveyard);
        settle(&mut g, None);
        assert!(
            !g.state
                .objects
                .iter()
                .any(|o| o.card.token_info().is_some_and(|t| t.subtypes == "Bird"))
        );
        let target = named(
            &mut g,
            p.other(),
            "Malevolent Chandelier",
            Zone::Battlefield,
        );
        let slice = named(&mut g, p, "Slice from the Shadows", Zone::Hand);
        announce(&mut g, slice, 3, Some(Target::Card(target)));
        let counter = named(&mut g, p.other(), "Negate", Zone::Hand);
        g.state.priority = p.other();
        replay(&mut g, Action::AnnounceCast { card: counter });
        while g.state.announcing.is_some() {
            choose(&mut g, Some(Target::Spell(slice)));
        }
        settle(&mut g, Some(Target::Spell(slice)));
        assert_eq!(g.stats(target).toughness, 1);
    }
}
#[test]
fn disconnected_domain_uses_conversion_and_sacrificial_virtual_funding_routes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        named(&mut g, p, "The Scarlet Witch", Zone::Battlefield);
        let a = named(&mut g, p, "Prophetic Prism", Zone::Battlefield);
        named(&mut g, p, "Prophetic Prism", Zone::Battlefield);
        let id = named(&mut g, p, "Mind Spring", Zone::Hand);
        let expected = vec![
            XInterval {
                minimum: 0,
                maximum: 0,
            },
            XInterval {
                minimum: 2,
                maximum: 2,
            },
        ];
        let saved = g.snapshot();
        assert_eq!(g.scalar_x_intervals(p, id), expected);
        assert_eq!(g.snapshot(), saved);
        g.state.objects[a as usize].tapped = true;
        assert!(g.scalar_x_intervals(p, id).is_empty());
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        named(&mut g, p, "The Scarlet Witch", Zone::Battlefield);
        let treasure = Card::catalogue()
            .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Treasure"))
            .unwrap();
        let sources = [
            spawn(&mut g, p, treasure, Zone::Battlefield),
            spawn(&mut g, p, treasure, Zone::Battlefield),
        ];
        let id = named(&mut g, p, "Mind Spring", Zone::Hand);
        assert_eq!(g.scalar_x_intervals(p, id), expected);
        replay(&mut g, Action::AnnounceCast { card: id });
        replay(&mut g, Action::ChooseX { value: 2 });
        for source in sources {
            let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,production,..} if *card==source && *production==[0,1,0,0,0,0])).unwrap();
            replay(&mut g, a);
        }
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::PayCosts{payment,..} if *payment==[0,2,0,0,0,0]))
            .unwrap();
        replay(&mut g, payment);
        settle(&mut g, None);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert!(
            sources
                .into_iter()
                .all(|id| g.state.objects[id as usize].zone != Zone::Battlefield)
        );
    }
}
