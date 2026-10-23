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
pub(super) fn unblocked(g: &mut Game, source: CardId) {
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

#[test]
fn tmt_sneak_leader_entry_boost_requires_paid_cost_and_freezes_recipients() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            let mut g = setup(p);
            let returned = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let ally = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let source = named(&mut g, p, "Leonardo, Leader in Blue", Zone::Hand);
            if paid {
                unblocked(&mut g, returned);
                sneak(&mut g, source, returned, None);
            } else {
                cast(&mut g, source, None, &[]);
            }
            settle(&mut g);
            assert_eq!(g.stats(source).power, if paid { 4 } else { 2 });
            assert_eq!(g.stats(ally).power, if paid { 4 } else { 2 });
            assert_eq!(g.stats(enemy).power, 2);
            assert_eq!(
                g.observe(p).sneak_entries,
                g.observe(p.other()).sneak_entries
            );
            assert_eq!(
                g.observe(p).sneak_entries.contains(&g.object_ref(source)),
                paid
            );
            let late = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            assert_eq!(g.stats(late).power, 2);
            g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
            let activation = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Activate{card,..} if *card==source))
                .unwrap();
            checked(&mut g, activation);
            settle(&mut g);
            assert!(g.keywords(source).contains(Keywords::FIRST_STRIKE));
            super::ability_timeline_tests::next_turn(&mut g);
            assert_eq!(g.stats(source).power, 2);
            assert_eq!(g.stats(ally).power, 2);
            assert!(!g.keywords(source).contains(Keywords::FIRST_STRIKE));
            assert!(g.observe(p).sneak_entries.is_empty());
        }
    }
}
#[test]
fn tmt_sneak_leader_intervening_guard_keeps_departed_incarnation_without_rebinding() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let returned = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let ally = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p, "Leonardo, Leader in Blue", Zone::Hand);
        unblocked(&mut g, returned);
        sneak(&mut g, source, returned, None);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        g.prepare_triggers();
        assert!(!g.state.stack.is_empty() || !g.state.pending_triggers.is_empty());
        let old = g.object_ref(source);
        g.move_object(source, Zone::Exile);
        g.move_object(source, Zone::Battlefield);
        assert_ne!(old, g.object_ref(source));
        settle(&mut g);
        assert_eq!(g.stats(ally).power, 4);
        assert_eq!(g.stats(source).power, 4); // new incarnation is a recipient, not a Sneak source
        assert!(!g.observe(p).sneak_entries.contains(&g.object_ref(source)));
    }
}
#[test]
fn tmt_sneak_kunoichi_normal_link_or_permanent_exile_and_source_departure() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            for depart in [false, true] {
                let mut g = setup(p);
                let returned = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                let source = named(&mut g, p, "Turncoat Kunoichi", Zone::Hand);
                if paid {
                    unblocked(&mut g, returned);
                    sneak(&mut g, source, returned, None);
                } else {
                    cast(&mut g, source, None, &[]);
                }
                checked(&mut g, Action::Pass);
                checked(&mut g, Action::Pass);
                g.prepare_triggers();
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(source),
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
                if g.legal_actions().contains(&Action::FinishTargets) {
                    checked(&mut g, Action::FinishTargets);
                }
                if depart {
                    g.move_object(source, Zone::Exile);
                }
                settle(&mut g);
                assert_eq!(
                    g.state.objects[enemy as usize].zone,
                    if !paid && depart {
                        Zone::Battlefield
                    } else {
                        Zone::Exile
                    }
                );
                if !depart {
                    g.move_object(source, Zone::Exile);
                    settle(&mut g);
                    assert_eq!(
                        g.state.objects[enemy as usize].zone,
                        if paid { Zone::Exile } else { Zone::Battlefield }
                    );
                }
            }
        }
    }
}
#[test]
fn tmt_sneak_karai_actual_combat_reanimates_only_during_paid_entry_turn() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            let mut g = setup(p);
            let returned = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let dead = named(&mut g, p, "Hill Giant", Zone::Graveyard);
            let source = named(&mut g, p, "Karai, Future of the Foot", Zone::Hand);
            if paid {
                unblocked(&mut g, returned);
                sneak(&mut g, source, returned, None);
                settle(&mut g);
            } else {
                cast(&mut g, source, None, &[]);
                settle(&mut g);
                g.state.objects[source as usize].summoning_sick = false;
                unblocked(&mut g, source);
            }
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            settle_target(&mut g, dead);
            assert_eq!(g.state.players[p.other().index()].life, 17);
            assert_eq!(
                g.state.objects[dead as usize].zone,
                if paid { Zone::Battlefield } else { Zone::Hand }
            );
            if paid {
                assert!(!g.object_view(dead).attacking && !g.object_view(dead).tapped);
            }
            super::ability_timeline_tests::next_turn(&mut g);
            assert!(g.observe(p).sneak_entries.is_empty());
            g.move_object(dead, Zone::Graveyard);
            g.state.active = p;
            g.state.priority = p;
            g.state.objects[source as usize].tapped = false;
            g.state.objects[source as usize].summoning_sick = false;
            unblocked(&mut g, source);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            settle_target(&mut g, dead);
            assert_eq!(g.state.objects[dead as usize].zone, Zone::Hand);
        }
    }
}
#[test]
fn tmt_sneak_gadget_master_copies_artifact_copiable_values_and_real_entry_triggers() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            let mut g = setup(p);
            let returned = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let foundry = named(&mut g, p, "Mouser Foundry", Zone::Battlefield);
            let foreign = named(&mut g, p.other(), "Mouser Foundry", Zone::Battlefield);
            let source = named(&mut g, p, "Donatello, Gadget Master", Zone::Hand);
            if paid {
                unblocked(&mut g, returned);
                sneak(&mut g, source, returned, None);
                settle(&mut g);
            } else {
                cast(&mut g, source, None, &[]);
                settle(&mut g);
                g.state.objects[source as usize].summoning_sick = false;
                unblocked(&mut g, source);
            }
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            g.prepare_triggers();
            for invalid in [source, foreign] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(invalid),
                    },
                );
            }
            assert!(g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(foundry)
            }));
            let count = g.state.objects.len();
            settle_target(&mut g, foundry);
            assert_eq!(g.state.players[p.other().index()].life, 17);
            let copy = (count..g.state.objects.len())
                .map(|id| id as CardId)
                .find(|&id| {
                    g.state.objects[id as usize].zone == Zone::Battlefield
                        && g.object_view(id).card == g.object_view(foundry).card
                })
                .unwrap();
            assert!(g.object_view(copy).is_token());
            assert_eq!(g.state.objects[copy as usize].controller, p);
            assert_eq!(g.state.objects.len(), count + 2); // copy plus its actual Robot ETB
            g.move_object(source, Zone::Exile);
            g.move_object(foundry, Zone::Exile);
            settle(&mut g);
            assert_eq!(g.state.objects[copy as usize].zone, Zone::Battlefield);
            g.move_object(copy, Zone::Exile);
            settle(&mut g);
            assert!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.card == Card::from_name("@token/667dad32c9f20b70d5e9bda3").unwrap())
                    .count()
                    >= 3
            );
        }
    }
}
#[test]
fn tmt_sneak_kitsune_mills_opponent_half_rounded_up_from_current_library() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            for size in [0, 1, 2, 5, 6] {
                let mut g = setup(p);
                // Clear only the opponent library to exercise the zero and one-card boundaries.
                let old = g.state.players[p.other().index()].library.to_vec();
                for id in old {
                    g.move_object(id, Zone::Exile);
                }
                let returned = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let source = named(&mut g, p, "Kitsune's Technique", Zone::Hand);
                for _ in 0..size {
                    named(&mut g, p.other(), "Forest", Zone::Library);
                }
                let before = g.state.players[p.other().index()].library.len();
                let own = g.state.players[p.index()].library.len();
                if paid {
                    unblocked(&mut g, returned);
                    let attacker = g.object_ref(returned);
                    checked(
                        &mut g,
                        Action::AnnounceSneak {
                            card: source,
                            attacker,
                        },
                    );
                } else {
                    checked(&mut g, Action::AnnounceCast { card: source });
                }
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(p),
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    },
                );
                for _ in 0..8 {
                    if g.state.announcing.is_none() {
                        break;
                    }
                    let a = g
                        .legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                        .unwrap();
                    checked(&mut g, a);
                }
                settle(&mut g);
                assert_eq!(g.state.players[p.other().index()].library.len(), before / 2);
                assert_eq!(g.state.players[p.index()].library.len(), own);
                assert_eq!(
                    g.state
                        .objects
                        .iter()
                        .filter(|o| o.zone == Zone::Graveyard && o.owner == p.other())
                        .count(),
                    before.div_ceil(2)
                );
                assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
            }
        }
    }
}
#[test]
fn tmt_sneak_dark_leo_token_counts_before_half_loss_and_only_attacking_own_ninjas_gain_dt() {
    for p in [Player::P0, Player::P1] {
        for others in [2, 3, 4] {
            let mut g = setup(p);
            let returned = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let source = named(&mut g, p, "Dark Leo & Shredder", Zone::Hand);
            let mut ninjas = Vec::new();
            for _ in 0..others {
                ninjas.push(named(
                    &mut g,
                    p,
                    "@token/ninja_1_1_black",
                    Zone::Battlefield,
                ));
            }
            let enemy = named(
                &mut g,
                p.other(),
                "@token/ninja_1_1_black",
                Zone::Battlefield,
            );
            unblocked(&mut g, returned);
            sneak(&mut g, source, returned, None);
            settle(&mut g);
            assert!(g.keywords(source).contains(Keywords::DEATHTOUCH));
            for id in ninjas.iter().copied().chain([enemy]) {
                assert!(!g.keywords(id).contains(Keywords::DEATHTOUCH));
            }
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            settle(&mut g);
            // One damage leaves 19, then a qualifying five-Ninja board loses ceil(19/2).
            assert_eq!(
                g.state.players[p.other().index()].life,
                if others >= 3 { 9 } else { 19 }
            );
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.card.has_subtype("Ninja"))
                    .count(),
                others + 2
            );
        }
    }
}
#[test]
fn tmt_sneak_last_ronin_creates_white_spirits_with_individual_defenders_and_no_sacrifice() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            let mut g = setup(p);
            let returned = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let source = named(&mut g, p, "The Last Ronin's Technique", Zone::Hand);
            let walker = named(&mut g, p.other(), "Kaito, Cunning Infiltrator", Zone::Hand);
            g.move_object(walker, Zone::Battlefield);
            let reference = g.object_ref(walker);
            let loyalty = g.counter_count(walker, CounterKind::Loyalty);
            if paid {
                attackers(&mut g, returned, Some(reference));
                checked(&mut g, Action::FinishBlockers);
                sneak(&mut g, source, returned, None);
                checked(&mut g, Action::Pass);
                checked(&mut g, Action::Pass);
                assert!(matches!(g.state.pending, Pending::Mobilize { .. }));
                assert!(g.observe(p.other()).mobilize_choice.is_none());
                reject(
                    &mut g,
                    Action::ChooseMobilizeDefender {
                        defender: CombatDefender::Player(p),
                    },
                );
                for defender in [
                    CombatDefender::Player(p.other()),
                    CombatDefender::Planeswalker(reference),
                    CombatDefender::Player(p.other()),
                ] {
                    checked(&mut g, Action::ChooseMobilizeDefender { defender });
                }
            } else {
                cast(&mut g, source, None, &[]);
            }
            settle(&mut g);
            let tokens: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(_, o)| {
                    o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.card
                            == Card::from_name("@token/ninja_turtle_spirit_1_1_white").unwrap()
                })
                .map(|(i, _)| i as CardId)
                .collect();
            assert_eq!(tokens.len(), 3);
            for &id in &tokens {
                let view = g.object_view(id);
                assert!(view.is_token());
                assert_eq!((view.power, view.toughness), (1, 1));
                assert_eq!(view.colors, 1);
                for subtype in ["Ninja", "Turtle", "Spirit"] {
                    assert!(g.has_subtype(id, subtype));
                }
                assert_eq!(view.tapped, paid);
                assert_eq!(view.attacking, paid);
            }
            assert!(g.observe(p).delayed_sacrifices.is_empty());
            if paid {
                checked(&mut g, Action::Pass);
                checked(&mut g, Action::Pass);
                settle(&mut g);
                assert_eq!(g.state.players[p.other().index()].life, 18);
                assert_eq!(g.counter_count(walker, CounterKind::Loyalty), loyalty - 1);
            }
            super::ability_timeline_tests::next_turn(&mut g);
            assert!(tokens.iter().all(|&id| g.on_battlefield(id)));
        }
    }
}
#[test]
fn tmt_sneak_michelangelo_private_top_eight_total_mana_budget_and_random_bottom_commit() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            for count in 0..=2 {
                let mut g = setup(p);
                let returned = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let source = named(&mut g, p, "Michelangelo's Technique", Zone::Hand);
                let original = g.state.players[p.index()].library.to_vec();
                let mut top = Vec::new();
                for name in [
                    "Plains",
                    "Forest",
                    "Opt",
                    "Mountain",
                    "Grasping Longneck",
                    "Bear Cub",
                    "Hill Giant",
                    "Island",
                ] {
                    top.push(named(&mut g, p, name, Zone::Library));
                }
                if paid {
                    unblocked(&mut g, returned);
                    sneak(&mut g, source, returned, None);
                } else {
                    cast(&mut g, source, None, &[]);
                }
                checked(&mut g, Action::Pass);
                checked(&mut g, Action::Pass);
                let own = g.observe(p).library_selection.unwrap();
                assert_eq!(
                    (
                        own.remaining.len(),
                        own.minimum,
                        own.maximum,
                        own.public,
                        own.reveal
                    ),
                    (8, 0, 2, false, false)
                );
                assert!(g.observe(p.other()).library_selection.is_none());
                assert!(g.observe(p.other()).actions.is_empty());
                for &id in &top {
                    assert!(
                        !g.observe(p.other())
                            .objects
                            .iter()
                            .any(|o| o.reference.card == id)
                    );
                }
                reject(&mut g, Action::SelectLibrary { card: top[0] }); // land
                let mut chosen = Vec::new();
                if count > 0 {
                    checked(&mut g, Action::SelectLibrary { card: top[6] }); // four
                    chosen.push(top[6]);
                    reject(&mut g, Action::SelectLibrary { card: top[6] });
                    reject(&mut g, Action::SelectLibrary { card: top[4] }); // four + three exceeds six
                    assert_eq!(g.state.objects[top[6] as usize].zone, Zone::Library);
                }
                if count == 2 {
                    checked(&mut g, Action::SelectLibrary { card: top[5] }); // four + two is legal
                    chosen.push(top[5]);
                    reject(&mut g, Action::SelectLibrary { card: top[4] });
                }
                checked(&mut g, Action::FinishLibrarySelection);
                settle(&mut g);
                for &id in &chosen {
                    let view = g.object_view(id);
                    assert_eq!(view.zone, Zone::Battlefield);
                    assert!(!view.tapped && !view.attacking);
                }
                let mut unchosen: Vec<_> = top
                    .iter()
                    .copied()
                    .filter(|id| !chosen.contains(id))
                    .collect();
                unchosen.sort_unstable();
                let library = &g.state.players[p.index()].library;
                let mut bottom = library[..unchosen.len()].to_vec();
                bottom.sort_unstable();
                assert_eq!(bottom, unchosen);
                assert_eq!(&library[unchosen.len()..], original.as_slice());
                assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
            }
        }
    }
}
#[test]
fn tmt_sneak_shark_first_strike_reanimates_damaged_players_creature_attacking_same_player() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            for change in 0..3 {
                let mut g = setup(p);
                let returned = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let source = named(&mut g, p, "Shark Shredder, Killer Clone", Zone::Hand);
                let dead = named(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
                let own = named(&mut g, p, "Bear Cub", Zone::Graveyard);
                let land = named(&mut g, p.other(), "Forest", Zone::Graveyard);
                if paid {
                    unblocked(&mut g, returned);
                    sneak(&mut g, source, returned, None);
                    settle(&mut g);
                } else {
                    cast(&mut g, source, None, &[]);
                    settle(&mut g);
                    g.state.objects[source as usize].summoning_sick = false;
                    unblocked(&mut g, source);
                }
                assert!(g.keywords(source).contains(Keywords::FIRST_STRIKE));
                checked(&mut g, Action::Pass);
                checked(&mut g, Action::Pass);
                g.prepare_triggers();
                assert_eq!(g.state.players[p.other().index()].life, 16);
                for id in [own, land] {
                    reject(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(id),
                        },
                    );
                }
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(dead),
                    },
                );
                if g.legal_actions().contains(&Action::FinishTargets) {
                    checked(&mut g, Action::FinishTargets);
                }
                if change == 1 {
                    g.move_object(source, Zone::Exile);
                }
                if change == 2 {
                    let r = g.object_ref(source);
                    g.record_control(r, p.other(), None);
                }
                settle(&mut g);
                let view = g.object_view(dead);
                assert_eq!(view.zone, Zone::Battlefield);
                assert_eq!(view.controller, p);
                assert_eq!(view.owner, p.other());
                assert!(view.tapped && view.attacking);
                assert_eq!(
                    g.state
                        .attackers
                        .iter()
                        .find(|a| a.card == dead)
                        .unwrap()
                        .defender,
                    Some(CombatDefender::Player(p.other()))
                );
                checked(&mut g, Action::Pass);
                checked(&mut g, Action::Pass);
                settle(&mut g);
                assert_eq!(g.state.players[p.other().index()].life, 14);
            }
        }
    }
}
#[test]
fn tmt_sneak_shark_optional_zero_or_invalidated_grave_target_makes_no_new_attacker() {
    for p in [Player::P0, Player::P1] {
        for choose in [false, true] {
            let mut g = setup(p);
            let returned = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let source = named(&mut g, p, "Shark Shredder, Killer Clone", Zone::Hand);
            let dead = named(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
            unblocked(&mut g, returned);
            sneak(&mut g, source, returned, None);
            settle(&mut g);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            g.prepare_triggers();
            if choose {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(dead),
                    },
                );
            }
            if g.legal_actions().contains(&Action::FinishTargets) {
                checked(&mut g, Action::FinishTargets);
            }
            if choose {
                g.move_object(dead, Zone::Exile);
                g.move_object(dead, Zone::Graveyard);
            }
            settle(&mut g);
            assert_eq!(g.state.objects[dead as usize].zone, Zone::Graveyard);
            assert!(!g.state.attackers.iter().any(|a| a.card == dead));
        }
    }
}
#[test]
fn tmt_sneak_shredder_technique_counts_actual_destroyed_enchantment_lki_before_death_triggers() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            for owner in [p, p.other()] {
                for name in ["Bear Cub", "Doubling Season", "Grasping Longneck"] {
                    for indestructible in [false, true] {
                        let mut g = setup(p);
                        let returned = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                        let target = named(&mut g, owner, name, Zone::Battlefield);
                        let bad_land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
                        let bad_artifact =
                            named(&mut g, p.other(), "Mouser Foundry", Zone::Battlefield);
                        let source = named(&mut g, p, "Shredder's Technique", Zone::Hand);
                        if indestructible {
                            let r = g.object_ref(target);
                            g.grant_keyword_effects(&[target], r, Keywords::INDESTRUCTIBLE);
                        }
                        if paid {
                            unblocked(&mut g, returned);
                            let attacker = g.object_ref(returned);
                            checked(
                                &mut g,
                                Action::AnnounceSneak {
                                    card: source,
                                    attacker,
                                },
                            );
                        } else {
                            checked(&mut g, Action::AnnounceCast { card: source });
                        }
                        for id in [bad_land, bad_artifact] {
                            reject(
                                &mut g,
                                Action::ChooseTarget {
                                    target: Target::Card(id),
                                },
                            );
                        }
                        checked(
                            &mut g,
                            Action::ChooseTarget {
                                target: Target::Card(target),
                            },
                        );
                        while g.state.announcing.is_some() {
                            let a = g
                                .legal_actions()
                                .into_iter()
                                .find(|a| {
                                    matches!(a, Action::PayCosts { .. } | Action::FinishTargets)
                                })
                                .unwrap();
                            checked(&mut g, a);
                        }
                        settle(&mut g);
                        assert_eq!(
                            g.state.objects[target as usize].zone,
                            if indestructible {
                                Zone::Battlefield
                            } else {
                                Zone::Graveyard
                            }
                        );
                        let loss = if !indestructible && name != "Bear Cub" {
                            2
                        } else {
                            0
                        };
                        let gain = if !indestructible && owner == p && name == "Grasping Longneck" {
                            2
                        } else {
                            0
                        };
                        assert_eq!(g.state.players[p.index()].life, 20 - loss + gain);
                    }
                }
            }
        }
    }
}
#[test]
fn tmt_sneak_shredder_technique_invalidated_enchantment_target_never_charges_life() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Shredder's Technique", Zone::Hand);
        let target = named(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
        cast(&mut g, source, None, &[Target::Card(target)]);
        g.move_object(target, Zone::Exile);
        g.move_object(target, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.objects[target as usize].zone, Zone::Battlefield);
    }
}
fn to_hand_pair(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..30 {
        if matches!(g.state.pending, Pending::EffectChoice { .. })
            && g.state
                .resolving
                .as_ref()
                .unwrap()
                .choice
                .as_ref()
                .unwrap()
                .kind
                == EffectChoiceKind::HandCreatureAndLand
        {
            return;
        }
        let a = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap()
        };
        checked(g, a);
    }
    panic!("missing private hand placement");
}
#[test]
fn tmt_sneak_improviser_private_optional_hand_pair_is_simultaneous_and_bypasses_land_play_limit() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            for choice in 0..4 {
                for leave in [false, true] {
                    let mut g = setup(p);
                    g.state.land_played = true;
                    let returned = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                    let source = named(&mut g, p, "Michelangelo, Improviser", Zone::Hand);
                    let creature = named(&mut g, p, "Rampaging Baloths", Zone::Hand);
                    let land = named(&mut g, p, "Forest", Zone::Hand);
                    let extra_creature = named(&mut g, p, "Bear Cub", Zone::Hand);
                    let extra_land = named(&mut g, p, "Island", Zone::Hand);
                    let wrong = named(&mut g, p, "Opt", Zone::Hand);
                    let foreign = named(&mut g, p.other(), "Forest", Zone::Hand);
                    if paid {
                        unblocked(&mut g, returned);
                        sneak(&mut g, source, returned, None);
                        settle(&mut g);
                    } else {
                        cast(&mut g, source, None, &[]);
                        settle(&mut g);
                        g.state.objects[source as usize].summoning_sick = false;
                        unblocked(&mut g, source);
                    }
                    checked(&mut g, Action::Pass);
                    checked(&mut g, Action::Pass);
                    assert_eq!(g.state.players[p.other().index()].life, 16);
                    if leave {
                        g.move_object(source, Zone::Exile);
                    }
                    to_hand_pair(&mut g);
                    assert!(g.observe(p.other()).effect_choice.is_none());
                    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                    let bad = g.object_ref(foreign);
                    reject(&mut g, Action::PutHandLand { object: bad });
                    let bad = g.object_ref(wrong);
                    reject(&mut g, Action::PutHandCreature { object: bad });
                    if choice & 2 != 0 {
                        let object = g.object_ref(land);
                        checked(&mut g, Action::PutHandLand { object });
                        assert_eq!(g.state.objects[land as usize].zone, Zone::Hand);
                        let object = g.object_ref(extra_land);
                        reject(&mut g, Action::PutHandLand { object });
                    }
                    if choice & 1 != 0 {
                        let object = g.object_ref(creature);
                        checked(&mut g, Action::PutHandCreature { object });
                        if choice == 1 {
                            assert_eq!(g.state.objects[creature as usize].zone, Zone::Hand);
                            let object = g.object_ref(extra_creature);
                            reject(&mut g, Action::PutHandCreature { object });
                        }
                    }
                    if g.legal_actions().contains(&Action::FinishHandSelection) {
                        checked(&mut g, Action::FinishHandSelection);
                    }
                    settle(&mut g);
                    for (id, selected) in [(creature, choice & 1 != 0), (land, choice & 2 != 0)] {
                        assert_eq!(
                            g.state.objects[id as usize].zone,
                            if selected {
                                Zone::Battlefield
                            } else {
                                Zone::Hand
                            }
                        );
                        if selected {
                            let v = g.object_view(id);
                            assert!(!v.tapped && !v.attacking);
                            assert_eq!(v.controller, p);
                        }
                    }
                    assert!(g.state.land_played);
                    let beasts = g
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(id, o)| {
                            o.zone == Zone::Battlefield
                                && o.controller == p
                                && g.has_subtype(*id as CardId, "Beast")
                        })
                        .count();
                    assert_eq!(
                        beasts,
                        usize::from(choice & 1 != 0) + usize::from(choice == 3)
                    ); // land-first choice still triggers the simultaneously entering Baloths
                }
            }
        }
    }
}
#[test]
fn tmt_sneak_improviser_empty_hand_still_offers_private_decline_and_foreign_return_is_not_eligible()
{
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            let mut g = setup(p);
            let returned = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let r = g.object_ref(returned);
            g.record_control(r, p, None);
            g.state.objects[returned as usize].summoning_sick = false;
            let source = named(&mut g, p, "Michelangelo, Improviser", Zone::Hand);
            if paid {
                unblocked(&mut g, returned);
                sneak(&mut g, source, returned, None);
                settle(&mut g);
            } else {
                cast(&mut g, source, None, &[]);
                settle(&mut g);
                g.state.objects[source as usize].summoning_sick = false;
                unblocked(&mut g, source);
            }
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            to_hand_pair(&mut g);
            assert!(g.observe(p.other()).effect_choice.is_none());
            assert!(g.observe(p).effect_choice.unwrap().candidates.is_empty());
            assert_eq!(
                g.legal_actions()
                    .into_iter()
                    .filter(|a| !matches!(a, Action::Concede))
                    .collect::<Vec<_>>(),
                vec![Action::FinishHandSelection]
            );
            checked(&mut g, Action::FinishHandSelection);
            settle(&mut g);
        }
    }
}
#[test]
fn tmt_sneak_improviser_placed_enchantment_creature_enters_untapped_without_attacking() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let returned = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p, "Michelangelo, Improviser", Zone::Hand);
        let card = named(&mut g, p, "Grasping Longneck", Zone::Hand);
        unblocked(&mut g, returned);
        sneak(&mut g, source, returned, None);
        settle(&mut g);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        to_hand_pair(&mut g);
        let object = g.object_ref(card);
        checked(&mut g, Action::PutHandCreature { object });
        checked(&mut g, Action::FinishHandSelection);
        settle(&mut g);
        let view = g.object_view(card);
        assert_eq!(view.zone, Zone::Battlefield);
        assert!(!view.tapped && !view.attacking);
    }
}
#[test]
fn tmt_sneak_raphael_public_apnap_choices_precede_simultaneous_discards_and_seven_draws() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            for enemy_turn in [false, true] {
                if paid && enemy_turn {
                    continue;
                }
                for mask in 0..4 {
                    let mut g = setup(p);
                    if enemy_turn {
                        g.state.active = p.other();
                        g.state.priority = p;
                    }
                    let returned = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                    let source = named(&mut g, p, "Raphael's Technique", Zone::Hand);
                    for q in [Player::P0, Player::P1] {
                        named(&mut g, q, "Opt", Zone::Hand);
                        named(&mut g, q, "Bear Cub", Zone::Hand);
                    }
                    if paid {
                        unblocked(&mut g, returned);
                        sneak(&mut g, source, returned, None);
                    } else {
                        cast(&mut g, source, None, &[]);
                    }
                    checked(&mut g, Action::Pass);
                    checked(&mut g, Action::Pass);
                    let first = g.state.active;
                    assert_eq!(g.actor(), Some(first));
                    assert!(matches!(g.state.pending, Pending::Wheel { .. }));
                    assert_eq!(g.observe(p).wheel_choice.unwrap().decisions, [None, None]);
                    let hands: [Vec<_>; 2] = std::array::from_fn(|index| {
                        g.state
                            .objects
                            .iter()
                            .enumerate()
                            .filter(|(_, o)| o.zone == Zone::Hand && o.owner.index() == index)
                            .map(|(id, _)| id as CardId)
                            .collect()
                    });
                    let libraries: [usize; 2] =
                        std::array::from_fn(|index| g.state.players[index].library.len());
                    checked(
                        &mut g,
                        Action::ChooseOptional {
                            accept: mask & (1 << first.index()) != 0,
                        },
                    );
                    assert_eq!(g.actor(), Some(first.other()));
                    for &id in hands.iter().flatten() {
                        assert_eq!(g.state.objects[id as usize].zone, Zone::Hand);
                    }
                    let view = g.observe(p).wheel_choice.unwrap();
                    assert_eq!(view.draw, 7);
                    assert_eq!(
                        view.decisions[first.index()],
                        Some(mask & (1 << first.index()) != 0)
                    );
                    assert_eq!(view.decisions[first.other().index()], None);
                    assert_eq!(Some(view), g.observe(p.other()).wheel_choice);
                    checked(
                        &mut g,
                        Action::ChooseOptional {
                            accept: mask & (1 << first.other().index()) != 0,
                        },
                    );
                    settle(&mut g);
                    for q in [Player::P0, Player::P1] {
                        let accepted = mask & (1 << q.index()) != 0;
                        assert_eq!(
                            g.hand_count(q),
                            if accepted { 7 } else { hands[q.index()].len() }
                        );
                        assert_eq!(
                            g.state.players[q.index()].library.len(),
                            libraries[q.index()] - if accepted { 7 } else { 0 }
                        );
                        assert_eq!(
                            g.state.players[q.index()].cards_drawn_this_turn,
                            if accepted { 7 } else { 0 }
                        );
                        assert_eq!(
                            g.state.players[q.index()].cards_discarded_this_turn,
                            if accepted {
                                hands[q.index()].len() as u64
                            } else {
                                0
                            }
                        );
                        for &id in &hands[q.index()] {
                            assert_eq!(
                                g.state.objects[id as usize].zone,
                                if accepted {
                                    Zone::Graveyard
                                } else {
                                    Zone::Hand
                                }
                            );
                        }
                    }
                    assert!(g.observe(p).wheel_choice.is_none());
                    assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
                }
            }
        }
    }
}
#[test]
fn tmt_sneak_raphael_can_refresh_empty_hands_and_both_empty_library_draws_lose_together() {
    for p in [Player::P0, Player::P1] {
        for empty in [false, true] {
            let mut g = setup(p);
            if empty {
                for q in [Player::P0, Player::P1] {
                    let ids = g.state.players[q.index()].library.to_vec();
                    for id in ids {
                        g.move_object(id, Zone::Exile);
                    }
                }
            }
            let source = named(&mut g, p, "Raphael's Technique", Zone::Hand);
            cast(&mut g, source, None, &[]);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::ChooseOptional { accept: true });
            checked(&mut g, Action::ChooseOptional { accept: true });
            if empty {
                assert_eq!(g.outcome(), Some(Outcome::Draw));
            } else {
                settle(&mut g);
                assert_eq!(g.outcome(), None);
                for q in [Player::P0, Player::P1] {
                    assert_eq!(g.hand_count(q), 7);
                }
            }
        }
    }
}
#[test]
fn tmt_sneak_raphael_last_answer_and_all_discard_events_roll_back_on_capacity_failure() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, "Raphael's Technique", Zone::Hand);
    let first = named(&mut g, Player::P0, "Opt", Zone::Hand);
    let second = named(&mut g, Player::P1, "Opt", Zone::Hand);
    cast(&mut g, source, None, &[]);
    checked(&mut g, Action::Pass);
    checked(&mut g, Action::Pass);
    checked(&mut g, Action::ChooseOptional { accept: true });
    g.state.objects[second as usize].generation = u64::MAX;
    let snapshot = g.snapshot();
    let revision = g.revision();
    let views =
        [g.observe(Player::P0), g.observe(Player::P1)].map(|o| serde_json::to_value(o).unwrap());
    assert!(matches!(
        g.apply(Player::P1, Action::ChooseOptional { accept: true }),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), snapshot);
    assert_eq!(g.revision(), revision);
    assert_eq!(
        [g.observe(Player::P0), g.observe(Player::P1)].map(|o| serde_json::to_value(o).unwrap()),
        views
    );
    assert_eq!(g.state.objects[first as usize].zone, Zone::Hand);
    assert_eq!(
        g.observe(Player::P0).wheel_choice.unwrap().decisions,
        [Some(true), None]
    );
    checked(&mut g, Action::ChooseOptional { accept: false });
    settle(&mut g);
    assert_eq!(g.hand_count(Player::P0), 7);
    assert_eq!(g.state.objects[second as usize].zone, Zone::Hand);
}

