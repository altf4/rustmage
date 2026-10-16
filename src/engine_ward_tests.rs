use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let after = g.snapshot();
    assert_eq!(after, fork.snapshot());
    g.restore(&saved).unwrap();
    act(g, a);
    assert_eq!(after, g.snapshot());
}
fn reject(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(saved, g.snapshot());
}
fn order(g: &mut Game) {
    g.prepare_triggers();
    while let Some(a) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PutTrigger { .. }))
    {
        replay(g, a);
    }
}
fn ward_spell(g: &mut Game) -> (CardId, CardId) {
    let target = named(g, Player::P1, "Hulking Raptor", Zone::Battlefield);
    let spell = named(g, Player::P0, "Shock", Zone::Hand);
    g.state.players[0].mana = [10; 6];
    cast(g, spell, Some(Target::Card(target)));
    assert_eq!(g.state.stack.len(), 2);
    g.state.players[0].mana = [0; 6];
    (spell, target)
}
fn counter_spell(g: &mut Game, name: &str) -> (CardId, CardId) {
    let target = named(g, Player::P0, "Grizzly Bears", Zone::Hand);
    g.state.players[0].mana = [10; 6];
    cast(g, target, None);
    act(g, Action::Pass);
    let counter = named(g, Player::P1, name, Zone::Hand);
    g.state.players[1].mana = [10; 6];
    cast(g, counter, Some(Target::Spell(target)));
    g.state.players[0].mana = [0; 6];
    (target, counter)
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayResolutionMana { .. }))
        .unwrap();
    replay(g, a);
}

#[test]
fn ward_is_a_triggered_payment_not_a_targeting_restriction_or_casting_cost() {
    let mut g = scenario();
    let (spell, target) = ward_spell(&mut g);
    let victim = g.state.stack[0].id;
    let ward = &g.state.stack[1];
    assert_eq!(ward.card, target);
    assert_eq!(
        ward.related_stack,
        Some(StackLink {
            id: victim,
            controller: Player::P0
        })
    );
    assert!(ward.targets.is_empty());
    pass_pair(&mut g);
    assert_eq!(g.observe(Player::P0).decision, "resolution_mana");
    assert_eq!(g.actor(), Some(Player::P0));
    assert_eq!(
        g.observe(Player::P1)
            .resolution_payment
            .unwrap()
            .cost
            .generic,
        2
    );
    assert!(g.observe(Player::P1).actions.is_empty());
    reject(&mut g, Action::Pass);
    reject(&mut g, Action::PayResolutionMana { payment: [0; 6] });
    replay(&mut g, Action::DeclineResolutionMana);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(g.object_view(target).damage, 0);
    assert!(g.state.stack.is_empty());
}

#[test]
fn ward_payment_can_tap_lands_mid_resolution_and_resume_original_spell() {
    let mut g = scenario();
    let (spell, target) = ward_spell(&mut g);
    let a = named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    let b = named(&mut g, Player::P0, "Island", Zone::Battlefield);
    pass_pair(&mut g);
    replay(&mut g, Action::TapMana { card: a });
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::PayResolutionMana { .. }))
    );
    replay(&mut g, Action::TapMana { card: b });
    pay(&mut g);
    assert_eq!(g.state.stack.len(), 1);
    assert_eq!(g.object_view(spell).zone, Zone::Stack);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    pass_pair(&mut g);
    assert_eq!(g.object_view(target).damage, 2);
}

#[test]
fn player_can_float_mana_then_decline_even_when_full_payment_is_impossible() {
    let mut g = scenario();
    let (spell, _) = ward_spell(&mut g);
    let land = named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    pass_pair(&mut g);
    replay(&mut g, Action::TapMana { card: land });
    replay(&mut g, Action::DeclineResolutionMana);
    assert_eq!(g.state.players[0].mana[4], 1);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
}

