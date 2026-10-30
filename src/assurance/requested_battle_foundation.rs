//! Partial battle foundation: targeting and damage to defense counters.
//! Defeat, protector and combat behavior remain separate unsupported work.
use super::*;

#[test]
fn any_target_accepts_battle_and_damage_removes_defense() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let battle = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let mut recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
        recipe.overrides = Some(CopyOverrides {
            kinds: Some(crate::cards::BATTLE_KIND),
            ..CopyOverrides::NONE
        });
        g.apply_temporary_copy(g.object_ref(battle), recipe, CopyDeadline::EndOfTurn);
        assert_eq!(g.current_kinds(battle), crate::cards::BATTLE_KIND);
        g.add_counter(battle, CounterKind::Defense, 5);
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        big_helpers::checked(&mut g, Action::AnnounceCast { card: spell });
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(battle),
            },
        );
        pay(&mut g);
        big_helpers::settle(&mut g);
        assert_eq!(g.counter_count(battle, CounterKind::Defense), 3);
        assert_eq!(g.object_view(battle).damage, 0);
    }
}

#[test]
fn battle_creature_damage_applies_both_and_defense_is_owned() {
    let mut g = scenario(Player::P0);
    let battle = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let mut recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
    recipe.overrides = Some(CopyOverrides {
        kinds: Some(1 | crate::cards::BATTLE_KIND),
        ..CopyOverrides::NONE
    });
    g.apply_temporary_copy(g.object_ref(battle), recipe, CopyDeadline::EndOfTurn);
    g.add_counter(battle, CounterKind::Defense, 5);
    let source = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let before = g.snapshot();
    let event = DamageEvent {
        source: g.damage_source(source),
        target: Target::Card(battle),
        amount: 1,
        combat: false,
        preventable: true,
    };
    let mut branch = g.clone();
    branch.apply_damage_batch(&[event]);
    assert_eq!(branch.counter_count(battle, CounterKind::Defense), 4);
    assert_eq!(branch.object_view(battle).damage, 1);
    assert_eq!(g.counter_count(battle, CounterKind::Defense), 5);
    g.apply_damage_batch(&[event]);
    assert_eq!(g.snapshot(), branch.snapshot());
    g.restore(&before).unwrap();
    assert_eq!(g.counter_count(battle, CounterKind::Defense), 5);
    assert_eq!(g.object_view(battle).damage, 0);
    let encoded = serde_json::to_value(Counters::of(CounterKind::Defense, 5)).unwrap();
    assert_eq!(encoded["named"], serde_json::json!([["Defense", 5]]));
    assert_eq!(
        serde_json::from_value::<Counters>(encoded)
            .unwrap()
            .get(CounterKind::Defense),
        5
    );
}

#[test]
fn attack_battle_uses_protector_not_controller_and_combat_removes_defense() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let battle = real_siege(&mut g, p);
        let reference = g.object_ref(battle);
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        big_helpers::checked(
            &mut g,
            Action::AttackBattle {
                card: attacker,
                defender: reference,
            },
        );
        big_helpers::checked(&mut g, Action::FinishAttackers);
        for _ in 0..40 {
            if matches!(g.state.pending, Pending::Blockers) {
                break;
            }
            big_helpers::checked(&mut g, Action::Pass);
        }
        assert!(matches!(g.state.pending, Pending::Blockers));
        big_helpers::checked(&mut g, Action::FinishBlockers);
        for _ in 0..40 {
            if g.counter_count(battle, CounterKind::Defense) == 3 {
                break;
            }
            big_helpers::checked(&mut g, Action::Pass);
        }
        assert_eq!(g.counter_count(battle, CounterKind::Defense), 3);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        let saved = g.snapshot();
        let mut branch = g.clone();
        g.move_object(battle, Zone::Graveyard);
        assert_eq!(g.battle_protector(reference), None);
        branch.restore(&saved).unwrap();
        assert_eq!(branch.battle_protector(reference), Some(p.other()));
    }
}

fn real_siege(g: &mut Game, p: Player) -> CardId {
    let siege = put(g, p, "@fixture/siege-front", Zone::Hand);
    big_helpers::cast(g, siege, None, &[]);
    big_helpers::settle(g);
    assert_eq!(g.counter_count(siege, CounterKind::Defense), 5);
    assert_eq!(g.battle_protector(g.object_ref(siege)), Some(p.other()));
    assert_eq!(g.object_view(siege).battle_protector, Some(p.other()));
    siege
}