#[test]
fn tmt_sneak_raphael_opponent_caused_discard_replacement_enters_before_draws_resume() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Raphael's Technique", Zone::Hand);
        let own = named(&mut g, p, "Wilt-Leaf Liege", Zone::Hand);
        let enemy = named(&mut g, p.other(), "Wilt-Leaf Liege", Zone::Hand);
        cast(&mut g, source, None, &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::ChooseOptional { accept: true });
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.state.objects[own as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[enemy as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[enemy as usize].controller, p.other());
        for q in [Player::P0, Player::P1] {
            assert_eq!(g.hand_count(q), 7);
            assert_eq!(g.state.players[q.index()].cards_drawn_this_turn, 7);
        }
    }
}

#[test]
fn tmt_sneak_raphael_cannot_choose_impossible_draw_under_mornsong_aria() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            let mut g = setup(p);
            named(&mut g, p.other(), "Mornsong Aria", Zone::Battlefield);
            let returned = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let source = named(&mut g, p, "Raphael's Technique", Zone::Hand);
            named(&mut g, p, "Opt", Zone::Hand);
            named(&mut g, p.other(), "Opt", Zone::Hand);
            if paid {
                unblocked(&mut g, returned);
                sneak(&mut g, source, returned, None);
            } else {
                cast(&mut g, source, None, &[]);
            }
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            let before = [g.hand_count(Player::P0), g.hand_count(Player::P1)];
            for _ in 0..2 {
                assert!(
                    !g.legal_actions()
                        .contains(&Action::ChooseOptional { accept: true })
                );
                reject(&mut g, Action::ChooseOptional { accept: true });
                checked(&mut g, Action::ChooseOptional { accept: false });
            }
            settle(&mut g);
            for q in [Player::P0, Player::P1] {
                assert_eq!(g.hand_count(q), before[q.index()]);
                assert_eq!(g.state.players[q.index()].cards_discarded_this_turn, 0);
                assert_eq!(g.state.players[q.index()].cards_drawn_this_turn, 0);
            }
        }
    }
}

