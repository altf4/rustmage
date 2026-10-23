//! Morcant's paid Elf costs and public non-targeting proliferate decisions.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn pay(g: &mut Game) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, action);
}
fn source_and_elves(g: &mut Game, p: Player) -> [CardId; 3] {
    let ids = [
        named(g, p, "High Perfect Morcant", Zone::Battlefield),
        named(g, p, "Morcant's Eyes", Zone::Battlefield),
        named(g, p, "Llanowar Elves", Zone::Battlefield),
    ];
    for &id in &ids {
        g.state.objects[id as usize].summoning_sick = true;
    }
    ids
}
fn activate(g: &mut Game, elves: [CardId; 3], leave_source: bool) {
    checked(
        g,
        Action::AnnounceActivate {
            card: elves[0],
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    for id in elves {
        let object = g.object_ref(id);
        checked(g, Action::ChooseAdditionalCost { object });
    }
    pay(g);
    for id in elves {
        assert!(g.object_view(id).tapped);
    }
    if leave_source {
        g.move_object(elves[0], Zone::Exile);
    }
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    assert!(matches!(g.state.pending, Pending::Proliferate { .. }));
}
#[test]
fn ecl_morcant_paid_body_self_and_other_kindred_elf_entry_blight_opponent_only() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = named(&mut g, p.other(), "Skybeast Tracker", Zone::Battlefield);
        let source = named(&mut g, p, "High Perfect Morcant", Zone::Hand);
        let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            mana - 4
        );
        assert_eq!(
            g.stats(source),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        assert_eq!(g.counter_count(victim, CounterKind::MinusOne), 1);
        let eyes = named(&mut g, p, "Morcant's Eyes", Zone::Hand);
        cast(&mut g, eyes, None, &[]);
        settle(&mut g);
        assert!(!g.is_creature(eyes));
        assert_eq!(g.counter_count(victim, CounterKind::MinusOne), 2);
        let own_nonelf = named(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, own_nonelf, None, &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(victim, CounterKind::MinusOne), 2);
        let elf = named(&mut g, p.other(), "Llanowar Elves", Zone::Hand);
        g.state.active = p.other();
        g.state.priority = p.other();
        cast(&mut g, elf, None, &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(victim, CounterKind::MinusOne), 2);
        assert_eq!(g.counter_count(elf, CounterKind::MinusOne), 0);
    }
}
#[test]
fn ecl_morcant_entry_without_eligible_opposing_creature_resolves_and_departed_source_keeps_trigger()
{
    for p in [Player::P0, Player::P1] {
        for prohibited in [false, true] {
            let mut g = setup(p);
            let victim = if prohibited {
                let victim = named(&mut g, p.other(), "Skybeast Tracker", Zone::Battlefield);
                let aura = named(&mut g, p, "Blossombind", Zone::Hand);
                cast(&mut g, aura, None, &[Target::Card(victim)]);
                settle(&mut g);
                Some(victim)
            } else {
                None
            };
            let source = named(&mut g, p, "High Perfect Morcant", Zone::Hand);
            cast(&mut g, source, None, &[]);
            settle(&mut g);
            assert!(g.on_battlefield(source));
            if let Some(victim) = victim {
                assert_eq!(g.counter_count(victim, CounterKind::MinusOne), 0);
            }
        }
        let mut g = setup(p);
        let victim = named(&mut g, p.other(), "Skybeast Tracker", Zone::Battlefield);
        let source = named(&mut g, p, "High Perfect Morcant", Zone::Hand);
        cast(&mut g, source, None, &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        g.move_object(source, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.counter_count(victim, CounterKind::MinusOne), 1);
    }
}
#[test]
fn ecl_morcant_tap_cost_accepts_new_kindred_elves_ignores_sickness_and_enforces_untapped_distinct_own_elves()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elves = source_and_elves(&mut g, p);
        let own_nonelf = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        let action = Action::AnnounceActivate {
            card: elves[0],
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.step = Step::End;
        reject(&mut g, action.clone());
        g.state.step = Step::Main1;
        g.state.priority = p.other();
        reject(&mut g, action.clone());
        g.state.priority = p;
        g.state.objects[elves[1] as usize].tapped = true;
        reject(&mut g, action.clone());
        g.state.objects[elves[1] as usize].tapped = false;
        checked(&mut g, action);
        for id in [own_nonelf, enemy] {
            let object = g.object_ref(id);
            reject(&mut g, Action::ChooseAdditionalCost { object });
        }
        let object = g.object_ref(elves[0]);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        reject(&mut g, Action::ChooseAdditionalCost { object });
        assert!(!g.object_view(elves[0]).tapped);
        for &id in &elves[1..] {
            let object = g.object_ref(id);
            checked(&mut g, Action::ChooseAdditionalCost { object });
        }
        pay(&mut g);
        for id in elves {
            assert!(g.object_view(id).tapped);
        }
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        for viewer in [p, p.other()] {
            let view = g.observe(viewer).effect_choice.unwrap();
            assert!(view.optional);
            assert!(view.proliferate.unwrap().candidate_players.is_empty());
        }
        checked(&mut g, Action::FinishProliferate);
        settle(&mut g);
    }
}
#[test]
fn ecl_morcant_proliferate_all_named_kinds_is_nontargeting_and_one_mixed_replacement_event() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elves = source_and_elves(&mut g, p);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let own = named(&mut g, p, "Springleaf Drum", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Springleaf Drum", Zone::Battlefield);
        let mut counters = Counters::default();
        for kind in CounterKind::ALL {
            if kind != CounterKind::MinusOne {
                counters.add(kind, 2);
            }
        }
        g.set_counters(own, counters);
        g.set_counters(enemy, Counters::of(CounterKind::MinusOne, 2));
        let source = g.object_ref(elves[0]);
        g.grant_keyword_effects(&[enemy], source, Keywords::HEXPROOF);
        activate(&mut g, elves, false);
        let zero = g.object_ref(elves[0]);
        reject(&mut g, Action::ProliferatePermanent { object: zero });
        for id in [own, enemy] {
            let object = g.object_ref(id);
            checked(&mut g, Action::ProliferatePermanent { object });
            reject(&mut g, Action::ProliferatePermanent { object });
        }
        let object = ObjectRef {
            card: own,
            generation: g.object_ref(own).generation + 1,
        };
        reject(&mut g, Action::ProliferatePermanent { object });
        checked(&mut g, Action::FinishProliferate);
        let choice = g.observe(p).replacement_choice.unwrap();
        assert_eq!(choice.effects.len(), 2);
        let amounts = choice.counters.unwrap();
        for kind in CounterKind::ALL {
            assert_eq!(amounts.get(kind), u64::from(kind != CounterKind::MinusOne));
        }
        settle(&mut g);
        for kind in CounterKind::ALL {
            assert_eq!(
                g.counter_count(own, kind),
                if kind == CounterKind::MinusOne { 0 } else { 6 }
            );
        }
        assert_eq!(g.counter_count(enemy, CounterKind::MinusOne), 3);
    }
}
#[test]
fn ecl_morcant_proliferate_counter_prohibition_keeps_choice_legal_and_source_departure_does_not_cancel()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elves = source_and_elves(&mut g, p);
        let recipient = named(&mut g, p.other(), "Skybeast Tracker", Zone::Battlefield);
        g.set_counters(
            recipient,
            Counters::of(CounterKind::PlusOne, 2).with(CounterKind::Flying, 1),
        );
        let aura = named(&mut g, p, "Blossombind", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(recipient)]);
        settle(&mut g);
        activate(&mut g, elves, true);
        let object = g.object_ref(recipient);
        checked(&mut g, Action::ProliferatePermanent { object });
        checked(&mut g, Action::FinishProliferate);
        settle(&mut g);
        assert_eq!(g.counter_count(recipient, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(recipient, CounterKind::Flying), 1);
        assert_eq!(g.object_view(elves[0]).zone, Zone::Exile);
    }
}
#[test]
fn ecl_morcant_proliferate_renews_keyword_counter_timestamps_after_ability_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elves = source_and_elves(&mut g, p);
        let recipient = named(&mut g, p, "Skybeast Tracker", Zone::Battlefield);
        let mut counters = Counters::default();
        for kind in keyword_counters::KINDS {
            counters.add(kind, 1);
        }
        g.set_counters(recipient, counters);
        let aura = named(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(recipient)]);
        settle(&mut g);
        assert_eq!(g.keywords(recipient), Keywords::NONE);
        activate(&mut g, elves, true);
        let object = g.object_ref(recipient);
        checked(&mut g, Action::ProliferatePermanent { object });
        checked(&mut g, Action::FinishProliferate);
        settle(&mut g);
        for kind in keyword_counters::KINDS {
            assert_eq!(g.counter_count(recipient, kind), 2);
        }
        assert_eq!(
            g.keywords(recipient),
            Keywords::FLYING
                .union(Keywords::FIRST_STRIKE)
                .union(Keywords::LIFELINK)
        );
        assert!(!g.printed_abilities_available(recipient));
    }
}
#[test]
fn ecl_morcant_proliferate_player_poison_optional_selection_and_simultaneous_defeat() {
    for p in [Player::P0, Player::P1] {
        for selected in [0u8, 1, 2, 3] {
            let mut g = setup(p);
            let elves = source_and_elves(&mut g, p);
            g.state.players[p.index()].poison_counters = 9;
            g.state.players[p.other().index()].poison_counters = 9;
            activate(&mut g, elves, false);
            for viewer in [p, p.other()] {
                let view = g
                    .observe(viewer)
                    .effect_choice
                    .unwrap()
                    .proliferate
                    .unwrap();
                assert_eq!(view.candidate_players, vec![Player::P0, Player::P1]);
            }
            for player in [Player::P0, Player::P1] {
                if selected & (1 << player.index()) != 0 {
                    checked(&mut g, Action::ProliferatePlayer { player });
                    reject(&mut g, Action::ProliferatePlayer { player });
                }
            }
            checked(&mut g, Action::FinishProliferate);
            for player in [Player::P0, Player::P1] {
                assert_eq!(
                    g.state.players[player.index()].poison_counters,
                    9 + u64::from(selected & (1 << player.index()) != 0)
                );
            }
            match selected {
                0 => assert!(g.state.outcome.is_none()),
                1 => assert_eq!(g.state.outcome, Some(Outcome::Winner(Player::P1))),
                2 => assert_eq!(g.state.outcome, Some(Outcome::Winner(Player::P0))),
                _ => assert_eq!(g.state.outcome, Some(Outcome::Draw)),
            }
        }
    }
}
#[test]
fn ecl_morcant_proliferate_poison_loss_waits_for_counter_replacement_completion() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elves = source_and_elves(&mut g, p);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let recipient = named(&mut g, p, "Springleaf Drum", Zone::Battlefield);
        g.set_counters(recipient, Counters::of(CounterKind::Charge, 2));
        g.state.players[p.other().index()].poison_counters = 9;
        activate(&mut g, elves, false);
        let object = g.object_ref(recipient);
        checked(&mut g, Action::ProliferatePermanent { object });
        checked(&mut g, Action::ProliferatePlayer { player: p.other() });
        checked(&mut g, Action::FinishProliferate);
        assert_eq!(g.state.players[p.other().index()].poison_counters, 10);
        assert!(g.state.outcome.is_none());
        assert!(g.observe(p).replacement_choice.is_some());
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseReplacement { .. }))
            .unwrap();
        checked(&mut g, action);
        assert_eq!(g.counter_count(recipient, CounterKind::Charge), 6);
        assert_eq!(g.state.outcome, Some(Outcome::Winner(p)));
    }
}
#[test]
fn ecl_morcant_proliferate_poison_and_named_counter_overflow_restore_owned_choice() {
    for p in [Player::P0, Player::P1] {
        for poison in [false, true] {
            let mut g = setup(p);
            let elves = source_and_elves(&mut g, p);
            let recipient = named(&mut g, p, "Springleaf Drum", Zone::Battlefield);
            g.set_counters(recipient, Counters::of(CounterKind::Flying, u64::MAX));
            activate(&mut g, elves, false);
            if poison {
                g.state.players[p.other().index()].poison_counters = u64::MAX;
                checked(&mut g, Action::ProliferatePlayer { player: p.other() });
            } else {
                let object = g.object_ref(recipient);
                checked(&mut g, Action::ProliferatePermanent { object });
            }
            let before = g.snapshot();
            let revision = g.revision();
            let views =
                [g.observe(p), g.observe(p.other())].map(|v| serde_json::to_value(v).unwrap());
            assert!(matches!(
                g.apply(g.actor().unwrap(), Action::FinishProliferate),
                Err(Error::CapacityExceeded(_))
            ));
            assert_eq!(g.snapshot(), before);
            assert_eq!(g.revision(), revision);
            assert_eq!(
                [g.observe(p), g.observe(p.other())].map(|v| serde_json::to_value(v).unwrap()),
                views
            );
            assert!(matches!(g.state.pending, Pending::Proliferate { .. }));
        }
    }
}

