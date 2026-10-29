//! Noncombat damage history and two gated Reality Fracture cards.
use super::*;

#[test]
fn noncombat_history_tracks_actual_player_damage_and_rolls_over() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let repriser = put(&mut g, p, "Grim Repriser", Zone::Graveyard);
        let action = Action::AnnounceActivate {
            card: repriser,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, action.clone());
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p.other())));
        settle(&mut g);
        assert!(g.observe(p).players[p.other().index()].noncombat_damage_this_turn);
        assert!(g.observe(p.other()).players[p.other().index()].noncombat_damage_this_turn);
        assert!(!g.observe(p).players[p.index()].noncombat_damage_this_turn);
        super::super::contracts::branch_contract(g.clone());
        act(&mut g, action);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(repriser).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(repriser, CounterKind::Finality), 1);
        g.finish_cleanup();
        assert!(!g.observe(p).players[p.other().index()].noncombat_damage_this_turn);
        assert!(g.observe(p).players[p.other().index()].noncombat_damage_last_turn);
        g.finish_cleanup();
        assert!(!g.observe(p).players[p.other().index()].noncombat_damage_last_turn);
    }
}

#[test]
fn command_stage_counters_other_wizard_tokens_and_returns_next_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.create_tokens(
            p,
            Card::from_name("@token/adcb48b581ad80b8654a11b9").unwrap(),
            1,
            false,
        );
        let existing = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && o.is_token()
                    && o.controller == p
                    && o.card == Card::from_name("@token/adcb48b581ad80b8654a11b9").unwrap())
                .then_some(id as CardId)
            })
            .unwrap();
        let spell = put(&mut g, p, "Command the Stage", Zone::Hand);
        cast(&mut g, spell, None);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        let tokens = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && o.is_token()
                    && o.controller == p
                    && o.card == Card::from_name("@token/adcb48b581ad80b8654a11b9").unwrap())
                .then_some(id as CardId)
            })
            .collect::<Vec<_>>();
        assert_eq!(tokens.len(), 2);
        assert_eq!(g.counter_count(existing, CounterKind::PlusOne), 1);
        let new = *tokens.iter().find(|&&id| id != existing).unwrap();
        assert_eq!(g.counter_count(new, CounterKind::PlusOne), 0);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p.other())));
        settle(&mut g);
        g.finish_cleanup();
        g.begin_step(Step::Upkeep);
        g.prepare_triggers();
        while matches!(g.state.pending, Pending::OrderTriggers { .. }) {
            choose(&mut g, |action| matches!(action, Action::PutTrigger { .. }));
        }
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Hand);
    }
}

#[test]
fn noncombat_fact_distinguishes_player_combat_and_zero_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let repriser = put(&mut g, p, "Grim Repriser", Zone::Graveyard);
        let action = Action::AnnounceActivate {
            card: repriser,
            ability: 0,
            counter_payment: Counters::default(),
        };
        let event = |target, amount, combat| DamageEvent {
            preventable: true,
            source: g.damage_source(source),
            target,
            amount,
            combat,
        };
        let events = [
            event(Target::Player(p.other()), 0, false),
            event(Target::Player(p.other()), 1, true),
            event(Target::Card(source), 1, false),
            event(Target::Player(p), 1, false),
        ];
        g.apply_damage_batch(&events);
        assert!(g.observe(p).players[p.index()].noncombat_damage_this_turn);
        assert!(!g.observe(p).players[p.other().index()].noncombat_damage_this_turn);
        reject(&mut g, action);
    }
}

fn finish_damage_replacements(g: &mut Game) {
    if g.state.replacements.is_some() {
        g.advance_replacements(true);
    }
    for _ in 0..32 {
        if g.state.replacements.is_none() {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::ChooseReplacement { .. } | Action::ChooseDamagePrevention { .. }
            )
        });
    }
    panic!("damage replacements did not finish");
}

#[test]
fn noncombat_history_uses_actual_prevention_and_replaced_damage() {
    for p in [Player::P0, Player::P1] {
        for unpreventable in [false, true] {
            let mut g = scenario(p);
            let dealer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let prevention = put(&mut g, p.other(), "Forest", Zone::Battlefield);
            let source = g.object_ref(prevention);
            let recipient = g.object_ref(dealer);
            let timestamp = g.next_role_timestamp();
            g.state
                .roles
                .get_or_insert_default()
                .source_linked_effects
                .push(SourceLinkedEffectView {
                    source,
                    recipient,
                    controller: p.other(),
                    kind: SourceLinkedEffectKind::PreventDamageBy,
                    timestamp,
                });
            let repriser = put(&mut g, p, "Grim Repriser", Zone::Graveyard);
            let activation = Action::AnnounceActivate {
                card: repriser,
                ability: 0,
                counter_payment: Counters::default(),
            };
            let life = g.state.players[p.other().index()].life;
            g.apply_damage_batch(&[DamageEvent {
                source: g.damage_source(dealer),
                target: Target::Player(p.other()),
                amount: 2,
                combat: false,
                preventable: !unpreventable,
            }]);
            finish_damage_replacements(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                life - if unpreventable { 2 } else { 0 }
            );
            for viewer in [p, p.other()] {
                assert_eq!(
                    g.observe(viewer).players[p.other().index()].noncombat_damage_this_turn,
                    unpreventable
                );
            }
            if unpreventable {
                g.state.players[p.index()].mana = [0; 6];
                reject(&mut g, activation.clone());
                assert_eq!(g.object_view(repriser).zone, Zone::Graveyard);
                g.state.players[p.index()].mana = [0, 0, 1, 1, 0, 0];
                act(&mut g, activation);
                pay(&mut g);
                settle(&mut g);
                assert_eq!(g.counter_count(repriser, CounterKind::Finality), 1);
            } else {
                reject(&mut g, activation);
            }
        }
        let mut g = scenario(p);
        put(&mut g, p, "Twinflame Tyrant", Zone::Battlefield);
        let dealer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let life = g.state.players[p.other().index()].life;
        g.apply_damage_batch(&[DamageEvent {
            source: g.damage_source(dealer),
            target: Target::Player(p.other()),
            amount: 1,
            combat: false,
            preventable: true,
        }]);
        finish_damage_replacements(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, life - 2);
        assert!(g.observe(p.other()).players[p.other().index()].noncombat_damage_this_turn);
    }
}