fn grave_spell(
    card: CardId,
    permission: ObjectRef,
    method: CastMethod,
    attacker: Option<ObjectRef>,
) -> Action {
    Action::AnnounceGraveyardSpell {
        card,
        permission,
        permanent_type: None,
        method,
        attacker,
        flash_surcharge: false,
    }
}
fn finish_grave_spell(g: &mut Game) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .expect("grave spell payment");
        checked(g, a);
    }
    panic!("grave spell not paid");
}
#[test]
fn tmt_sneak_sewer_samurai_graveyard_filter_turn_timing_ownership_and_independent_permission() {
    assert_eq!(std::mem::size_of::<CastInfo>(), 40);
    assert_eq!(std::mem::size_of::<Object>(), 144);
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let leo = named(&mut g, p, "Leonardo, Sewer Samurai", Zone::Battlefield);
        let permission = g.object_ref(leo);
        let lion = named(&mut g, p, "Savannah Lions", Zone::Graveyard); // 2/1: toughness qualifies.
        let oroku = named(&mut g, p, "Oroku Saki, Shredder Rising", Zone::Graveyard); // 3/1.
        let wall = named(&mut g, p, "Aegis Turtle", Zone::Graveyard); // 0/5: power qualifies.
        let bear = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let foreign = named(&mut g, p.other(), "Savannah Lions", Zone::Graveyard);
        let bolt = named(&mut g, p, "Lightning Bolt", Zone::Graveyard);
        for id in [lion, oroku, wall] {
            assert!(g.legal_actions().contains(&grave_spell(
                id,
                permission,
                CastMethod::Normal,
                None
            )));
        }
        for id in [bear, foreign, bolt] {
            reject(
                &mut g,
                grave_spell(id, permission, CastMethod::Normal, None),
            );
        }
        reject(&mut g, Action::AnnounceCast { card: lion });
        g.state.active = p.other();
        reject(
            &mut g,
            grave_spell(lion, permission, CastMethod::Normal, None),
        );
        g.state.active = p;
        g.state.step = Step::Blockers;
        reject(
            &mut g,
            grave_spell(lion, permission, CastMethod::Normal, None),
        );
        g.state.step = Step::Main1;
        let m = named(&mut g, p, "Muldrotha, the Gravetide", Zone::Battlefield);
        let m_permission = g.object_ref(m);
        checked(
            &mut g,
            Action::PlayGraveyardPermanent {
                card: lion,
                permission: m_permission,
                permanent_type: PermanentType::Creature,
            },
        );
        finish_grave_spell(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(lion, CounterKind::Finality), 0);
        assert_eq!(g.observe(p).graveyard_type_uses.len(), 1);
        checked(
            &mut g,
            grave_spell(wall, permission, CastMethod::Normal, None),
        );
        finish_grave_spell(&mut g);
        g.move_object(leo, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.counter_count(wall, CounterKind::Finality), 1);
        assert_eq!(g.observe(p).graveyard_type_uses.len(), 1);
        reject(
            &mut g,
            grave_spell(oroku, permission, CastMethod::Normal, None),
        );
        g.move_object(leo, Zone::Battlefield);
        reject(
            &mut g,
            grave_spell(oroku, permission, CastMethod::Normal, None),
        ); // old incarnation.
        assert!(g.legal_actions().contains(&grave_spell(
            oroku,
            g.object_ref(leo),
            CastMethod::Normal,
            None
        )));
    }
}
#[test]
fn tmt_sneak_sewer_permission_survives_returning_provider_and_preserves_sneak_defender() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let leo = named(&mut g, p, "Leonardo, Sewer Samurai", Zone::Battlefield);
        let oroku = named(&mut g, p, "Oroku Saki, Shredder Rising", Zone::Graveyard);
        let permission = g.object_ref(leo);
        unblocked(&mut g, leo);
        checked(
            &mut g,
            grave_spell(oroku, permission, CastMethod::Sneak, Some(permission)),
        );
        finish_grave_spell(&mut g);
        assert_eq!(g.state.objects[leo as usize].zone, Zone::Hand);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().method,
            CastMethod::Sneak
        );
        settle(&mut g);
        assert_eq!(g.counter_count(oroku, CounterKind::Finality), 1);
        assert!(g.state.objects[oroku as usize].tapped);
        assert!(
            g.state
                .attackers
                .iter()
                .any(|a| a.card == oroku && a.defender == Some(CombatDefender::Player(p.other())))
        );
        let murder = named(&mut g, p, "Murder", Zone::Hand);
        cast(&mut g, murder, None, &[Target::Card(oroku)]);
        settle(&mut g);
        assert_eq!(g.state.objects[oroku as usize].zone, Zone::Exile);
        assert_eq!(g.counter_count(oroku, CounterKind::Finality), 0);
    }
}
#[test]
fn tmt_sneak_muldrotha_can_choose_sneak_without_samurai_finality() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let m = named(&mut g, p, "Muldrotha, the Gravetide", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let leo = named(&mut g, p, "Leonardo, Sewer Samurai", Zone::Graveyard);
        unblocked(&mut g, bear);
        let a = Action::AnnounceGraveyardSpell {
            card: leo,
            permission: g.object_ref(m),
            permanent_type: Some(PermanentType::Creature),
            method: CastMethod::Sneak,
            attacker: Some(g.object_ref(bear)),
            flash_surcharge: false,
        };
        checked(&mut g, a);
        finish_grave_spell(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(leo, CounterKind::Finality), 0);
        assert_eq!(g.observe(p).graveyard_type_uses.len(), 1);
        assert!(g.keywords(leo).contains(Keywords::DOUBLE_STRIKE));
        let life = g.state.players[p.other().index()].life;
        while g.state.step != Step::Main2 {
            checked(&mut g, Action::Pass);
        }
        assert_eq!(g.state.players[p.other().index()].life, life - 6);
    }
}
#[test]
fn tmt_sneak_finality_intrinsic_on_abilityless_and_noncreature_permanents_and_clears_on_bounce() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let lion = named(&mut g, p, "Savannah Lions", Zone::Battlefield);
        let artifact = named(&mut g, p, "Mouser Foundry", Zone::Battlefield);
        g.set_counters(lion, Counters::of(CounterKind::Finality, 2));
        g.set_counters(artifact, Counters::of(CounterKind::Finality, 1));
        let aura = named(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(lion)]);
        settle(&mut g);
        let murder = named(&mut g, p, "Murder", Zone::Hand);
        cast(&mut g, murder, None, &[Target::Card(lion)]);
        settle(&mut g);
        assert_eq!(g.state.objects[lion as usize].zone, Zone::Exile);
        let sage = named(&mut g, p, "Reclamation Sage", Zone::Hand);
        cast(&mut g, sage, None, &[]);
        // Resolve Sage and its targeted entry trigger through existing scenario machinery.
        for _ in 0..20 {
            if g.state.stack.is_empty()
                && g.state.resolving.is_none()
                && g.state.announcing.is_none()
            {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(
                    |a| matches!(a,Action::ChooseTarget{target:Target::Card(id)} if *id==artifact),
                )
                .or_else(|| {
                    g.legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::Pass | Action::FinishTargets))
                })
                .or_else(|| {
                    g.legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::ChooseOptional { accept: true }))
                })
                .expect("Sage resolution");
            checked(&mut g, a);
        }
        assert_eq!(g.state.objects[artifact as usize].zone, Zone::Exile);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.set_counters(bear, Counters::of(CounterKind::Finality, 1));
        let unsummon = named(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, unsummon, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Hand);
        assert_eq!(g.counter_count(bear, CounterKind::Finality), 0);
        cast(&mut g, bear, None, &[]);
        settle(&mut g);
        let murder = named(&mut g, p, "Murder", Zone::Hand);
        cast(&mut g, murder, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Graveyard);
    }
}