#[test]
fn siege_last_counter_queues_defeat_then_offers_only_transformed_cast() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let siege = real_siege(&mut g, p);
        let old = g.object_ref(siege);
        g.pay_counters(siege, &Counters::of(CounterKind::Defense, 5));
        assert_eq!(g.state.pending_triggers.len(), 1);
        g.state_based_actions();
        assert_eq!(g.object_view(siege).zone, Zone::Battlefield);
        g.prepare_triggers();
        for _ in 0..40 {
            if matches!(g.state.pending, Pending::ResolutionCasts { .. }) {
                break;
            }
            let actions = g.legal_actions();
            let action = actions
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                .unwrap();
            big_helpers::checked(&mut g, action);
        }
        assert!(matches!(g.state.pending, Pending::ResolutionCasts { .. }));
        assert_eq!(g.object_view(siege).zone, Zone::Exile);
        let exile = g.object_ref(siege);
        assert_ne!(old, exile);
        let back = Action::CastExiledDuringResolution {
            object: exile,
            face: CastFace::Back,
        };
        assert!(g.legal_actions().contains(&back));
        assert!(
            !g.legal_actions()
                .contains(&Action::CastExiledDuringResolution {
                    object: exile,
                    face: CastFace::Normal
                })
        );
        big_helpers::checked(&mut g, back);
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        big_helpers::checked(&mut g, pay);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(siege).zone, Zone::Battlefield);
        assert_eq!(g.object_view(siege).card.name(), "@fixture/siege-back");
        assert_eq!(g.stats(siege).power, 3);
    }
}

#[test]
fn siege_zero_defense_without_pending_trigger_goes_to_graveyard() {
    let mut g = scenario(Player::P0);
    let siege = real_siege(&mut g, Player::P0);
    g.pay_counters(siege, &Counters::of(CounterKind::Defense, 5));
    g.state.pending_triggers.clear(); // Equivalent to the defeat trigger having left the stack.
    g.state_based_actions();
    assert_eq!(g.object_view(siege).zone, Zone::Graveyard);
}

#[test]
fn siege_control_change_reassigns_invalid_protector_as_state_based_action() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let siege = real_siege(&mut g, p);
        let object = g.object_ref(siege);
        g.change_controller(object, p.other());
        assert_eq!(g.battle_protector(object), Some(p.other()));
        g.state_based_actions();
        assert_eq!(g.battle_protector(object), Some(p));
        assert_eq!(g.object_view(siege).battle_protector, Some(p));
    }
}

#[test]
fn attacked_pure_battle_that_becomes_planeswalker_is_no_longer_attacked() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let siege = real_siege(&mut g, p);
        let reference = g.object_ref(siege);
        g.add_counter(siege, CounterKind::Loyalty, 5);
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        big_helpers::checked(
            &mut g,
            Action::AttackBattle {
                card: attacker,
                defender: reference,
            },
        );
        big_helpers::checked(&mut g, Action::FinishAttackers);
        let mut recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
        recipe.overrides = Some(CopyOverrides {
            kinds: Some(64),
            ..CopyOverrides::NONE
        });
        g.apply_temporary_copy(reference, recipe, CopyDeadline::EndOfTurn);
        g.state_based_actions();
        assert_eq!(g.state.attackers[0].defender, None);
        assert_eq!(
            g.combat_defender_target(g.state.attackers[0].defender),
            None
        );
    }
}

#[test]
fn siege_entry_freezes_protector_before_replacement_processing() {
    let mut g = scenario(Player::P0);
    let siege = put(&mut g, Player::P0, "@fixture/siege-front", Zone::Hand);
    let mut movement = g.entry_move(siege, Zone::Battlefield, false);
    movement.controller = Player::P1;
    movement.choose_battle_protector();
    assert_eq!(movement.battle_protector, Some(Player::P0));
    assert_eq!(movement.before.battle_protector, None);
    g.stage_entries(vec![movement], None);
    assert_eq!(g.battle_protector(g.object_ref(siege)), Some(Player::P0));
}