#[test]
fn own_targets_and_non_targeted_effects_do_not_trigger_ward() {
    let mut g = scenario();
    let own = named(&mut g, Player::P0, "Toadstool Admirer", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Giant Growth", Zone::Hand);
    g.state.players[0].mana = [10; 6];
    cast(&mut g, spell, Some(Target::Card(own)));
    assert_eq!(g.state.stack.len(), 1);
    pass_pair(&mut g);
    let opponent = named(&mut g, Player::P1, "Toadstool Admirer", Zone::Battlefield);
    let source = named(&mut g, Player::P0, "Ultimecia, Temporal Threat", Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    order(&mut g);
    pass_pair(&mut g);
    assert!(g.object_view(opponent).tapped);
    assert!(g.state.pending_triggers.is_empty());
    assert!(g.state.stack.is_empty());
}

#[test]
fn ward_does_not_trigger_when_its_creature_spell_is_countered_on_the_stack() {
    let mut g = scenario();
    let target = named(&mut g, Player::P0, "Hulking Raptor", Zone::Hand);
    g.state.players[0].mana = [10; 6];
    cast(&mut g, target, None);
    act(&mut g, Action::Pass);
    let counter = named(&mut g, Player::P1, "Essence Scatter", Zone::Hand);
    g.state.players[1].mana = [10; 6];
    cast(&mut g, counter, Some(Target::Spell(target)));
    pass_pair(&mut g);
    assert_eq!(g.object_view(target).zone, Zone::Graveyard);
    assert!(g.state.stack.is_empty());
}

#[test]
fn ward_survives_source_departure_and_counters_associated_activated_ability() {
    let mut g = scenario();
    let target = named(&mut g, Player::P1, "Toadstool Admirer", Zone::Battlefield);
    let source = named(&mut g, Player::P0, "Fanatical Firebrand", Zone::Battlefield);
    let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,target:Some(Target::Card(t)),..} if *card==source && *t==target)).unwrap();
    replay(&mut g, action);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    let victim = g.state.stack[0].id;
    g.move_object(target, Zone::Exile);
    assert_eq!(g.state.stack[1].related_stack.unwrap().id, victim);
    pass_pair(&mut g);
    replay(&mut g, Action::DeclineResolutionMana);
    assert!(g.state.stack.is_empty());
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert_eq!(g.object_view(target).zone, Zone::Exile);
}

#[test]
fn triggered_ability_targeting_creates_an_independent_ward_batch() {
    let mut g = scenario();
    let target = named(&mut g, Player::P1, "Toadstool Admirer", Zone::Battlefield);
    let archer = named(&mut g, Player::P0, "Skeleton Archer", Zone::Hand);
    g.move_object(archer, Zone::Battlefield);
    order(&mut g);
    assert_eq!(g.observe(Player::P0).decision, "targets");
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    assert_eq!(g.state.stack.len(), 2);
    assert!(matches!(
        g.state.stack[0].kind,
        StackKind::TriggeredAbility { .. }
    ));
    pass_pair(&mut g);
    replay(&mut g, Action::DeclineResolutionMana);
    assert_eq!(g.object_view(target).damage, 0);
    assert_eq!(g.object_view(archer).zone, Zone::Battlefield);
}

#[test]
fn same_permanent_in_multiple_target_groups_triggers_each_ward_once() {
    let mut g = scenario();
    let target = named(&mut g, Player::P1, "Toadstool Admirer", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Giant Growth", Zone::Hand);
    let mut item = g.cast_item(Player::P0, spell, [0; 6]);
    // Synthetic target groups test identity deduplication independently of any
    // claim that Giant Growth actually has two target occurrences.
    let first = g.bind_target(Target::Card(target));
    g.target_becomes_bound(&item, first);
    item.targets.push(first);
    let mut second = first;
    second.group = 1;
    g.target_becomes_bound(&item, second);
    item.targets.push(second);
    assert_eq!(g.state.pending_triggers.len(), 1);
    assert_eq!(
        g.state.pending_triggers[0].related_stack.unwrap().id,
        item.id
    );
}

#[test]
fn ward_queues_before_payment_and_retains_departure_during_costs() {
    let mut g = scenario();
    let target = named(&mut g, Player::P1, "Toadstool Admirer", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Shock", Zone::Hand);
    named(&mut g, Player::P0, "Mountain", Zone::Battlefield);
    replay(&mut g, Action::AnnounceCast { card: spell });
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    assert_eq!(g.observe(Player::P0).decision, "pay_costs");
    assert_eq!(g.state.pending_triggers.len(), 1);
    let old = g.object_ref(target);
    g.move_object(target, Zone::Exile);
    g.move_object(target, Zone::Battlefield);
    let mana = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::TapMana { .. }))
        .unwrap();
    replay(&mut g, mana);
    let payment = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(&mut g, payment);
    assert_eq!(g.state.stack[1].object, old);
    pass_pair(&mut g);
    replay(&mut g, Action::DeclineResolutionMana);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
}