#[test]
fn tmt_sneak_sewer_permission_lost_ability_and_countered_spell_have_no_finality() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let leo = named(&mut g, p, "Leonardo, Sewer Samurai", Zone::Battlefield);
        let lion = named(&mut g, p, "Savannah Lions", Zone::Graveyard);
        let permission = g.object_ref(leo);
        checked(
            &mut g,
            grave_spell(lion, permission, CastMethod::Normal, None),
        );
        finish_grave_spell(&mut g);
        checked(&mut g, Action::Pass);
        let counter = named(&mut g, p.other(), "Negate", Zone::Hand);
        // A creature cannot be Negated; use the unconditional Counterspell.
        reject(&mut g, Action::AnnounceCast { card: counter });
        let counter = named(&mut g, p.other(), "Counterspell", Zone::Hand);
        cast(&mut g, counter, None, &[Target::Spell(lion)]);
        settle(&mut g);
        assert_eq!(g.state.objects[lion as usize].zone, Zone::Graveyard);
        assert_eq!(g.counter_count(lion, CounterKind::Finality), 0);
        g.state.priority = p;
        let aura = named(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(leo)]);
        settle(&mut g);
        reject(
            &mut g,
            grave_spell(lion, permission, CastMethod::Normal, None),
        );
    }
}
#[test]
fn tmt_sneak_finality_competes_with_shuffle_once_and_is_not_a_keyword_counter() {
    for p in [Player::P0, Player::P1] {
        for choice in [
            ZoneReplacementKind::Exile,
            ZoneReplacementKind::RevealShuffle,
        ] {
            let mut g = setup(p);
            let colossus = named(&mut g, p, "Darksteel Colossus", Zone::Battlefield);
            let ghoul = named(&mut g, p, "Hungry Ghoul", Zone::Battlefield);
            g.set_counters(colossus, Counters::of(CounterKind::Finality, 3));
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: ghoul,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            let reference = g.object_ref(colossus);
            checked(&mut g, Action::ChooseAdditionalCost { object: reference });
            for _ in 0..8 {
                if matches!(g.state.pending, Pending::ZoneReplacement { .. }) {
                    break;
                }
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                    .expect("sacrifice payment");
                checked(&mut g, a);
            }
            assert_eq!(
                g.observe(p).zone_replacement_choice.unwrap().choices,
                [
                    ZoneReplacementKind::Exile,
                    ZoneReplacementKind::RevealShuffle
                ]
            );
            checked(&mut g, Action::ChooseZoneReplacement { effect: choice });
            assert_eq!(
                g.state.objects[colossus as usize].zone,
                if choice == ZoneReplacementKind::Exile {
                    Zone::Exile
                } else {
                    Zone::Library
                }
            );
            assert_eq!(g.counter_count(colossus, CounterKind::Finality), 0);
        }
    }
}