#[test]
fn siege_entry_replacement_reads_chosen_protector() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(
            &mut g,
            p,
            "@fixture/battle-reserved-5433",
            Zone::Battlefield,
        );
        let siege = put(&mut g, p, "@fixture/siege-front", Zone::Hand);
        big_helpers::cast(&mut g, siege, None, &[]);
        big_helpers::settle(&mut g);
        assert_eq!(g.battle_protector(g.object_ref(siege)), Some(p.other()));
        assert_eq!(g.counter_count(siege, CounterKind::Defense), 10);
    }
}

#[test]
fn greatwurm_cast_removes_last_defense_and_defeat_waits_until_payment_finishes() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let siege = real_siege(&mut g, p);
        let object = g.object_ref(siege);
        let mut recipe = CopyRecipe::new(Card::from_name("@fixture/siege-front").unwrap());
        recipe.overrides = Some(CopyOverrides {
            kinds: Some(1 | crate::cards::BATTLE_KIND),
            ..CopyOverrides::NONE
        });
        g.apply_temporary_copy(object, recipe, CopyDeadline::EndOfTurn);
        // The artificial type override supplies no creature base stats.
        g.state.objects[siege as usize].growth = Stats {
            power: 2,
            toughness: 2,
        };
        g.add_counter(siege, CounterKind::Defense, 1);
        let wurm = put(&mut g, p, "Quilled Greatwurm", Zone::Graveyard);
        big_helpers::checked(&mut g, Action::AnnounceCounterGraveyard { card: wurm });
        for _ in 0..6 {
            big_helpers::checked(
                &mut g,
                Action::ChooseCreatureCounterCost {
                    object,
                    counter: CounterKind::Defense,
                },
            );
        }
        assert_eq!(g.counter_count(siege, CounterKind::Defense), 6);
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        big_helpers::checked(&mut g, pay);
        assert_eq!(g.counter_count(siege, CounterKind::Defense), 0);
        assert_eq!(g.object_view(siege).zone, Zone::Battlefield);
        assert!(
            g.state
                .pending_triggers
                .iter()
                .chain(&g.state.ordering_triggers)
                .chain(&g.state.stack)
                .any(|item| item.kind == StackKind::SiegeDefeat
                    && item.source.unwrap().object == object)
        );
    }
}

#[test]
fn equipment_that_becomes_battle_detaches_as_state_based_action() {
    let mut g = scenario(Player::P0);
    let creature = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let equipment = put(&mut g, Player::P0, "Goldvein Pick", Zone::Battlefield);
    let object = g.object_ref(equipment);
    g.state.objects[equipment as usize].attached_to = Some(g.object_ref(creature));
    let mut recipe = CopyRecipe::new(Card::from_name("Goldvein Pick").unwrap());
    recipe.overrides = Some(CopyOverrides {
        kinds: Some(16 | crate::cards::BATTLE_KIND),
        ..CopyOverrides::NONE
    });
    g.apply_temporary_copy(object, recipe, CopyDeadline::EndOfTurn);
    g.add_counter(equipment, CounterKind::Defense, 5);
    g.state_based_actions();
    assert_eq!(g.object_view(equipment).attached_to, None);
    assert_eq!(g.object_view(equipment).zone, Zone::Battlefield);
}

#[test]
fn attacking_or_blocking_creature_that_becomes_battle_leaves_combat() {
    for blocking in [false, true] {
        let mut g = scenario(Player::P0);
        let attacker = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        let blocker = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        big_helpers::checked(&mut g, Action::Attack { card: attacker });
        big_helpers::checked(&mut g, Action::FinishAttackers);
        if blocking {
            for _ in 0..40 {
                if matches!(g.state.pending, Pending::Blockers) {
                    break;
                }
                big_helpers::checked(&mut g, Action::Pass);
            }
            big_helpers::checked(&mut g, Action::Block { blocker, attacker });
            big_helpers::checked(&mut g, Action::FinishBlockers);
        }
        let subject = if blocking { blocker } else { attacker };
        g.state_based_actions();
        let object = g.object_ref(subject);
        g.add_counter(subject, CounterKind::Defense, 5);
        let mut recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
        recipe.overrides = Some(CopyOverrides {
            kinds: Some(1 | crate::cards::BATTLE_KIND),
            ..CopyOverrides::NONE
        });
        g.apply_temporary_copy(object, recipe, CopyDeadline::EndOfTurn);
        g.state_based_actions();
        assert!(!g.state.attackers.iter().any(|a| a.card == subject));
        assert!(!g.state.blocking.contains(&subject));
    }
}

