use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;

fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert!(g.apply(g.actor().unwrap(), action).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn announce(g: &mut Game, source: CardId, ability: u16, counters: Counters) {
    checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability,
            counter_payment: counters,
        },
    );
}
fn pay(g: &mut Game) {
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .or_else(|| {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::FinishTargets))
            })
            .unwrap();
        checked(g, action);
    }
    panic!("unpaid activation");
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn drakes(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield && o.controller == p && o.card == Card::DrakeToken)
                .then_some(i as CardId)
        })
        .collect()
}
fn combat(g: &mut Game, source: CardId) {
    g.state.objects[source as usize].summoning_sick = false;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    checked(g, Action::Attack { card: source });
    checked(g, Action::FinishAttackers);
    settle(g);
    // Advance priority from declared attackers to the actual blocker declaration.
    while g.state.pending == Pending::Priority {
        checked(g, Action::Pass);
    }
    assert_eq!(g.state.pending, Pending::Blockers);
    checked(g, Action::FinishBlockers);
    // Actual damage happens when the post-blocker priority window ends.
    checked(g, Action::Pass);
    checked(g, Action::Pass);
}

#[test]
fn fdn_drake_paid_body_prowess_qualification_cleanup_and_ability_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, "Drake Hatcher");
        let v = g.object_view(h);
        assert_eq!((v.power, v.toughness), (1, 3));
        assert_eq!(v.name(), "Drake Hatcher");
        assert!(v.keywords.contains(Keywords::VIGILANCE));
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, bear, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(h).power, 1);
        let opponent = named(&mut g, p.other(), "Opt", Zone::Hand);
        g.state.priority = p.other();
        cast(&mut g, opponent, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(h).power, 1);
        g.state.priority = p;
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Player(p.other())]);
        assert!(
            !g.state.pending_triggers.is_empty()
                || !g.state.ordering_triggers.is_empty()
                || g.state.stack.len() > 1
        );
        settle(&mut g);
        assert_eq!((g.object_view(h).power, g.object_view(h).toughness), (2, 4));
        g.finish_cleanup();
        assert_eq!((g.object_view(h).power, g.object_view(h).toughness), (1, 3));
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Main1;
        g.give_priority();
        let stop = named(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, stop, None, &[Target::Card(h)]);
        settle(&mut g);
        assert!(!g.keywords(h).contains(Keywords::VIGILANCE));
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Player(p.other())]);
        settle(&mut g);
        assert_eq!(
            g.object_view(h).power,
            2,
            "Stop Cold itself triggered prowess before removing abilities; Shock adds no new boost"
        );
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceActivate{card,..} if *card==h))
        );
    }
}