#[test]
fn counter_unless_paid_preserves_spell_or_counters_it_atomically() {
    for paid in [false, true] {
        let mut g = scenario();
        let (target, counter) = counter_spell(&mut g, "It'll Quench Ya!");
        pass_pair(&mut g);
        let view = g.observe(Player::P0).resolution_payment.unwrap();
        assert_eq!(view.cost.generic, 2);
        if paid {
            g.state.players[0].mana = [0, 0, 0, 0, 2, 0];
            pay(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Stack);
            pass_pair(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        } else {
            replay(&mut g, Action::DeclineResolutionMana);
            assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        }
        assert_eq!(g.object_view(counter).zone, Zone::Graveyard);
    }
}

#[test]
fn no_more_lies_exiles_only_when_its_payment_is_declined() {
    for paid in [false, true] {
        let mut g = scenario();
        let (target, _) = counter_spell(&mut g, "No More Lies");
        pass_pair(&mut g);
        if paid {
            g.state.players[0].mana = [0, 0, 0, 0, 3, 0];
            pay(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Stack);
        } else {
            replay(&mut g, Action::DeclineResolutionMana);
            assert_eq!(g.object_view(target).zone, Zone::Exile);
        }
    }
}

#[test]
fn paid_conditional_surveil_belongs_to_counter_controller_and_waits_for_payment() {
    let mut g = scenario();
    let (target, counter) = counter_spell(&mut g, "Don't Make a Sound");
    pass_pair(&mut g);
    g.state.players[0].mana = [0, 0, 0, 0, 2, 0];
    pay(&mut g);
    assert_eq!(g.actor(), Some(Player::P1));
    assert_eq!(g.observe(Player::P1).decision, "library_choice");
    assert!(g.observe(Player::P0).library_choice.is_none());
    assert_eq!(g.object_view(counter).zone, Zone::Stack);
    assert_eq!(g.object_view(target).zone, Zone::Stack);
    while g.observe(Player::P1).library_choice.is_some() {
        let card = g.observe(Player::P1).library_choice.unwrap().remaining[0].id;
        replay(
            &mut g,
            Action::ArrangeLibrary {
                card,
                destination: LibraryDestination::Top,
            },
        );
    }
    assert_eq!(g.object_view(counter).zone, Zone::Graveyard);
    let mut g = scenario();
    counter_spell(&mut g, "Don't Make a Sound");
    pass_pair(&mut g);
    replay(&mut g, Action::DeclineResolutionMana);
    assert!(g.state.resolving.is_none());
}

#[test]
fn paid_conditional_lander_is_created_only_for_successful_payment() {
    for paid in [false, true] {
        let mut g = scenario();
        counter_spell(&mut g, "Divert Disaster");
        pass_pair(&mut g);
        if paid {
            g.state.players[0].mana = [0, 0, 0, 0, 2, 0];
            pay(&mut g);
        } else {
            replay(&mut g, Action::DeclineResolutionMana);
        }
        let tokens: Vec<_> = g
            .observe(Player::P1)
            .objects
            .into_iter()
            .filter(|o| {
                o.zone == Zone::Battlefield
                    && o.card.token_info().is_some_and(|t| t.subtypes == "Lander")
            })
            .collect();
        assert_eq!(tokens.len(), usize::from(paid));
        if paid {
            assert_eq!(tokens[0].owner, Player::P1);
        }
    }
}

#[test]
fn restricted_counter_targets_respect_spell_types_and_exclude_abilities() {
    for (counter, card, wanted) in [
        ("Negate", "Grizzly Bears", false),
        ("Negate", "Giant Growth", true),
        ("Essence Scatter", "Grizzly Bears", true),
        ("Essence Scatter", "Giant Growth", false),
        ("Spell Pierce", "Grizzly Bears", false),
        ("Spell Pierce", "Giant Growth", true),
    ] {
        let mut g = scenario();
        let target = named(&mut g, Player::P0, card, Zone::Hand);
        g.state.players[0].mana = [10; 6];
        let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        cast(
            &mut g,
            target,
            if card == "Giant Growth" {
                Some(Target::Card(bear))
            } else {
                None
            },
        );
        act(&mut g, Action::Pass);
        let source = named(&mut g, Player::P1, counter, Zone::Hand);
        g.state.players[1].mana = [10; 6];
        assert_eq!(g.legal_actions().iter().any(|a|matches!(a,Action::Cast{card,target:Some(Target::Spell(t)),..} if *card==source && *t==target)),wanted,"{counter} {card}");
    }
}

#[test]
fn counters_fizzle_without_payment_when_target_has_left() {
    let mut g = scenario();
    let (target, counter) = counter_spell(&mut g, "It'll Quench Ya!");
    g.move_object(target, Zone::Graveyard);
    pass_pair(&mut g);
    assert!(g.state.resolving.is_none());
    assert_eq!(g.object_view(counter).zone, Zone::Graveyard);
    assert!(g.observe(Player::P0).resolution_payment.is_none());
}

#[test]
fn absent_related_stack_item_does_not_alias_a_new_spell_incarnation() {
    let mut g = scenario();
    let (spell, _) = ward_spell(&mut g);
    pass_pair(&mut g);
    let old = g
        .observe(Player::P0)
        .resolution_payment
        .unwrap()
        .victim
        .unwrap();
    // Synthetic intervening zone movement checks the owned association by stack
    // identity, rather than by a reusable physical card slot.
    g.move_object(spell, Zone::Graveyard);
    g.move_object(spell, Zone::Hand);
    let item = g.cast_item(Player::P0, spell, [0; 6]);
    assert_ne!(old, item.id);
    g.state.stack.push(item);
    replay(&mut g, Action::DeclineResolutionMana);
    assert_eq!(g.object_view(spell).zone, Zone::Stack);
    assert_eq!(g.state.stack.len(), 1);
}

#[test]
fn colored_and_generic_allocations_are_exact() {
    static CODE: &[Instruction] = &[Instruction::CounterUnlessPay {
        cost: ManaCost {
            colored: [1, 0, 0, 0, 0, 1],
            generic: 1,
        },
        subject: CounterSubject::Related,
        exile: false,
        unpaid_skip: 0,
    }];
    let mut g = scenario();
    ward_spell(&mut g);
    pass_pair(&mut g);
    let r = g.state.resolving.as_mut().unwrap();
    r.program = CODE;
    r.pc = 0;
    r.mana_payment = None;
    g.run_program();
    g.state.players[0].mana = [1, 1, 0, 0, 0, 1];
    reject(
        &mut g,
        Action::PayResolutionMana {
            payment: [1, 1, 0, 0, 0, 0],
        },
    );
    pay(&mut g);
    assert_eq!(g.state.players[0].mana, [0; 6]);
}

#[test]
fn mana_sacrifice_linked_aura_entry_restores_resolution_payment() {
    let mut g = scenario();
    ward_spell(&mut g);
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let aura = named(&mut g, Player::P0, "Frantic Strength", Zone::Exile);
    let token = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Treasure"))
        .unwrap();
    let treasure = spawn(&mut g, Player::P0, token, Zone::Battlefield);
    let land = named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    // Synthetic link: exercises the existing return vocabulary when a mana
    // activation's sacrifice cost opens an Aura entry transaction.
    g.state.exile_links.push(ExileLink {
        source: g.object_ref(treasure),
        exiled: g.object_ref(aura),
        return_zone: Zone::Battlefield,
    });
    pass_pair(&mut g);
    let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,production,..} if *card==treasure && production[4]==1)).unwrap();
    replay(&mut g, action);
    assert_eq!(g.observe(Player::P0).decision, "aura_entry");
    assert!(g.state.resolving.as_ref().unwrap().mana_payment.is_some());
    let object = g.object_ref(bear);
    replay(&mut g, Action::ChooseAuraAttachment { object });
    assert_eq!(g.observe(Player::P0).decision, "resolution_mana");
    assert_eq!(g.state.players[0].mana[4], 1);
    replay(&mut g, Action::TapMana { card: land });
    pay(&mut g);
    assert!(g.state.resolving.is_none());
    assert_eq!(g.object_view(aura).zone, Zone::Battlefield);
}