#[test]
fn creature_that_becomes_battle_then_reverts_during_one_spell_leaves_combat() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let battle = real_siege(&mut g, p);
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        big_helpers::checked(&mut g, Action::Attack { card: attacker });
        big_helpers::checked(&mut g, Action::FinishAttackers);
        let spell = put(&mut g, p, "@fixture/battle-reserved-5434", Zone::Hand);
        big_helpers::cast(
            &mut g,
            spell,
            None,
            &[
                Target::Card(attacker),
                Target::Card(battle),
                Target::Card(creature),
            ],
        );
        big_helpers::settle(&mut g);
        assert!(g.is_creature(attacker));
        assert_eq!(g.current_kinds(attacker) & crate::cards::BATTLE_KIND, 0);
        assert!(!g.state.attackers.iter().any(|a| a.card == attacker));
    }
}

#[test]
fn battle_ordinary_combat_timing() {
    let mut g = scenario(Player::P0);
    let attacker = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    big_helpers::checked(&mut g, Action::Attack { card: attacker });
    big_helpers::checked(&mut g, Action::FinishAttackers);
    let start = std::time::Instant::now();
    for _ in 0..10_000 {
        g.state_based_actions();
    }
    let elapsed = start.elapsed();
    println!(
        "ordinary-combat SBA: 10000 checks in {:.6}s ({:.0} checks/s)",
        elapsed.as_secs_f64(),
        10_000.0 / elapsed.as_secs_f64()
    );
    assert_eq!(g.state.attackers.len(), 1);
}

#[test]
fn battle_entering_as_nonbattle_artifact_has_no_protector() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let siege = put(&mut g, p, "@fixture/siege-front", Zone::Graveyard);
        let mut movement = g.entry_move(siege, Zone::Battlefield, false);
        movement.artifact_return = true;
        let projected = g.preview_battle_entry(&movement).unwrap();
        assert_eq!(projected.kinds & crate::cards::BATTLE_KIND, 0);
        g.stage_entries(vec![movement], None);
        assert_eq!(g.current_kinds(siege) & crate::cards::BATTLE_KIND, 0);
        assert_eq!(g.battle_protector(g.object_ref(siege)), None);
        assert_eq!(g.counter_count(siege, CounterKind::Defense), 0);
    }
}

#[test]
fn invalid_siege_protector_is_reassigned_even_if_it_is_attacked() {
    let mut g = scenario(Player::P0);
    let siege = real_siege(&mut g, Player::P0);
    let object = g.object_ref(siege);
    let attacker = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    big_helpers::checked(
        &mut g,
        Action::AttackBattle {
            card: attacker,
            defender: object,
        },
    );
    big_helpers::checked(&mut g, Action::FinishAttackers);
    // Isolate the SBA by changing control without the separate remove-from-combat event.
    g.state.objects[siege as usize].controller = Player::P1;
    g.state_based_actions();
    assert_eq!(g.battle_protector(object), Some(Player::P0));
    assert!(
        g.state
            .attackers
            .iter()
            .all(|a| a.defender != Some(CombatDefender::Battle(object)))
    );
}

#[test]
fn non_siege_with_zero_defense_dies_even_when_its_siege_trigger_is_pending() {
    let mut g = scenario(Player::P0);
    let siege = real_siege(&mut g, Player::P0);
    let object = g.object_ref(siege);
    g.pay_counters(siege, &Counters::of(CounterKind::Defense, 5));
    assert!(
        g.state
            .pending_triggers
            .iter()
            .any(|item| item.kind == StackKind::SiegeDefeat)
    );
    let mut recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
    recipe.overrides = Some(CopyOverrides {
        kinds: Some(crate::cards::BATTLE_KIND),
        ..CopyOverrides::NONE
    });
    g.apply_temporary_copy(object, recipe, CopyDeadline::EndOfTurn);
    g.state_based_actions();
    assert_eq!(g.object_view(siege).zone, Zone::Graveyard);
}