#[test]
fn fdn_drake_actual_combat_saved_damage_counters_and_instant_paid_token() {
    for p in [Player::P0, Player::P1] {
        for doubled in [false, true] {
            let mut g = setup(p);
            let h = paid(&mut g, p, "Drake Hatcher");
            let ground = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let flyer = named(&mut g, p.other(), "Healer's Hawk", Zone::Battlefield);
            if doubled {
                paid(&mut g, p, "Doubling Season");
                g.state.objects[h as usize].growth = Stats::default();
            }
            let growth = named(&mut g, p, "Giant Growth", Zone::Hand);
            cast(&mut g, growth, None, &[Target::Card(h)]);
            settle(&mut g);
            // Prowess plus Giant Growth makes 5/7, not merely 4/6.
            assert_eq!(g.object_view(h).power, 5);
            let life = g.state.players[p.other().index()].life;
            combat(&mut g, h);
            assert!(
                !g.object_view(h).tapped,
                "vigilance in actual attack declaration"
            );
            assert_eq!(g.state.players[p.other().index()].life, life - 5);
            assert_eq!(
                g.state.stack.last().unwrap().damage_event.unwrap().amount,
                5
            );
            // Saved damage must not be recomputed from the creature's new power.
            g.state.objects[h as usize].growth.power = 20;
            settle(&mut g);
            assert_eq!(
                g.counter_count(h, CounterKind::Incubation),
                if doubled { 10 } else { 5 }
            );
            let before = g.counter_count(h, CounterKind::Incubation);
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: h,
                    ability: 0,
                    counter_payment: Counters::of(CounterKind::PlusOne, 3),
                },
            );
            announce(&mut g, h, 0, Counters::of(CounterKind::Incubation, 3));
            pay(&mut g);
            assert_eq!(g.counter_count(h, CounterKind::Incubation), before - 3);
            assert!(drakes(&g, p).is_empty());
            settle(&mut g);
            let tokens = drakes(&g, p);
            assert_eq!(tokens.len(), if doubled { 2 } else { 1 });
            for token in tokens {
                let v = g.object_view(token);
                assert_eq!((v.power, v.toughness), (2, 2));
                assert_eq!(v.name(), "Drake Token");
                assert!(v.summoning_sick);
                assert!(!v.tapped);
                assert!(v.keywords.contains(Keywords::FLYING));
                assert!(g.has_subtype(token, "Drake"));
                assert_eq!(v.card.token_info().unwrap().colors, 2);
                assert!(!g.can_block_pair(ground, token));
                assert!(g.can_block_pair(flyer, token));
            }
            if !doubled {
                reject(
                    &mut g,
                    Action::AnnounceActivate {
                        card: h,
                        ability: 0,
                        counter_payment: Counters::of(CounterKind::Incubation, 3),
                    },
                );
            }
        }
    }
}

#[test]
fn fdn_drake_departed_trigger_and_departed_activation_do_not_refund_costs() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, "Drake Hatcher");
        combat(&mut g, h);
        let old = g.object_ref(h);
        g.move_object(h, Zone::Hand);
        g.move_object(h, Zone::Battlefield);
        assert_ne!(old, g.object_ref(h));
        settle(&mut g);
        assert_eq!(g.counter_count(h, CounterKind::Incubation), 0);
        g.add_counter(h, CounterKind::Incubation, 3);
        g.state.active = p.other();
        g.state.priority = p;
        announce(&mut g, h, 0, Counters::of(CounterKind::Incubation, 3));
        pay(&mut g);
        assert_eq!(g.counter_count(h, CounterKind::Incubation), 0);
        g.move_object(h, Zone::Hand);
        g.move_object(h, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(
            drakes(&g, p).len(),
            1,
            "ability resolves independently after source leaves"
        );
        assert_eq!(g.counter_count(h, CounterKind::Incubation), 0);
    }
}

#[test]
fn fdn_amulet_paid_sorcery_sacrifice_draw_soul_privacy_and_departure_lki() {
    for p in [Player::P0, Player::P1] {
        for doubled in [false, true] {
            let mut g = setup(p);
            let a = paid(&mut g, p, "Ravenous Amulet");
            if doubled {
                paid(&mut g, p, "Doubling Season");
            }
            let resource = paid(&mut g, p, "Bear Cub");
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let land = named(&mut g, p, "Island", Zone::Battlefield);
            let library = named(&mut g, p, "Hill Giant", Zone::Library);
            let draws = g.state.players[p.index()].cards_drawn_this_turn;
            announce(&mut g, a, 0, Counters::default());
            let object = g.object_ref(enemy);
            reject(&mut g, Action::ChooseAdditionalCost { object });
            let object = g.object_ref(land);
            reject(&mut g, Action::ChooseAdditionalCost { object });
            let object = g.object_ref(resource);
            checked(&mut g, Action::ChooseAdditionalCost { object });
            pay(&mut g);
            assert_eq!(g.object_view(resource).zone, Zone::Graveyard);
            assert!(g.object_view(a).tapped);
            assert_eq!(g.counter_count(a, CounterKind::Soul), 0);
            assert_eq!(g.object_view(library).zone, Zone::Library);
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: a,
                    ability: 1,
                    counter_payment: Counters::default(),
                },
            );
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, draws + 1);
            assert_eq!(
                g.counter_count(a, CounterKind::Soul),
                if doubled { 2 } else { 1 }
            );
            assert_eq!(g.object_view(library).zone, Zone::Hand);
            let foreign = serde_json::to_string(&g.observe(p.other())).unwrap();
            assert!(
                !foreign.contains("Hill Giant"),
                "drawn hand identity stays private"
            );
            // A later independent priority window permits instant-speed cash-out.
            g.state.objects[a as usize].tapped = false;
            g.state.active = p.other();
            g.state.priority = p;
            let expected = g.counter_count(a, CounterKind::Soul);
            let life = g.state.players[p.other().index()].life;
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: a,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            announce(&mut g, a, 1, Counters::default());
            pay(&mut g);
            assert_eq!(g.object_view(a).zone, Zone::Graveyard);
            assert_eq!(
                g.state
                    .stack
                    .last()
                    .unwrap()
                    .source_counter_lki
                    .as_ref()
                    .unwrap()[0]
                    .count,
                expected
            );
            g.move_object(a, Zone::Battlefield);
            g.add_counter(a, CounterKind::Soul, 99);
            settle(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                life - i128::from(expected)
            );
            assert_eq!(g.counter_count(a, CounterKind::Soul), 99);
        }
    }
}