#[test]
fn wrong_actor_and_overpayment_are_rejected_without_mutation() {
    let mut g = scenario();
    ward_spell(&mut g);
    pass_pair(&mut g);
    g.state.players[0].mana = [3, 0, 0, 0, 0, 0];
    reject(
        &mut g,
        Action::PayResolutionMana {
            payment: [3, 0, 0, 0, 0, 0],
        },
    );
    let saved = g.snapshot();
    assert_eq!(
        g.apply(Player::P1, Action::DeclineResolutionMana),
        Err(Error::WrongActor)
    );
    assert_eq!(saved, g.snapshot());
    pay(&mut g);
    assert_eq!(g.state.players[0].mana[0], 1);
}

#[test]
fn resolution_payment_memory_and_context_are_owned_by_snapshots() {
    let mut g = scenario();
    ward_spell(&mut g);
    pass_pair(&mut g);
    let bytes = g.state_bytes();
    let original = g.snapshot();
    let mut fork = g.clone();
    fork.state.resolving.as_mut().unwrap().mana_payment = None;
    assert_eq!(
        bytes - fork.state_bytes(),
        std::mem::size_of::<resolution_mana::ResolutionPayment>()
    );
    assert_eq!(g.snapshot(), original);
    replay(&mut g, Action::DeclineResolutionMana);
}

