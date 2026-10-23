//! Paid Sneak, exact return reservations and inherited attack destinations.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;

fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert!(g.apply(g.actor().unwrap(), action).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn attackers(g: &mut Game, source: CardId, defender: Option<ObjectRef>) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    checked(
        g,
        defender.map_or(Action::Attack { card: source }, |defender| {
            Action::AttackPlaneswalker {
                card: source,
                defender,
            }
        }),
    );
    checked(g, Action::FinishAttackers);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    assert_eq!(g.state.pending, Pending::Blockers);
    for player in &mut g.state.players {
        player.mana = [30; 6];
    }
}
fn unblocked(g: &mut Game, source: CardId) {
    attackers(g, source, None);
    checked(g, Action::FinishBlockers);
    assert_eq!(g.state.step, Step::Blockers);
}
fn sneak(g: &mut Game, source: CardId, attacker: CardId, cost: Option<u16>) {
    let witness = g.object_ref(attacker);
    checked(
        g,
        Action::AnnounceSneak {
            card: source,
            attacker: witness,
        },
    );
    if let Some(index) = cost {
        checked(g, Action::ChooseSpellCost { cost: index });
    }
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .expect("Sneak payment");
        checked(g, action);
    }
    panic!("Sneak payment stalled");
}
#[test]
fn tmt_sneak_only_own_post_declaration_blockers_step_with_live_unblocked_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Donatello's Technique", Zone::Hand);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let action = Action::AnnounceSneak {
            card: source,
            attacker: g.object_ref(bear),
        };
        reject(&mut g, action.clone());
        attackers(&mut g, bear, None);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::AnnounceSneak { .. }))
        );
        checked(&mut g, Action::FinishBlockers);
        assert!(g.legal_actions().contains(&action));
        let enemy_ref = g.object_ref(enemy);
        reject(
            &mut g,
            Action::AnnounceSneak {
                card: source,
                attacker: enemy_ref,
            },
        );
        let old = g.object_ref(bear);
        reject(
            &mut g,
            Action::AnnounceSneak {
                card: source,
                attacker: ObjectRef {
                    generation: old.generation + 1,
                    ..old
                },
            },
        );
        g.state.active = p.other();
        reject(&mut g, action.clone());
        g.state.active = p;
        for step in [
            Step::Main1,
            Step::Attackers,
            Step::FirstStrikeDamage,
            Step::CombatDamage,
            Step::EndCombat,
            Step::Main2,
            Step::End,
        ] {
            g.state.step = step;
            reject(&mut g, action.clone());
        }
    }
}
#[test]
fn tmt_sneak_blocked_attacker_remains_ineligible_after_its_blocker_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Foot Ninjas", Zone::Hand);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let blocker = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        attackers(&mut g, bear, None);
        checked(
            &mut g,
            Action::Block {
                blocker,
                attacker: bear,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        g.move_object(blocker, Zone::Exile);
        let bear_ref = g.object_ref(bear);
        reject(
            &mut g,
            Action::AnnounceSneak {
                card: source,
                attacker: bear_ref,
            },
        );
    }
}
#[test]
fn tmt_sneak_sorcery_pays_only_alternative_and_return_cost_before_any_response() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p, "Donatello's Technique", Zone::Hand);
        unblocked(&mut g, bear);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        let hand = g.observe(p).players[p.index()].hand_count;
        let witness = g.object_ref(bear);
        checked(
            &mut g,
            Action::AnnounceSneak {
                card: source,
                attacker: witness,
            },
        );
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Battlefield);
        assert!(g.observe(p.other()).additional_cost.is_none());
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 1, 0, 0, 0, 0],
            },
        );
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Hand);
        assert_eq!(g.actor(), Some(p));
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().method,
            CastMethod::Sneak
        );
        assert_eq!(
            g.state
                .stack
                .last()
                .unwrap()
                .sneak
                .as_ref()
                .unwrap()
                .attacker,
            witness
        );
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.observe(p).players[p.index()].permanent_left_battlefield_this_turn);
        settle(&mut g);
        assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
        assert_eq!(g.observe(p).players[p.index()].hand_count, hand + 2);
        assert!(g.state.attackers.is_empty());
    }
}
#[test]
fn tmt_sneak_hybrid_creature_enters_tapped_attacking_and_resolves_etb() {
    for p in [Player::P0, Player::P1] {
        for index in 0..2 {
            let mut g = setup(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let source = named(&mut g, p, "Foot Ninjas", Zone::Hand);
            unblocked(&mut g, bear);
            g.state.players[p.index()].mana = if index == 0 {
                [1, 0, 0, 0, 0, 3]
            } else {
                [0, 0, 1, 0, 0, 3]
            };
            let life = g.state.players[p.index()].life;
            sneak(&mut g, source, bear, Some(index));
            settle(&mut g);
            let view = g.object_view(source);
            assert!(view.tapped && view.attacking && view.summoning_sick);
            assert_eq!((view.power, view.toughness), (5, 5));
            assert_eq!(g.state.players[p.index()].life, life + 3);
            assert_eq!(g.state.attackers.len(), 1);
            assert_eq!(
                g.state.attackers[0].defender,
                Some(CombatDefender::Player(p.other()))
            );
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn tmt_sneak_preserves_exact_planeswalker_defender_and_does_not_retarget_after_departure() {
    for p in [Player::P0, Player::P1] {
        for remove in [false, true] {
            let mut g = setup(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let source = named(&mut g, p, "Foot Ninjas", Zone::Hand);
            let walker = named(&mut g, p.other(), "Kaito, Cunning Infiltrator", Zone::Hand);
            g.move_object(walker, Zone::Battlefield);
            let defender = g.object_ref(walker);
            attackers(&mut g, bear, Some(defender));
            checked(&mut g, Action::FinishBlockers);
            sneak(&mut g, source, bear, Some(0));
            if remove {
                g.move_object(walker, Zone::Exile);
                g.move_object(walker, Zone::Battlefield);
            }
            settle(&mut g);
            assert!(g.object_view(source).tapped);
            assert_eq!(g.object_view(source).attacking, !remove);
            if !remove {
                assert_eq!(
                    g.state.attackers[0].defender,
                    Some(CombatDefender::Planeswalker(defender))
                );
            }
        }
    }
}
#[test]
fn tmt_sneak_normal_costs_remain_available_without_return_or_attacking_entry() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "Donatello's Technique",
            "Foot Ninjas",
            "Leonardo, Big Brother",
            "Oroku Saki, Shredder Rising",
        ] {
            let mut g = setup(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let source = named(&mut g, p, name, Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: source });
            if g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseSpellCost { .. }))
            {
                checked(&mut g, Action::ChooseSpellCost { cost: 0 });
            }
            settle(&mut g);
            assert_eq!(g.state.objects[bear as usize].zone, Zone::Battlefield);
            if name != "Donatello's Technique" {
                assert!(!g.object_view(source).tapped && !g.object_view(source).attacking);
            }
            assert!(!g.observe(p).players[p.index()].permanent_left_battlefield_this_turn);
        }
    }
}
#[test]
fn tmt_sneak_oroku_actual_combat_draws_and_loses_life_without_attack_declaration_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p, "Oroku Saki, Shredder Rising", Zone::Hand);
        unblocked(&mut g, bear);
        sneak(&mut g, source, bear, None);
        settle(&mut g);
        let life = g.state.players[p.index()].life;
        let enemy = g.state.players[p.other().index()].life;
        let hand = g.observe(p).players[p.index()].hand_count;
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, enemy - 3);
        assert_eq!(g.state.players[p.index()].life, life - 1);
        assert_eq!(g.observe(p).players[p.index()].hand_count, hand + 1);
    }
}
#[test]
fn tmt_sneak_leonardo_scales_after_return_and_with_later_creature_changes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let other = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p, "Leonardo, Big Brother", Zone::Hand);
        unblocked(&mut g, bear);
        sneak(&mut g, source, bear, None);
        settle(&mut g);
        assert_eq!((g.stats(source).power, g.stats(source).toughness), (2, 3));
        g.move_object(other, Zone::Hand);
        assert_eq!(g.stats(source).power, 1);
        g.move_object(other, Zone::Battlefield);
        assert_eq!(g.stats(source).power, 2);
    }
}
#[test]
fn tmt_sneak_countered_spell_does_not_refund_return_and_creates_no_attacker() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p, "Foot Ninjas", Zone::Hand);
        let counter = named(&mut g, p.other(), "Essence Scatter", Zone::Hand);
        unblocked(&mut g, bear);
        sneak(&mut g, source, bear, Some(0));
        checked(&mut g, Action::Pass);
        cast(&mut g, counter, None, &[Target::Spell(source)]);
        settle(&mut g);
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Hand);
        assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
        assert!(g.state.attackers.is_empty());
        assert_eq!(g.state.players[p.index()].life, 20);
    }
}
#[test]
fn tmt_sneak_return_capacity_failure_rolls_back_mana_history_and_entire_action() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[bear as usize].generation = u64::MAX;
        let source = named(&mut g, p, "Donatello's Technique", Zone::Hand);
        unblocked(&mut g, bear);
        let witness = g.object_ref(bear);
        checked(
            &mut g,
            Action::AnnounceSneak {
                card: source,
                attacker: witness,
            },
        );
        let snapshot = g.snapshot();
        let revision = g.revision();
        for _ in 0..2 {
            assert_eq!(
                g.apply(
                    p,
                    Action::PayCosts {
                        payment: [0, 1, 0, 0, 0, 0]
                    }
                ),
                Err(Error::CapacityExceeded(crate::CapacityExceeded {
                    resource: "object generation exhausted"
                }))
            );
            assert_eq!(g.snapshot(), snapshot);
            assert_eq!(g.revision(), revision);
        }
        assert!(!g.observe(p).players[p.index()].permanent_left_battlefield_this_turn);
    }
}

