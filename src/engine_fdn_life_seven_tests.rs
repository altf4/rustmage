use super::tests::{act, pass_pair, scenario, spawn};
use super::*;
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for s in &mut g.state.players {
        s.mana = [20; 6];
    }
    g
}
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn replay(g: &mut Game, a: Action) {
    let p = g.actor().unwrap();
    let revision = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|b| *b == a)
        .expect("legal");
    let before = g.snapshot();
    assert_eq!(g.apply(p.other(), a.clone()), Err(Error::WrongActor));
    assert_eq!(g.snapshot(), before);
    let mut raw = {
        let parent = g.clone();
        parent.clone()
    };
    let mut indexed = {
        let parent = g.clone();
        parent.clone()
    };
    raw.apply(p, a.clone()).unwrap();
    indexed.apply_index(p, revision, index).unwrap();
    act(g, a.clone());
    assert_eq!(g.snapshot(), raw.snapshot());
    assert_eq!(g.snapshot(), indexed.snapshot());
    for viewer in [p, p.other()] {
        assert_eq!(
            serde_json::to_value(g.observe(viewer)).unwrap(),
            serde_json::to_value(indexed.observe(viewer)).unwrap()
        );
    }
    let after = g.snapshot();
    g.restore(&before).unwrap();
    assert_eq!(g.apply_index(p, revision, index), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), after);
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
fn pay(g: &mut Game) {
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .expect("payment");
        replay(g, a);
    }
}
fn settle(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..150 {
        if g.outcome().is_some() || g.state.pending == Pending::Priority && g.state.stack.is_empty()
        {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::PutTrigger { .. }
                        | Action::Pass
                        | Action::ChooseTarget {
                            target: Target::Player(_)
                        }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                )
            })
            .expect("settle action");
        replay(g, a);
    }
    panic!("unsettled");
}
fn cast(g: &mut Game, p: Player, n: &str) -> CardId {
    let id = named(g, p, n, Zone::Hand);
    replay(g, Action::AnnounceCast { card: id });
    pay(g);
    settle(g);
    id
}
fn count(g: &Game, id: CardId) -> usize {
    g.state
        .pending_triggers
        .iter()
        .chain(&g.state.ordering_triggers)
        .chain(&g.state.stack)
        .filter(|s| s.card == id)
        .count()
}
#[test]
fn fdn_life_seven_angel_replacements_threshold_and_cindermaw_prohibition() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 2];
        let a = cast(&mut g, p, "Angel of Vitality");
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let b = named(&mut g, p, "Angel of Vitality", Zone::Battlefield);
        g.gain_life(p, 0);
        assert_eq!(g.state.players[p.index()].life, 20);
        g.gain_life(p, 3);
        assert_eq!(g.state.players[p.index()].life, 25);
        assert_eq!(
            g.stats(a),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        g.lose_life(p, 1);
        assert_eq!(
            g.stats(a),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        g.gain_life(p.other(), 1);
        assert_eq!(g.state.players[p.other().index()].life, 21);
        let c = named(&mut g, p.other(), "Giant Cindermaw", Zone::Battlefield);
        let history = g.state.players[p.index()].life_gain_events_this_turn;
        g.gain_life(p, 100);
        g.gain_life(p.other(), 100);
        assert_eq!(g.state.players[p.index()].life, 24);
        assert_eq!(g.state.players[p.other().index()].life, 21);
        assert_eq!(
            g.state.players[p.index()].life_gain_events_this_turn,
            history
        );
        g.move_object(c, Zone::Graveyard);
        g.move_object(b, Zone::Graveyard);
        g.gain_life(p, 1);
        assert_eq!(g.state.players[p.index()].life, 26);
    }
}
#[test]
fn fdn_life_seven_conqueror_actual_loss_damage_batch_and_departed_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = cast(&mut g, p, "Bloodthirsty Conqueror");
        g.lose_life(p, 2);
        g.lose_life(p.other(), 0);
        assert_eq!(count(&g, source), 0);
        g.lose_life(p.other(), 4);
        assert_eq!(count(&g, source), 1);
        assert_eq!(g.state.pending_triggers[0].life_event.unwrap().amount, 4);
        g.move_object(source, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 22);
        g.move_object(source, Zone::Battlefield);
        let dealer = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let damage = g.damage_source(dealer);
        g.apply_damage_batch(&[
            DamageEvent {
                preventable: true,
                source: damage,
                target: Target::Player(p.other()),
                amount: 2,
                combat: true,
            },
            DamageEvent {
                preventable: true,
                source: damage,
                target: Target::Player(p.other()),
                amount: 3,
                combat: true,
            },
        ]);
        assert_eq!(count(&g, source), 1);
        assert_eq!(g.state.pending_triggers[0].life_event.unwrap().amount, 5);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 27);
        let c = named(&mut g, p, "Giant Cindermaw", Zone::Battlefield);
        g.lose_life(p.other(), 1);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 27);
        assert!(g.keywords(source).contains(Keywords::DEATHTOUCH));
        g.move_object(c, Zone::Graveyard);
    }
}
#[test]
fn fdn_life_seven_exemplar_counter_event_once_per_turn_and_private_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = cast(&mut g, p, "Exemplar of Light");
        let library = g.state.players[p.index()].library.len();
        let hand = g.hand_count(p);
        let drawn = *g.state.players[p.index()].library.last().unwrap();
        g.gain_life(p, 3);
        settle(&mut g);
        assert_eq!(g.object_view(source).counters.plus_one, 1);
        assert_eq!(g.state.players[p.index()].library.len(), library - 1);
        assert_eq!(g.hand_count(p), hand + 1);
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == drawn));
        assert!(
            !g.observe(p.other())
                .known_hand
                .iter()
                .any(|o| o.card == drawn)
        );
        g.gain_life(p, 1);
        settle(&mut g);
        assert_eq!(g.object_view(source).counters.plus_one, 2);
        assert_eq!(g.state.players[p.index()].library.len(), library - 1);
        g.state.active = p.other();
        g.begin_turn();
        g.state.step = Step::Main1;
        g.state.priority = p.other();
        g.state.pending = Pending::Priority;
        g.effect_counter(
            source,
            CounterKind::PlusOne,
            3,
            replacements::ResumeMode::Pending,
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), library - 2);
        g.move_object(source, Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        g.effect_counter(
            source,
            CounterKind::PlusOne,
            1,
            replacements::ResumeMode::Pending,
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), library - 3);
    }
}
#[test]
fn fdn_life_seven_panda_lki_power_excludes_bears_self_foreign_and_stale_targets() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let panda = cast(&mut g, p, "Fiendish Panda");
        g.gain_life(p, 1);
        settle(&mut g);
        assert_eq!(g.object_view(panda).power, 4);
        let target = named(&mut g, p, "Serra Angel", Zone::Graveyard); // MV5 excluded at power4.
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
        let changeling = named(&mut g, p, "Three Tree Mascot", Zone::Graveyard);
        let own = named(&mut g, p, "Exemplar of Light", Zone::Graveyard);
        let enemy = named(&mut g, p.other(), "Dazzling Angel", Zone::Graveyard);
        g.move_object(panda, Zone::Graveyard);
        g.prepare_triggers();
        if let Some(action) = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PutTrigger { .. }))
        {
            replay(&mut g, action);
        }
        for invalid in [panda, target, bear, changeling, enemy] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(invalid),
                },
            );
        }
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        pay(&mut g);
        g.move_object(panda, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.state.objects[own as usize].zone, Zone::Battlefield);
        assert_eq!(g.object_view(panda).counters.plus_one, 0);
        g.move_object(own, Zone::Graveyard);
        g.state.objects[panda as usize].growth.power = 1;
        g.move_object(panda, Zone::Graveyard);
        g.prepare_triggers();
        if let Some(action) = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PutTrigger { .. }))
        {
            replay(&mut g, action);
        }
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        pay(&mut g);
        g.move_object(own, Zone::Exile);
        g.move_object(own, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.objects[own as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn fdn_life_seven_bloodwitch_sees_simultaneous_own_deaths_and_target_fizzle() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let witch = cast(&mut g, p, "Vengeful Bloodwitch");
        let a = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let b = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.sacrifice_batch(&[(p, witch), (p, a), (p, b)]);
        assert_eq!(count(&g, witch), 3);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 23);
        assert_eq!(g.state.players[p.other().index()].life, 17);
    }
}
#[test]
fn fdn_life_seven_snack_paid_sacrifice_late_amount_and_raid_food() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let snack = cast(&mut g, p, "Midnight Snack");
        g.gain_life(p, 2);
        g.lose_life(p, 4);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: snack,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        assert_eq!(g.state.objects[snack as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.gain_life(p, 3);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 15);
        let snack = named(&mut g, p, "Midnight Snack", Zone::Battlefield);
        g.state.players[p.index()].attacked_this_turn = true;
        g.state.step = Step::Main2;
        g.give_priority();
        pass_pair(&mut g);
        settle(&mut g);
        assert_eq!(g.state.step, Step::End);
        let food = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Battlefield && o.card.has_subtype("Food"))
            .map(|(id, _)| id as CardId)
            .expect("raid Food");
        assert!(g.state.objects[snack as usize].zone == Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        g.state.priority = p;
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: food,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.objects[food as usize].zone, Zone::Outside);
    }
}
#[test]
fn fdn_life_seven_all_printed_mana_costs_and_intrinsic_keywords() {
    for p in [Player::P0, Player::P1] {
        for (name, cost, keywords) in [
            ("Angel of Vitality", [1, 0, 0, 0, 0, 2], Keywords::FLYING),
            (
                "Bloodthirsty Conqueror",
                [0, 0, 2, 0, 0, 3],
                Keywords::FLYING.union(Keywords::DEATHTOUCH),
            ),
            ("Giant Cindermaw", [0, 0, 0, 1, 0, 2], Keywords::TRAMPLE),
            ("Exemplar of Light", [2, 0, 0, 0, 0, 2], Keywords::FLYING),
            ("Fiendish Panda", [1, 0, 1, 0, 0, 2], Keywords::NONE),
            ("Vengeful Bloodwitch", [0, 0, 1, 0, 0, 1], Keywords::NONE),
            ("Midnight Snack", [0, 0, 1, 0, 0, 2], Keywords::NONE),
        ] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = cost;
            let card = cast(&mut g, p, name);
            assert_eq!(g.state.players[p.index()].mana, [0; 6], "{name}");
            assert!(g.keywords(card).contains(keywords));
        }
    }
}
fn stop_cold(g: &mut Game, p: Player, id: CardId) {
    g.state.priority = p;
    g.state.players[p.index()].mana = [20; 6];
    let spell = named(g, p, "Stop Cold", Zone::Hand);
    replay(g, Action::AnnounceCast { card: spell });
    replay(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
    pay(g);
    settle(g);
}
#[test]
fn fdn_life_seven_rule_fragments_and_life_observers_respect_layer_six_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let angel = cast(&mut g, p, "Angel of Vitality");
        g.gain_life(p, 4);
        assert_eq!(
            g.stats(angel),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        stop_cold(&mut g, p, angel);
        assert_eq!(
            g.stats(angel),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(!g.keywords(angel).contains(Keywords::FLYING));
        g.gain_life(p, 1);
        assert_eq!(g.state.players[p.index()].life, 26);
        let cinder = named(&mut g, p, "Giant Cindermaw", Zone::Battlefield);
        g.gain_life(p, 2);
        assert_eq!(g.state.players[p.index()].life, 26);
        stop_cold(&mut g, p, cinder);
        g.gain_life(p, 2);
        assert_eq!(g.state.players[p.index()].life, 28);
        let conqueror = named(&mut g, p, "Bloodthirsty Conqueror", Zone::Battlefield);
        stop_cold(&mut g, p, conqueror);
        g.lose_life(p.other(), 2);
        assert_eq!(count(&g, conqueror), 0);
        let exemplar = named(&mut g, p, "Exemplar of Light", Zone::Battlefield);
        stop_cold(&mut g, p, exemplar);
        let library = g.state.players[p.index()].library.len();
        g.gain_life(p, 2);
        settle(&mut g);
        assert_eq!(g.object_view(exemplar).counters.plus_one, 0);
        g.effect_counter(
            exemplar,
            CounterKind::PlusOne,
            1,
            replacements::ResumeMode::Pending,
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), library);
    }
}
#[test]
fn fdn_life_seven_counter_attribution_entry_and_overflow_atomicity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let exemplar = cast(&mut g, p, "Exemplar of Light");
        let library = g.state.players[p.index()].library.len();
        let object = g.object_view(exemplar);
        g.stage_counter_placements(
            vec![replacements::CounterPlacement {
                object: object.reference,
                placed_by: p.other(),
                card: object.card,
                owner: object.owner,
                controller: p,
                subtypes: object.granted_subtypes,
                kind: 1,
                amount: Counters {
                    plus_one: 3,
                    ..Counters::default()
                },
                effect: true,
                entry_index: None,
                private: false,
            }],
            replacements::ResumeMode::Pending,
        );
        assert_eq!(count(&g, exemplar), 0);
        assert_eq!(g.object_view(exemplar).counters.plus_one, 3);
        g.effect_counter(
            exemplar,
            CounterKind::PlusOne,
            1,
            replacements::ResumeMode::Pending,
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), library - 1);
        let entered = named(&mut g, p, "Exemplar of Light", Zone::Hand);
        let mut movement = g.entry_move(entered, Zone::Battlefield, false);
        movement.effect_counter = Some((CounterKind::PlusOne, 2));
        g.stage_entries(vec![movement], None);
        assert!(!g.advance_entries());
        settle(&mut g);
        assert_eq!(g.object_view(entered).counters.plus_one, 2);
        assert_eq!(g.state.players[p.index()].library.len(), library - 2);
        g.state.objects[entered as usize].counters.plus_one = u64::MAX;
        g.gain_life(p, 1);
        g.prepare_triggers();
        while !g.state.ordering_triggers.is_empty() {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PutTrigger { .. }))
                .unwrap();
            replay(&mut g, a);
        }
        // The top counter trigger must fail atomically at resolution.
        replay(&mut g, Action::Pass);
        let before = g.snapshot();
        let rev = g.revision();
        let actor = g.actor().unwrap();
        assert!(matches!(
            g.apply(actor, Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), rev);
    }
}
#[test]
fn fdn_life_seven_bloodwitch_hexproof_fizzles_whole_trigger_and_snack_raid_guard() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let witch = cast(&mut g, p, "Vengeful Bloodwitch");
        g.move_object(witch, Zone::Graveyard);
        g.prepare_triggers();
        if let Some(a) = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PutTrigger { .. }))
        {
            replay(&mut g, a);
        }
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        g.state.players[p.other().index()].hexproof_until_end_turn = true;
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        let snack = named(&mut g, p, "Midnight Snack", Zone::Battlefield);
        g.begin_step(Step::End);
        assert_eq!(count(&g, snack), 0);
        g.state.players[p.index()].attacked_this_turn = true;
        g.state.active = p.other();
        g.begin_step(Step::End);
        assert_eq!(count(&g, snack), 0);
        g.state.active = p;
        g.begin_step(Step::End);
        assert_eq!(count(&g, snack), 1);
        g.state.players[p.index()].attacked_this_turn = false;
        settle(&mut g);
        assert!(
            !g.state
                .objects
                .iter()
                .any(|o| o.zone == Zone::Battlefield && o.card.has_subtype("Food"))
        );
    }
}