#[test]
fn tmt_sneak_sewer_filter_uses_graveyard_characteristic_defining_power_and_not_battlefield_lords() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let leo = named(&mut g, p, "Leonardo, Sewer Samurai", Zone::Battlefield);
        let permission = g.object_ref(leo);
        let goyf = named(&mut g, p, "Tarmogoyf", Zone::Graveyard);
        assert_eq!(
            g.stats(goyf),
            Stats {
                power: 1,
                toughness: 2
            }
        );
        assert!(g.legal_actions().contains(&grave_spell(
            goyf,
            permission,
            CastMethod::Normal,
            None
        )));
        let opt = named(&mut g, p.other(), "Opt", Zone::Graveyard);
        assert_eq!(
            g.stats(goyf),
            Stats {
                power: 2,
                toughness: 3
            }
        );
        reject(
            &mut g,
            grave_spell(goyf, permission, CastMethod::Normal, None),
        );
        g.move_object(opt, Zone::Exile);
        checked(
            &mut g,
            grave_spell(goyf, permission, CastMethod::Normal, None),
        );
        finish_grave_spell(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(goyf, CounterKind::Finality), 1);
    }
}

#[test]
fn tmt_sneak_muldrotha_can_pay_warp_as_a_separate_alternative_without_finality() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let m = named(&mut g, p, "Muldrotha, the Gravetide", Zone::Battlefield);
        let knight = named(&mut g, p, "Knight Luminary", Zone::Graveyard);
        let action = Action::AnnounceGraveyardSpell {
            card: knight,
            permission: g.object_ref(m),
            permanent_type: Some(PermanentType::Creature),
            method: CastMethod::Warp,
            attacker: None,
            flash_surcharge: false,
        };
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
        checked(&mut g, action.clone());
        finish_grave_spell(&mut g);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().method,
            CastMethod::Warp
        );
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().payment,
            [1, 0, 0, 0, 0, 1]
        );
        settle(&mut g);
        assert!(g.on_battlefield(knight));
        assert_eq!(g.counter_count(knight, CounterKind::Finality), 0);
        assert_eq!(g.observe(p).graveyard_type_uses.len(), 1);
        g.move_object(knight, Zone::Graveyard);
        g.state.players[p.index()].mana = [30; 6];
        reject(&mut g, action);
    }
}