fn nonmana_ward_spell(g: &mut Game, payer: Player, name: &str) -> (CardId, CardId) {
    g.state.active = payer;
    g.state.priority = payer;
    let target = named(g, payer.other(), name, Zone::Battlefield);
    let spell = named(g, payer, "Shock", Zone::Hand);
    g.state.players[payer.index()].mana = [10; 6];
    cast(g, spell, Some(Target::Card(target)));
    assert_eq!(g.state.stack.len(), 2);
    g.state.players[payer.index()].mana = [0; 6];
    (spell, target)
}

#[test]
fn nonmana_ward_costs_are_owned_full_payments_with_private_discard_and_replay() {
    for payer in [Player::P0, Player::P1] {
        for name in ["Spectral Snatcher", "Sunset Saboteur", "Vein Ripper"] {
            for pay in [false, true] {
                let mut g = scenario();
                let resource = named(
                    &mut g,
                    payer,
                    "Grizzly Bears",
                    if name == "Vein Ripper" {
                        Zone::Battlefield
                    } else {
                        Zone::Hand
                    },
                );
                let wrong = named(&mut g, payer.other(), "Grizzly Bears", Zone::Hand);
                let (spell, target) = nonmana_ward_spell(&mut g, payer, name);
                pass_pair(&mut g);
                assert_eq!(g.actor(), Some(payer));
                assert!(matches!(g.state.pending, Pending::EffectChoice { .. }));
                assert!(g.observe(payer.other()).actions.is_empty());
                if name != "Vein Ripper" {
                    assert!(g.observe(payer.other()).effect_choice.is_none());
                    reject(&mut g, Action::Discard { card: wrong });
                    let object = g.object_ref(resource);
                    reject(&mut g, Action::Sacrifice { object });
                } else {
                    reject(&mut g, Action::Discard { card: wrong });
                }
                reject(&mut g, Action::Pass);
                if pay {
                    let action = if name == "Vein Ripper" {
                        Action::Sacrifice {
                            object: g.object_ref(resource),
                        }
                    } else {
                        Action::Discard { card: resource }
                    };
                    replay(&mut g, action);
                    assert_eq!(g.object_view(resource).zone, Zone::Graveyard);
                    assert_eq!(g.object_view(spell).zone, Zone::Stack);
                    assert_eq!(g.object_view(target).damage, 0);
                    if name == "Vein Ripper" {
                        // Payment deaths queue the complete printed drain trigger.
                        assert!(
                            !g.state.pending_triggers.is_empty()
                                || !g.state.ordering_triggers.is_empty()
                                || g.state.stack.len() > 1
                                || g.state.announcing.is_some()
                        );
                    } else {
                        pass_pair(&mut g);
                        assert_eq!(
                            g.object_view(target).damage,
                            if name == "Sunset Saboteur" { 0 } else { 2 }
                        );
                    }
                } else {
                    replay(&mut g, Action::DeclineEffectPayment);
                    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
                    assert_eq!(
                        g.object_view(resource).zone,
                        if name == "Vein Ripper" {
                            Zone::Battlefield
                        } else {
                            Zone::Hand
                        }
                    );
                    assert_eq!(g.object_view(target).damage, 0);
                }
            }
        }
    }
}