#[test]
fn noncombat_history_mixed_batch_is_per_recipient_and_replays_through_turns() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.apply_damage_batch(&[
            DamageEvent {
                source: g.damage_source(a),
                target: Target::Player(p.other()),
                amount: 2,
                combat: true,
                preventable: true,
            },
            DamageEvent {
                source: g.damage_source(b),
                target: Target::Player(p),
                amount: 1,
                combat: false,
                preventable: true,
            },
        ]);
        for viewer in [p, p.other()] {
            assert!(g.observe(viewer).players[p.index()].noncombat_damage_this_turn);
            assert!(!g.observe(viewer).players[p.other().index()].noncombat_damage_this_turn);
        }
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            g.finish_cleanup();
            for viewer in [p, p.other()] {
                assert!(g.observe(viewer).players[p.index()].noncombat_damage_last_turn);
                assert!(!g.observe(viewer).players[p.other().index()].noncombat_damage_last_turn);
                assert!(!g.observe(viewer).players[p.index()].noncombat_damage_this_turn);
            }
            g.finish_cleanup();
            assert!(!g.observe(p).players[p.index()].noncombat_damage_last_turn);
        }
    }
}

#[test]
fn command_stage_doubled_cadets_exclude_new_tokens_and_include_old_wizards_only() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.create_tokens(
            p,
            Card::from_name("@token/adcb48b581ad80b8654a11b9").unwrap(),
            1,
            false,
        );
        g.create_tokens(
            p.other(),
            Card::from_name("@token/adcb48b581ad80b8654a11b9").unwrap(),
            1,
            false,
        );
        let old: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && o.is_token()
                    && o.card == Card::from_name("@token/adcb48b581ad80b8654a11b9").unwrap())
                .then_some((id as CardId, o.controller))
            })
            .collect();
        assert_eq!(old.len(), 2);
        let non_token = put(&mut g, p, "Archmage of Runes", Zone::Battlefield);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let spell = put(&mut g, p, "Command the Stage", Zone::Hand);
        cast(&mut g, spell, None);
        super::super::contracts::branch_contract(g.clone());
        settle(&mut g);
        for (id, owner) in &old {
            assert_eq!(
                g.counter_count(*id, CounterKind::PlusOne),
                if *owner == p { 2 } else { 0 }
            );
        }
        assert_eq!(g.counter_count(non_token, CounterKind::PlusOne), 0);
        let created: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && o.is_token()
                    && o.controller == p
                    && o.card == Card::from_name("@token/adcb48b581ad80b8654a11b9").unwrap()
                    && old.iter().all(|(old_id, _)| *old_id != id as CardId))
                .then_some(id as CardId)
            })
            .collect();
        assert_eq!(created.len(), 2);
        for id in created {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
            assert_eq!(g.object_view(id).owner, p);
            assert_eq!(g.object_view(id).colors, 0);
            assert!(g.has_subtype(id, "Wizard") && g.has_subtype(id, "Soldier"));
            assert_eq!(stats(&g, id), (2, 2));
        }
    }
}

#[test]
fn command_stage_upkeep_needs_last_turn_and_exact_graveyard_incarnation() {
    for p in [Player::P0, Player::P1] {
        for reenter in [false, true] {
            let mut g = scenario(p);
            let spell = put(&mut g, p, "Command the Stage", Zone::Graveyard);
            let dealer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.apply_damage_batch(&[DamageEvent {
                source: g.damage_source(dealer),
                target: Target::Player(p.other()),
                amount: 1,
                combat: false,
                preventable: true,
            }]);
            g.finish_cleanup();
            g.state.active = p.other();
            g.begin_step(Step::Upkeep);
            g.prepare_triggers();
            if reenter {
                g.move_object(spell, Zone::Exile);
                g.move_object(spell, Zone::Graveyard);
            }
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                while matches!(g.state.pending, Pending::OrderTriggers { .. }) {
                    choose(&mut g, |action| matches!(action, Action::PutTrigger { .. }));
                }
                settle(&mut g);
                assert_eq!(
                    g.object_view(spell).zone,
                    if reenter { Zone::Graveyard } else { Zone::Hand }
                );
            }
        }
    }
}