#[test]
fn tmt_sneak_returned_foreign_creature_goes_to_owner_and_tokens_still_pay_the_cost() {
    for p in [Player::P0, Player::P1] {
        for token in [false, true] {
            let mut g = setup(p);
            let bear = named(
                &mut g,
                if token { p } else { p.other() },
                if token {
                    "@token/ninja_1_1_black"
                } else {
                    "Bear Cub"
                },
                Zone::Battlefield,
            );
            // Established prior-turn control is a scenario premise, independent of ownership.
            g.state.objects[bear as usize].controller = p;
            let source = named(&mut g, p, "Donatello's Technique", Zone::Hand);
            unblocked(&mut g, bear);
            sneak(&mut g, source, bear, None);
            settle(&mut g);
            if token {
                assert!(g.state.free_tokens.contains(&bear));
            } else {
                assert_eq!(g.state.objects[bear as usize].zone, Zone::Hand);
                assert_eq!(g.state.objects[bear as usize].owner, p.other());
            }
            assert!(g.observe(p).players[p.index()].permanent_left_battlefield_this_turn);
            assert!(
                !g.observe(p.other()).players[p.other().index()]
                    .permanent_left_battlefield_this_turn
            );
        }
    }
}
#[test]
fn tmt_sneak_reserved_attacker_can_tap_for_mana_before_return_without_losing_reservation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let grant_source = g.object_ref(elf);
        g.grant_keyword_effects(&[elf], grant_source, Keywords::VIGILANCE);
        let source = named(&mut g, p, "Foot Ninjas", Zone::Hand);
        unblocked(&mut g, elf);
        assert!(!g.object_view(elf).tapped);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 2];
        let witness = g.object_ref(elf);
        checked(
            &mut g,
            Action::AnnounceSneak {
                card: source,
                attacker: witness,
            },
        );
        checked(&mut g, Action::ChooseSpellCost { cost: 0 });
        assert!(g.mana_payment_domain().is_none());
        checked(&mut g, Action::TapMana { card: elf });
        assert_eq!(g.state.objects[elf as usize].zone, Zone::Battlefield);
        checked(
            &mut g,
            Action::PayCosts {
                payment: [1, 0, 0, 0, 1, 2],
            },
        );
        assert_eq!(g.state.objects[elf as usize].zone, Zone::Hand);
        settle(&mut g);
        assert!(g.object_view(source).attacking);
    }
}
#[test]
fn tmt_sneak_jennika_sweeps_both_sides_but_not_the_returned_cost_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let attacker = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let giant = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let source = named(&mut g, p, "Jennika's Technique", Zone::Hand);
        unblocked(&mut g, attacker);
        sneak(&mut g, source, attacker, None);
        settle(&mut g);
        assert_eq!(g.state.objects[attacker as usize].zone, Zone::Hand);
        assert_eq!(g.state.objects[own as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[enemy as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[giant as usize].zone, Zone::Battlefield);
        assert_eq!(g.object_view(giant).damage, 2);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
fn search_window(g: &mut Game) {
    for _ in 0..20 {
        if matches!(g.state.pending, Pending::EffectSearch { .. }) {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("no search window");
}
#[test]
fn tmt_sneak_splinter_private_tutor_requires_one_card_and_does_not_reveal_identity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let attacker = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let desired = named(&mut g, p, "Oroku Saki, Shredder Rising", Zone::Library);
        let source = named(&mut g, p, "Splinter's Technique", Zone::Hand);
        unblocked(&mut g, attacker);
        sneak(&mut g, source, attacker, None);
        search_window(&mut g);
        assert!(g.observe(p.other()).search_choice.is_none());
        assert!(g.observe(p.other()).actions.is_empty());
        reject(&mut g, Action::FinishSearch);
        reject(&mut g, Action::ChooseSearch { card: attacker });
        checked(&mut g, Action::ChooseSearch { card: desired });
        if g.legal_actions().contains(&Action::FinishSearch) {
            checked(&mut g, Action::FinishSearch);
        }
        settle(&mut g);
        assert_eq!(g.state.objects[desired as usize].zone, Zone::Hand);
        assert!(!g.state.known_hand.contains(&g.object_ref(desired)));
    }
}
#[test]
fn tmt_sneak_new_generation_selects_up_to_two_basic_lands_and_enters_them_tapped() {
    for p in [Player::P0, Player::P1] {
        for amount in 0..=2 {
            let mut g = setup(p);
            let attacker = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let first = named(&mut g, p, "Forest", Zone::Library);
            let second = named(&mut g, p, "Swamp", Zone::Library);
            let wrong = named(&mut g, p, "Mutant Town", Zone::Library);
            let source = named(&mut g, p, "New Generation's Technique", Zone::Hand);
            unblocked(&mut g, attacker);
            sneak(&mut g, source, attacker, None);
            search_window(&mut g);
            reject(&mut g, Action::ChooseSearch { card: wrong });
            for &card in [first, second].iter().take(amount) {
                checked(&mut g, Action::ChooseSearch { card });
            }
            if g.legal_actions().contains(&Action::FinishSearch) {
                checked(&mut g, Action::FinishSearch);
            }
            settle(&mut g);
            for (i, card) in [first, second].into_iter().enumerate() {
                assert_eq!(
                    g.state.objects[card as usize].zone,
                    if i < amount {
                        Zone::Battlefield
                    } else {
                        Zone::Library
                    }
                );
                if i < amount {
                    assert!(g.object_view(card).tapped);
                }
            }
        }
    }
}
#[test]
fn tmt_sneak_leonardo_reanimation_rejects_wrong_owner_kind_value_and_keeps_partial_targets() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let attacker = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let first = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let second = named(&mut g, p, "Oroku Saki, Shredder Rising", Zone::Graveyard);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
        let costly = named(&mut g, p, "Hill Giant", Zone::Graveyard);
        let artifact = named(&mut g, p, "Mouser Foundry", Zone::Graveyard);
        let source = named(&mut g, p, "Leonardo's Technique", Zone::Hand);
        unblocked(&mut g, attacker);
        let witness = g.object_ref(attacker);
        checked(
            &mut g,
            Action::AnnounceSneak {
                card: source,
                attacker: witness,
            },
        );
        reject(&mut g, Action::FinishTargets);
        for card in [enemy, costly, artifact, attacker] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(card),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(first),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(second),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        g.move_object(first, Zone::Exile);
        g.move_object(first, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.objects[first as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[second as usize].zone, Zone::Battlefield);
        assert!(!g.object_view(second).attacking && !g.object_view(second).tapped);
    }
}

fn settle_target(g: &mut Game, target: CardId) {
    g.prepare_triggers();
    for _ in 0..80 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let chosen = Action::ChooseTarget {
            target: Target::Card(target),
        };
        let action = if actions.contains(&chosen) {
            chosen
        } else if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::PutTrigger { .. } | Action::FinishTargets | Action::PayCosts { .. }
                    )
                })
                .unwrap()
        };
        checked(g, action);
    }
    panic!("targeted resolution stalled");
}
#[test]
fn tmt_sneak_cutting_edge_lifelink_and_food_each_add_one_counter_per_life_event() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let attacker = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p, "Leonardo, Cutting Edge", Zone::Hand);
        unblocked(&mut g, attacker);
        sneak(&mut g, source, attacker, None);
        settle(&mut g);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        let food = named(
            &mut g,
            p,
            "@token/283fd83a9aec83a51b736926",
            Zone::Battlefield,
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        let activation = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::Activate{card,..}if *card==food))
            .unwrap();
        checked(&mut g, activation);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 24);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 2);
        assert_eq!((g.stats(source).power, g.stats(source).toughness), (3, 3));
    }
}
#[test]
fn tmt_sneak_splinter_lord_excludes_self_foreign_ninjas_and_non_ninjas_and_has_real_menace() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ninja = named(&mut g, p, "@token/ninja_1_1_black", Zone::Battlefield);
        let foreign = named(
            &mut g,
            p.other(),
            "@token/ninja_1_1_black",
            Zone::Battlefield,
        );
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p, "Splinter, Hamato Yoshi", Zone::Hand);
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        assert_eq!((g.stats(source).power, g.stats(source).toughness), (1, 3));
        assert_eq!(g.stats(ninja).power, 2);
        assert_eq!(g.stats(foreign).power, 1);
        assert_eq!(g.stats(bear).power, 2);
        assert!(g.keywords(source).contains(Keywords::MENACE));
        let blocker1 = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let blocker2 = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[source as usize].summoning_sick = false;
        attackers(&mut g, source, None);
        checked(
            &mut g,
            Action::Block {
                blocker: blocker1,
                attacker: source,
            },
        );
        reject(&mut g, Action::FinishBlockers);
        checked(
            &mut g,
            Action::Block {
                blocker: blocker2,
                attacker: source,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        g.move_object(source, Zone::Exile);
        assert_eq!(g.stats(ninja).power, 1);
    }
}
#[test]
fn tmt_sneak_nightwatcher_grants_actual_double_strike_only_to_controlled_attackers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let returned = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let ally = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let resting = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p, "Raphael, the Nightwatcher", Zone::Hand);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: returned });
        checked(&mut g, Action::Attack { card: ally });
        checked(&mut g, Action::FinishAttackers);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::FinishBlockers);
        g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 1];
        sneak(&mut g, source, returned, None);
        settle(&mut g);
        for id in [source, ally] {
            assert!(g.keywords(id).contains(Keywords::DOUBLE_STRIKE));
        }
        for id in [resting, enemy] {
            assert!(!g.keywords(id).contains(Keywords::DOUBLE_STRIKE));
        }
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 12);
    }
}
#[test]
fn tmt_sneak_shredder_etb_and_attack_grants_are_other_controlled_targeted_and_expire() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p, "Shredder, Unrelenting", Zone::Hand);
        cast(&mut g, source, None, &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        g.prepare_triggers();
        for target in [source, enemy] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
        }
        settle_target(&mut g, bear);
        assert!(g.keywords(source).contains(Keywords::DEATHTOUCH));
        assert!(g.keywords(bear).contains(Keywords::DEATHTOUCH));
        g.state.objects[source as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: source });
        checked(&mut g, Action::FinishAttackers);
        settle_target(&mut g, bear);
        assert_eq!(
            g.observe(p)
                .keyword_effects
                .iter()
                .filter(|r| r.recipient.card == bear && r.keywords.contains(Keywords::DEATHTOUCH))
                .count(),
            2
        );
        g.move_object(source, Zone::Exile);
        assert!(g.keywords(bear).contains(Keywords::DEATHTOUCH));
        super::ability_timeline_tests::next_turn(&mut g);
        assert!(!g.keywords(bear).contains(Keywords::DEATHTOUCH));
    }
}
#[test]
fn tmt_sneak_karai_one_or_both_modes_allow_shared_targets_and_expire_normally() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            for shared in [false, true] {
                let mut g = setup(p);
                let returned = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let giant = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
                let source = named(&mut g, p, "Karai's Technique", Zone::Hand);
                unblocked(&mut g, returned);
                let witness = g.object_ref(returned);
                checked(
                    &mut g,
                    Action::AnnounceSneak {
                        card: source,
                        attacker: witness,
                    },
                );
                reject(&mut g, Action::ChooseMode { mode: 3 });
                checked(&mut g, Action::ChooseMode { mode });
                let first = if mode == 1 { giant } else { bear };
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(first),
                    },
                );
                if mode == 2 {
                    if g.legal_actions().contains(&Action::FinishTargets) {
                        checked(&mut g, Action::FinishTargets);
                    }
                    checked(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(if shared { bear } else { giant }),
                        },
                    );
                }
                if g.legal_actions().contains(&Action::FinishTargets) {
                    checked(&mut g, Action::FinishTargets);
                }
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. }))
                    .unwrap();
                checked(&mut g, action);
                settle(&mut g);
                assert_eq!(
                    g.stats(bear).power,
                    if mode == 0 || mode == 2 && !shared {
                        5
                    } else {
                        2
                    }
                );
                if mode == 1 || mode == 2 && !shared {
                    assert_eq!(g.state.objects[giant as usize].zone, Zone::Graveyard);
                }
                if mode == 0 {
                    super::ability_timeline_tests::next_turn(&mut g);
                    assert_eq!(g.stats(bear).power, 2);
                }
            }
        }
    }
}