#[test]
fn nonmana_ward_without_resources_counters_without_offering_partial_payment() {
    for payer in [Player::P0, Player::P1] {
        for name in ["Spectral Snatcher", "Sunset Saboteur", "Vein Ripper"] {
            let mut g = scenario();
            named(&mut g, payer, "Forest", Zone::Battlefield);
            let (spell, target) = nonmana_ward_spell(&mut g, payer, name);
            pass_pair(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(g.object_view(target).damage, 0);
            assert!(g.state.resolving.is_none());
        }
    }
}

#[test]
fn life_ward_requires_affordability_and_choice_charges_exact_payer_with_replay() {
    for payer in [Player::P0, Player::P1] {
        for life in [6, 7, 20] {
            for pay in [false, true] {
                let mut g = scenario();
                g.state.players[payer.index()].life = life;
                let (spell, target) = nonmana_ward_spell(&mut g, payer, "Sire of Seven Deaths");
                pass_pair(&mut g);
                if life < 7 {
                    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
                    assert_eq!(g.state.players[payer.index()].life, life);
                    continue;
                }
                assert_eq!(g.actor(), Some(payer));
                assert!(g.observe(payer.other()).actions.is_empty());
                assert_eq!(g.state.players[payer.index()].life, life);
                reject(&mut g, Action::DeclineResolutionMana);
                replay(&mut g, Action::ChooseOptional { accept: pay });
                assert_eq!(
                    g.state.players[payer.index()].life,
                    life - if pay { 7 } else { 0 }
                );
                assert_eq!(g.state.players[payer.other().index()].life, 20);
                assert_eq!(
                    g.state.players[payer.index()].life_lost_this_turn,
                    if pay { 7 } else { 0 }
                );
                assert_eq!(
                    g.object_view(spell).zone,
                    if pay { Zone::Stack } else { Zone::Graveyard }
                );
                assert_eq!(g.object_view(target).damage, 0);
                if pay && life == 7 {
                    assert!(g.outcome().is_some());
                }
            }
        }
    }
}

#[test]
fn nonmana_ward_counters_exact_ability_after_source_departure_or_reentry() {
    for name in ["Spectral Snatcher", "Sire of Seven Deaths", "Vein Ripper"] {
        let mut g = scenario();
        let target = named(&mut g, Player::P1, name, Zone::Battlefield);
        let source = named(&mut g, Player::P0, "Greenbelt Guardian", Zone::Battlefield);
        g.state.players[0].mana = [10; 6];
        named(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
        named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate {card,target:Some(Target::Card(t)),..} if *card==source && *t==target)).unwrap();
        replay(&mut g, action);
        g.move_object(source, Zone::Exile);
        let victim = g.state.stack[0].id;
        let original = g.object_ref(target);
        g.move_object(target, Zone::Exile);
        g.move_object(target, Zone::Battlefield);
        assert_ne!(original, g.object_ref(target));
        assert_eq!(g.state.stack[1].related_stack.unwrap().id, victim);
        pass_pair(&mut g);
        let decline = if name == "Sire of Seven Deaths" {
            Action::ChooseOptional { accept: false }
        } else {
            Action::DeclineEffectPayment
        };
        replay(&mut g, decline);
        assert!(g.state.stack.is_empty());
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        assert_eq!(g.object_view(target).damage, 0);
    }
}

#[test]
fn vein_ripper_observes_its_own_simultaneous_death_and_each_other_creature() {
    let mut g = scenario();
    let ripper = named(&mut g, Player::P0, "Vein Ripper", Zone::Battlefield);
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let enemy = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let sources = g.death_trigger_sources();
    let events = g.move_objects_raw(&[ripper, bear, enemy], Zone::Graveyard);
    g.finish_zone_events(&events, &sources);
    assert_eq!(
        g.state
            .pending_triggers
            .iter()
            .filter(|t| t.card == ripper)
            .count(),
        3
    );
    for _ in 0..40 {
        g.prepare_triggers();
        if g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
            && g.state.announcing.is_none()
        {
            break;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::PutTrigger { .. }
                        | Action::ChooseTarget {
                            target: Target::Player(Player::P1)
                        }
                        | Action::FinishTargets
                        | Action::Pass
                )
            })
            .unwrap();
        replay(&mut g, a);
    }
    assert!(g.state.stack.is_empty());
    assert_eq!(g.state.players[0].life, 26);
    assert_eq!(g.state.players[1].life, 14);
}

