use super::*;

fn draft(g: &mut Game, owner: Player, name: &str) -> CardId {
    let id = put(g, owner, "Bear Cub", Zone::Battlefield);
    g.state.objects[id as usize].card = Card::from_name(name).unwrap();
    g.register_battlefield_sources(id);
    id
}

fn finish(g: &mut Game) {
    for _ in 0..64 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
            && g.state.resolving.is_none()
            && g.state.replacements.is_none()
        {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::ChooseReplacement { .. }
                    | Action::ChooseDamagePrevention { .. }
            )
        });
    }
    panic!("unfinished decision: {:?}", g.state.pending);
}

#[test]
fn barbs_prevented_damage_does_not_trigger_and_simultaneous_sources_trigger_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let master = draft(&mut g, p, "Master of Barbs");
        let friend = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let dealer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let other = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
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
        let life = g.state.players[p.other().index()].life;
        g.apply_damage_batch(&[DamageEvent {
            source: g.damage_source(dealer),
            target: Target::Player(p.other()),
            amount: 2,
            combat: false,
            preventable: true,
        }]);
        if g.state.replacements.is_some() {
            g.advance_replacements(true);
        }
        while matches!(g.state.pending, Pending::Replacement { .. }) {
            choose(&mut g, |a| {
                matches!(
                    a,
                    Action::ChooseReplacement { .. } | Action::ChooseDamagePrevention { .. }
                )
            });
        }
        assert_eq!(g.state.players[p.other().index()].life, life);
        assert!(g.state.pending_triggers.is_empty());
        g.move_object(prevention, Zone::Exile);
        g.apply_damage_batch(&[
            DamageEvent {
                source: g.damage_source(dealer),
                target: Target::Player(p.other()),
                amount: 1,
                combat: false,
                preventable: true,
            },
            DamageEvent {
                source: g.damage_source(other),
                target: Target::Player(p.other()),
                amount: 1,
                combat: false,
                preventable: true,
            },
        ]);
        assert_eq!(g.state.pending_triggers.len(), 1);
        let reference = g.object_ref(master);
        g.change_controller(reference, p.other());
        super::contracts::branch_contract(g.clone());
        g.prepare_triggers();
        finish(&mut g);
        assert_eq!(stats(&g, friend), (3, 2));
        assert_eq!(stats(&g, enemy), (2, 2));
        g.finish_cleanup();
        assert_eq!(stats(&g, friend), (2, 2));
    }
}

#[test]
fn gardenize_simultaneous_token_deaths_replacements_and_departed_source() {
    for p in [Player::P0, Player::P1] {
        for (doubled, replaced, source_leaves) in [
            (false, false, false),
            (true, false, false),
            (false, true, false),
            (false, false, true),
        ] {
            let mut g = scenario(p);
            let garden = draft(&mut g, p, "Gardenize");
            if doubled {
                put(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            if replaced {
                put(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
            }
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.create_tokens(p, Card::test_index(5298), 1, false);
            if g.state.replacements.is_some() {
                g.advance_replacements(true);
            }
            while matches!(g.state.pending, Pending::Replacement { .. }) {
                choose(&mut g, |a| matches!(a, Action::ChooseReplacement { .. }));
            }
            let token = g
                .state
                .objects
                .iter()
                .enumerate()
                .find_map(|(i, o)| {
                    (o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.is_token()
                        && o.card == Card::test_index(5298))
                    .then_some(i as CardId)
                })
                .unwrap();
            let observers = g.death_trigger_sources();
            let mut ids = vec![bear, token];
            if source_leaves {
                ids.push(garden);
            }
            let events = g.move_objects_raw(&ids, Zone::Graveyard);
            g.finish_zone_events(&events, &observers);
            assert_eq!(g.state.pending_triggers.len(), if replaced { 0 } else { 2 });
            super::contracts::branch_contract(g.clone());
            g.prepare_triggers();
            finish(&mut g);
            if source_leaves {
                assert_eq!(g.object_view(garden).zone, Zone::Graveyard);
            } else {
                assert_eq!(
                    g.counter_count(garden, CounterKind::Charge),
                    if replaced {
                        0
                    } else if doubled {
                        4
                    } else {
                        2
                    }
                );
            }
            assert_eq!(
                g.object_view(bear).zone,
                if replaced {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}

#[test]
fn gardenize_main_phase_follows_current_controller_and_zero_counters() {
    for p in [Player::P0, Player::P1] {
        for amount in [0, 2] {
            let mut g = scenario(p);
            let garden = draft(&mut g, p, "Gardenize");
            if amount > 0 {
                g.add_counter(garden, CounterKind::Charge, amount);
            }
            let reference = g.object_ref(garden);
            g.change_controller(reference, p.other());
            g.state.active = p.other();
            g.state.priority = p.other();
            g.state.players[p.index()].mana = [0; 6];
            g.state.players[p.other().index()].mana = [0; 6];
            g.begin_step(Step::Main1);
            g.prepare_triggers();
            finish(&mut g);
            assert_eq!(g.state.players[p.other().index()].mana[4], amount as u32);
            assert_eq!(g.state.players[p.index()].mana[4], 0);
        }
    }
}