#[test]
fn tmt_sneak_uses_generic_graveyard_permission_but_never_flashback_permission() {
    for p in [Player::P0, Player::P1] {
        for kind in [GraveyardCastKind::Cast, GraveyardCastKind::Flashback] {
            let mut g = setup(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let source = named(&mut g, p, "Donatello's Technique", Zone::Graveyard);
            unblocked(&mut g, bear);
            let action = Action::AnnounceSneak {
                card: source,
                attacker: g.object_ref(bear),
            };
            reject(&mut g, action.clone());
            let mut permission = g.activation_item(p, bear, 0);
            permission.targets.push(g.bind_target(Target::Card(source)));
            g.grant_graveyard_cast(&permission, kind);
            if kind == GraveyardCastKind::Flashback {
                reject(&mut g, action);
                continue;
            }
            sneak(&mut g, source, bear, None);
            assert_eq!(
                g.state.stack.last().unwrap().cast.unwrap().origin,
                Zone::Graveyard
            );
            settle(&mut g);
            assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
            assert!(!g.has_graveyard_cast(p, source, GraveyardCastKind::Cast));
        }
    }
}
#[test]
fn tmt_sneak_uses_paid_exile_permission_for_a_foreign_card_but_not_a_free_only_route() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p.other(), "Foot Ninjas", Zone::Exile);
        unblocked(&mut g, bear);
        let action = Action::AnnounceSneak {
            card: source,
            attacker: g.object_ref(bear),
        };
        reject(&mut g, action.clone());
        let reference = g.object_ref(source);
        g.grant_exile_free_cast_permission(p, reference);
        reject(&mut g, action);
        g.grant_exile_cast_permission(p, reference);
        sneak(&mut g, source, bear, Some(0));
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().origin,
            Zone::Exile
        );
        settle(&mut g);
        let view = g.object_view(source);
        assert_eq!(view.owner, p.other());
        assert_eq!(view.controller, p);
        assert!(view.tapped && view.attacking);
    }
}
#[test]
fn tmt_sneak_uses_library_top_cast_permission_and_keeps_the_next_top_private_during_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Vizier of the Menagerie", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p, "Foot Ninjas", Zone::Library);
        unblocked(&mut g, bear);
        let witness = g.object_ref(bear);
        checked(
            &mut g,
            Action::AnnounceSneak {
                card: source,
                attacker: witness,
            },
        );
        assert!(g.observe(p).known_library_top[p.index()].is_none());
        assert!(g.observe(p.other()).known_library_top[p.index()].is_none());
        checked(&mut g, Action::ChooseSpellCost { cost: 0 });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().origin,
            Zone::Library
        );
        settle(&mut g);
        assert!(g.object_view(source).tapped && g.object_view(source).attacking);
    }
}