#[test]
fn creature_battle_cannot_be_declared_attacking_or_blocking() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let blocker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        for id in [attacker, blocker] {
            let mut recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
            recipe.overrides = Some(CopyOverrides {
                kinds: Some(1 | crate::cards::BATTLE_KIND),
                ..CopyOverrides::NONE
            });
            g.apply_temporary_copy(g.object_ref(id), recipe, CopyDeadline::EndOfTurn);
            g.add_counter(id, CounterKind::Defense, 5);
        }
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        reject(&mut g, Action::Attack { card: attacker });
        let ordinary = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        big_helpers::checked(&mut g, Action::Attack { card: ordinary });
        big_helpers::checked(&mut g, Action::FinishAttackers);
        for _ in 0..40 {
            if matches!(g.state.pending, Pending::Blockers) {
                break;
            }
            big_helpers::checked(&mut g, Action::Pass);
        }
        reject(
            &mut g,
            Action::Block {
                blocker,
                attacker: ordinary,
            },
        );
    }
}

#[test]
fn attacked_dual_type_retains_defender_when_allowed_by_506_4e() {
    for remaining in [64, crate::cards::BATTLE_KIND] {
        let mut g = scenario(Player::P0);
        let defender = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        let object = g.object_ref(defender);
        let mut recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
        recipe.overrides = Some(CopyOverrides {
            kinds: Some(64 | crate::cards::BATTLE_KIND),
            ..CopyOverrides::NONE
        });
        g.apply_temporary_copy(object, recipe, CopyDeadline::EndOfTurn);
        g.add_counter(defender, CounterKind::Defense, 5);
        g.add_counter(defender, CounterKind::Loyalty, 5);
        g.set_battle_protector(object, Player::P1);
        let attacker = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        big_helpers::checked(
            &mut g,
            Action::AttackBattle {
                card: attacker,
                defender: object,
            },
        );
        big_helpers::checked(&mut g, Action::FinishAttackers);
        let mut recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
        recipe.overrides = Some(CopyOverrides {
            kinds: Some(remaining),
            ..CopyOverrides::NONE
        });
        g.apply_temporary_copy(object, recipe, CopyDeadline::EndOfTurn);
        g.state_based_actions();
        assert_eq!(
            g.state.attackers[0].defender,
            Some(CombatDefender::Battle(object))
        );
        assert_eq!(
            g.combat_defender_target(g.state.attackers[0].defender),
            Some(Target::Card(defender))
        );
    }
}

#[test]
fn attacked_dual_type_losing_battle_leaves_combat_when_protector_differs_from_controller() {
    let mut g = scenario(Player::P0);
    let defender = real_siege(&mut g, Player::P0);
    let object = g.object_ref(defender);
    let mut recipe = CopyRecipe::new(Card::from_name("@fixture/siege-front").unwrap());
    recipe.overrides = Some(CopyOverrides {
        kinds: Some(64 | crate::cards::BATTLE_KIND),
        ..CopyOverrides::NONE
    });
    g.apply_temporary_copy(object, recipe, CopyDeadline::EndOfTurn);
    g.add_counter(defender, CounterKind::Loyalty, 5);
    let attacker = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    big_helpers::checked(
        &mut g,
        Action::AttackBattle {
            card: attacker,
            defender: object,
        },
    );
    big_helpers::checked(&mut g, Action::FinishAttackers);
    let mut recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
    recipe.overrides = Some(CopyOverrides {
        kinds: Some(64),
        ..CopyOverrides::NONE
    });
    g.apply_temporary_copy(object, recipe, CopyDeadline::EndOfTurn);
    g.state_based_actions();
    assert_eq!(g.state.attackers[0].defender, None);
}

#[test]
fn mandatory_attacker_can_attack_planeswalker_when_player_is_taxed() {
    let p = Player::P0;
    let mut g = scenario(p);
    let attacker = put(&mut g, p, "Juggernaut", Zone::Battlefield);
    let dain = put(
        &mut g,
        p.other(),
        "Dáin, Lord of the Iron Hills",
        Zone::Battlefield,
    );
    g.state.players[p.other().index()].enduring_story = true;
    assert_eq!(g.attack_controller_tax(p.other()), 1);
    let walker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let mut recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
    recipe.overrides = Some(CopyOverrides {
        kinds: Some(64),
        ..CopyOverrides::NONE
    });
    g.apply_temporary_copy(g.object_ref(walker), recipe, CopyDeadline::EndOfTurn);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    assert!(g.can_declare_attacker(p, attacker));
    assert!(!g.attack_declaration_complete(p));
    let _ = dain;
}