#[test]
fn spectral_snatcher_swampcycling_is_an_independent_hand_ability() {
    let mut g = scenario();
    let card = named(&mut g, Player::P0, "Spectral Snatcher", Zone::Hand);
    let swamp = named(&mut g, Player::P0, "Swamp", Zone::Library);
    let island = named(&mut g, Player::P0, "Island", Zone::Library);
    g.state.players[0].mana = [2, 0, 0, 0, 0, 0];
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate{card:c,..}if *c==card))
        .unwrap();
    replay(&mut g, action);
    assert_eq!(g.object_view(card).zone, Zone::Graveyard);
    assert_eq!(g.state.stack.len(), 1);
    pass_pair(&mut g);
    assert_eq!(
        g.observe(Player::P0).search_choice.unwrap().filter,
        crate::SearchFilter::Subtype("Swamp")
    );
    reject(&mut g, Action::ChooseSearch { card: island });
    replay(&mut g, Action::ChooseSearch { card: swamp });
    replay(&mut g, Action::FinishSearch);
    assert_eq!(g.object_view(swamp).zone, Zone::Hand);
}

#[test]
fn nonmana_counter_association_cannot_alias_recast_card_and_life_capacity_is_atomic() {
    for name in ["Spectral Snatcher", "Sire of Seven Deaths"] {
        let mut g = scenario();
        named(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
        let (spell, _) = nonmana_ward_spell(&mut g, Player::P0, name);
        pass_pair(&mut g);
        let old = g
            .state
            .resolving
            .as_ref()
            .unwrap()
            .item
            .related_stack
            .unwrap()
            .id;
        // Synthetic intervening movement isolates stack identity from the reused slot.
        g.move_object(spell, Zone::Graveyard);
        g.move_object(spell, Zone::Hand);
        let item = g.cast_item(Player::P0, spell, [0; 6]);
        assert_ne!(item.id, old);
        g.state.stack.push(item);
        let decline = if name == "Sire of Seven Deaths" {
            Action::ChooseOptional { accept: false }
        } else {
            Action::DeclineEffectPayment
        };
        replay(&mut g, decline);
        assert_eq!(g.object_view(spell).zone, Zone::Stack);
        assert_eq!(g.state.stack.len(), 1);
    }
    let mut g = scenario();
    nonmana_ward_spell(&mut g, Player::P0, "Sire of Seven Deaths");
    pass_pair(&mut g);
    g.state.players[0].life_lost_this_turn = u128::MAX;
    let saved = g.snapshot();
    let revision = g.revision();
    for _ in 0..2 {
        assert!(matches!(
            g.apply(Player::P0, Action::ChooseOptional { accept: true }),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), revision);
    }
    let mut fork = g.clone();
    assert!(matches!(
        fork.apply(Player::P0, Action::ChooseOptional { accept: true }),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(fork.snapshot(), saved);
    g.restore(&saved).unwrap();
    replay(&mut g, Action::ChooseOptional { accept: false });
}

#[test]
fn sunset_attack_counter_requires_an_opposing_creature_and_is_mandatory() {
    for opponent in [false, true] {
        let mut g = scenario();
        let source = named(&mut g, Player::P0, "Sunset Saboteur", Zone::Battlefield);
        let own = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let enemy = opponent.then(|| named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield));
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        replay(&mut g, Action::Attack { card: source });
        replay(&mut g, Action::FinishAttackers);
        if let Some(enemy) = enemy {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            reject(&mut g, Action::FinishTargets);
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            pass_pair(&mut g);
            assert_eq!(g.object_view(enemy).counters.plus_one, 1);
        } else {
            assert!(g.state.stack.is_empty());
            assert_eq!(g.object_view(own).counters.plus_one, 0);
        }
    }
}