#[test]
fn ecl_morcant_tapped_source_uses_three_other_elves_and_has_no_zero_poison_player_choices() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elves = source_and_elves(&mut g, p);
        let extra = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        g.state.objects[elves[0] as usize].tapped = true;
        let action = Action::AnnounceActivate {
            card: elves[0],
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.active = p.other();
        reject(&mut g, action.clone());
        g.state.active = p;
        checked(&mut g, action);
        for id in [elves[1], elves[2], extra] {
            let object = g.object_ref(id);
            checked(&mut g, Action::ChooseAdditionalCost { object });
        }
        pay(&mut g);
        assert!(g.object_view(elves[0]).tapped);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        for player in [Player::P0, Player::P1] {
            reject(&mut g, Action::ProliferatePlayer { player });
        }
        checked(&mut g, Action::FinishProliferate);
        settle(&mut g);
    }
}
#[test]
fn ecl_morcant_selected_counter_recipient_never_rebinds_a_reentered_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elves = source_and_elves(&mut g, p);
        let recipient = named(&mut g, p, "Springleaf Drum", Zone::Battlefield);
        g.set_counters(recipient, Counters::of(CounterKind::Charge, 2));
        activate(&mut g, elves, false);
        let old = g.object_ref(recipient);
        checked(&mut g, Action::ProliferatePermanent { object: old });
        // Exercise the exact-object continuation boundary directly; ordinary
        // game actions do not grant another priority window during this choice.
        g.move_object(recipient, Zone::Exile);
        g.move_object(recipient, Zone::Battlefield);
        g.set_counters(recipient, Counters::of(CounterKind::Charge, 7));
        assert_ne!(g.object_ref(recipient), old);
        reject(&mut g, Action::ProliferatePermanent { object: old });
        checked(&mut g, Action::FinishProliferate);
        settle(&mut g);
        assert_eq!(g.counter_count(recipient, CounterKind::Charge), 7);
    }
}