#[test]
fn fdn_amulet_missing_mana_creature_stack_and_source_departure_are_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = paid(&mut g, p, "Ravenous Amulet");
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: a,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let resource = paid(&mut g, p, "Bear Cub");
        g.state.players[p.index()].mana = [0; 6];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: a,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: a,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        assert_eq!(g.object_view(resource).zone, Zone::Battlefield);
        g.state.players[p.index()].mana = [20; 6];
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Player(p.other())]);
        assert!(!g.state.stack.is_empty());
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: a,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        assert!(
            g.legal_actions().iter().any(
                |action| matches!(action,Action::AnnounceActivate{card,ability:1,..} if *card==a)
            ),
            "cash-out retains instant timing with a nonempty stack"
        );
        settle(&mut g);
        announce(&mut g, a, 0, Counters::default());
        let object = g.object_ref(resource);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        let drawn = g.state.players[p.index()].cards_drawn_this_turn;
        // Returning the same card creates a fresh exact incarnation; drawing still works.
        g.move_object(a, Zone::Hand);
        g.move_object(a, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, drawn + 1);
        assert_eq!(g.counter_count(a, CounterKind::Soul), 0);
        // Zero-counter cash-out is legal and loses zero life.
        let life = g.state.players[p.other().index()].life;
        announce(&mut g, a, 1, Counters::default());
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, life);
    }
}

#[test]
fn fdn_juggernaut_paid_full_card_mandatory_attack_wall_evasion_and_normal_combat() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let j = paid(&mut g, p, "Juggernaut");
        let v = g.object_view(j);
        assert_eq!((v.power, v.toughness), (5, 3));
        assert_eq!(v.name(), "Juggernaut");
        assert!(v.card.is_artifact());
        let wall = paid(&mut g, p, "Gleaming Barrier");
        g.state.objects[wall as usize].controller = p.other();
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::FinishAttackers);
        assert!(
            g.state.attackers.is_empty(),
            "summoning sickness makes attack impossible"
        );
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        g.state.objects[j as usize].summoning_sick = false;
        reject(&mut g, Action::FinishAttackers);
        checked(&mut g, Action::Attack { card: j });
        checked(&mut g, Action::FinishAttackers);
        assert!(g.object_view(j).tapped);
        while g.state.pending == Pending::Priority {
            checked(&mut g, Action::Pass);
        }
        reject(
            &mut g,
            Action::Block {
                blocker: wall,
                attacker: j,
            },
        );
        checked(
            &mut g,
            Action::Block {
                blocker: bear,
                attacker: j,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(g.object_view(j).zone, Zone::Battlefield);
        assert_eq!(g.object_view(j).damage, 2);
    }
}
