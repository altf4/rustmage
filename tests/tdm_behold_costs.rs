//! Shared additional/alternative and target/scalar costs. No TDM definitions.
mod support;
use rustmage as _;
use rustmage_core::CardName;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const EQUIP: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 4,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        program: &[Instruction::AttachSource],
        sorcery: true,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        activation_target_color_discounts: &[(0, 1)],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const DRAGON_RULE: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Creature,
    predicate: &TargetPredicate::Subtype("Dragon"),
};
const PREY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::Destroy],
    ),
    extras: &CardExtras {
        target_cost_increase: Some(&TargetSpellIncrease {
            rule: DRAGON_RULE,
            generic: 2,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const X_LIFE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            colored: [0, 0, 1, 0, 0, 0],
            tap: true,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ActivationX,
            Instruction::ActivationLifeX,
            Instruction::GainLifeScaled(PlayerSet::Controller, AmountValue::ChosenX),
        ],
        ..EQUIP.activated[0]
    }],
    ..CardRules::EMPTY
};
const X_TARGETS: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Graveyard,
        cost: ActivationCost {
            colored: [0, 0, 2, 0, 0, 0],
            exile_self: true,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: 0,
        }),
        program: &[
            Instruction::ActivationX,
            Instruction::TargetsExactlyX,
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
        ..EQUIP.activated[0]
    }],
    ..CardRules::EMPTY
};
fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn target(g: &mut Game, card: CardId) {
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(card),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn color(g: &mut Game, card: CardId, mask: u8) {
    // Existing external color layer, not a replacement of the immutable target.
    let timestamp = g.next_role_timestamp();
    let object = g.object_ref(card);
    g.state
        .roles
        .as_mut()
        .unwrap()
        .animations
        .push(AnimationView {
            object,
            timestamp,
            artifact: false,
            land: false,
            preserve_card_types: true,
            food_only: false,
            artifact_noncreature: false,
            enchantment_noncreature: false,
            colors: Some(mask),
            creature_types: None,
            all_creature_types: false,
            base: None,
            keywords: Keywords::NONE,
            expires_turn: None,
        });
    assert_eq!(g.object_view(card).colors, mask);
}
fn choose_life_row(g: &mut Game, x: u32) {
    let index = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == CastCostClause::Life(x) && !row.paid)
        .unwrap();
    act(
        g,
        Action::ChooseCastCost {
            clause: index as u16,
        },
    );
}

#[test]
fn target_color_discount_uses_all_effective_colors_and_floor_zero_for_both_seats() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &EQUIP)]);
    for p in [Player::P0, Player::P1] {
        for mask in [0u8, 1, 3, 7, 15, 31] {
            let mut g = scenario(p);
            let equipment = put(&mut g, p, "Short Bow", Zone::Battlefield);
            let creature = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            color(&mut g, creature, mask);
            let generic = 4u32.saturating_sub(mask.count_ones());
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, generic];
            act(&mut g, activate(equipment));
            target(&mut g, creature);
            assert_eq!(g.observe(p).payment_request.unwrap().generic, generic);
            pay(&mut g);
            settle(&mut g);
            assert_eq!(
                g.state.objects[equipment as usize].attached_to,
                Some(g.object_ref(creature))
            );
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn target_color_discount_filters_real_targets_and_direct_activation_costs() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &EQUIP)]);
    let mut g = scenario(Player::P0);
    let equipment = put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
    let cheap = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    let expensive = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    let opponent = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    color(&mut g, cheap, 31);
    color(&mut g, expensive, 0);
    color(&mut g, opponent, 31);
    g.state.players[0].mana = [0; 6];
    assert!(g.legal_actions().iter().any(|a| matches!(a, Action::Activate { card, target: Some(Target::Card(t)), payment, .. } if *card == equipment && *t == cheap && *payment == [0;6])));
    act(&mut g, activate(equipment));
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(expensive),
        },
    );
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(opponent),
        },
    );
    target(&mut g, cheap);
    pay(&mut g);
    settle(&mut g);
    assert_eq!(
        g.state.objects[equipment as usize].attached_to,
        Some(g.object_ref(cheap))
    );
}

#[test]
fn target_color_total_locks_before_payment_and_does_not_follow_a_new_target_incarnation() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &EQUIP)]);
    for blink in [false, true] {
        let mut g = scenario(Player::P0);
        let equipment = put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
        let creature = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
        color(&mut g, creature, 31);
        g.state.players[0].mana = [0; 6];
        act(&mut g, activate(equipment));
        target(&mut g, creature);
        assert_eq!(g.observe(Player::P0).payment_request.unwrap().generic, 0);
        if blink {
            g.move_object(creature, Zone::Hand);
            g.move_object(creature, Zone::Battlefield);
        } else {
            color(&mut g, creature, 0);
        }
        pay(&mut g);
        settle(&mut g);
        assert_eq!(
            g.state.objects[equipment as usize].attached_to,
            (!blink).then(|| g.object_ref(creature))
        );
    }
}

#[test]
fn target_color_choice_is_owned_indexed_private_and_restorable_after_parent_disposal() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &EQUIP)]);
    let mut parent = scenario(Player::P0);
    let equipment = put(&mut parent, Player::P0, "Short Bow", Zone::Battlefield);
    let creature = put(&mut parent, Player::P0, "Hill Giant", Zone::Battlefield);
    color(&mut parent, creature, 31);
    parent.state.players[0].mana = [0; 6];
    act(&mut parent, activate(equipment));
    let action = Action::ChooseTarget {
        target: Target::Card(creature),
    };
    let saved = parent.snapshot();
    let revision = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut sibling = parent.clone();
    drop(parent);
    assert!(raw.observe(Player::P1).actions.is_empty());
    assert_eq!(
        raw.apply(Player::P1, action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(raw.snapshot(), saved);
    act(&mut raw, action.clone());
    indexed.apply_index(Player::P0, revision, index).unwrap();
    if raw.legal_actions().contains(&Action::FinishTargets) {
        act(&mut raw, Action::FinishTargets);
        act(&mut indexed, Action::FinishTargets);
    }
    pay(&mut raw);
    pay(&mut indexed);
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(sibling.snapshot(), saved);
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(Player::P0, revision, index),
        Err(Error::StaleRevision)
    );
    target(&mut indexed, creature);
    pay(&mut indexed);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    sibling.move_object(creature, Zone::Hand);
    reject(&mut sibling, action);
}

#[test]
fn target_subtype_surcharge_can_avoid_dragons_but_requires_extra_mana_to_choose_one() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Murder").unwrap(), &PREY)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let dragon = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        let bear = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, "Murder", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
        act(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(dragon),
            },
        );
        target(&mut g, bear);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[dragon as usize].zone, Zone::Battlefield);
    }
}

#[test]
fn target_subtype_surcharge_is_unavoidable_with_only_dragons_and_locks_before_payment() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Murder").unwrap(), &PREY)]);
    let mut g = scenario(Player::P0);
    let dragon = put(&mut g, Player::P1, "Shivan Dragon", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Murder", Zone::Hand);
    g.state.players[0].mana = [0, 0, 2, 0, 0, 1];
    reject(&mut g, Action::AnnounceCast { card: spell });
    g.state.players[0].mana[5] = 3;
    act(&mut g, Action::AnnounceCast { card: spell });
    target(&mut g, dragon);
    assert_eq!(g.observe(Player::P0).payment_request.unwrap().generic, 3);
    g.apply_temporary_copy(
        g.object_ref(dragon),
        CopyRecipe::new(Card::GrizzlyBears),
        CopyDeadline::EndOfTurn,
    );
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert_eq!(g.state.objects[dragon as usize].zone, Zone::Graveyard);
}

#[test]
fn activation_x_life_uses_one_resource_world_and_real_tap_life_and_mana_payment() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &X_LIFE)]);
    for p in [Player::P0, Player::P1] {
        for x in [0, 3] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            g.state.players[p.index()].life = 4;
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 10];
            act(&mut g, activate(source));
            assert_eq!(
                g.x_choice().unwrap().intervals,
                vec![XInterval {
                    minimum: 0,
                    maximum: 4
                }]
            );
            reject(&mut g, Action::ChooseX { value: 5 });
            act(&mut g, Action::ChooseX { value: x });
            pay(&mut g);
            choose_life_row(&mut g, x);
            assert_eq!(g.state.players[p.index()].life, 4 - i128::from(x));
            assert!(g.state.objects[source as usize].tapped);
            assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 10 - x]);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, 4);
        }
    }
}

#[test]
fn activation_x_life_preserves_sorcery_tap_and_source_control_legality() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &X_LIFE)]);
    for invalid in 0..4 {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        match invalid {
            0 => g.state.objects[source as usize].tapped = true,
            1 => g.state.objects[source as usize].summoning_sick = true,
            2 => g.state.objects[source as usize].controller = Player::P1,
            _ => g.state.step = Step::CombatDamage,
        }
        reject(&mut g, activate(source));
    }
}

#[test]
fn activation_exact_x_targets_caps_the_domain_pays_exile_and_handles_zero_and_distinctness() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &X_TARGETS)]);
    for p in [Player::P0, Player::P1] {
        for x in [0, 2] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
            let a = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let b = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            act(&mut g, activate(source));
            assert_eq!(
                g.x_choice().unwrap().intervals,
                vec![XInterval {
                    minimum: 0,
                    maximum: 2
                }]
            );
            reject(&mut g, Action::ChooseX { value: 3 });
            act(&mut g, Action::ChooseX { value: x });
            if x > 0 {
                reject(&mut g, Action::FinishTargets);
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(a),
                    },
                );
                reject(&mut g, Action::FinishTargets);
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(a),
                    },
                );
                target(&mut g, b);
            }
            pay(&mut g);
            assert_eq!(g.state.objects[source as usize].zone, Zone::Exile);
            settle(&mut g);
            assert_eq!(g.counter_count(a, CounterKind::PlusOne), u64::from(x > 0));
            assert_eq!(g.counter_count(b, CounterKind::PlusOne), u64::from(x > 0));
        }
    }
}

#[test]
fn target_subtype_increase_precedes_harmonize_reduction_in_proposal_and_real_payment() {
    const HARMONIZE_PREY: CardRules = CardRules {
        extras: &CardExtras {
            harmonize_cost: Some(ManaCost {
                colored: [0, 0, 2, 0, 0, 0],
                generic: 1,
            }),
            ..*PREY.extras
        },
        ..PREY
    };
    let _scope = fixture_rules::activate(&[(Card::from_name("Murder").unwrap(), &HARMONIZE_PREY)]);
    let mut g = scenario(Player::P0);
    let helper = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let dragon = put(&mut g, Player::P1, "Shivan Dragon", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Murder", Zone::Graveyard);
    g.state.players[0].mana = [0, 0, 2, 0, 0, 0];
    // The Bear itself is a legal non-Dragon target, but choosing the Dragon
    // needs one generic after all increases and the two-power discount.
    act(&mut g, Action::AnnounceHarmonize { card: spell });
    let object = g.object_ref(helper);
    act(
        &mut g,
        Action::ChooseHarmonizer {
            object: Some(object),
        },
    );
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(dragon),
        },
    );
    g.state.players[0].mana[5] = 1;
    target(&mut g, dragon);
    assert_eq!(g.observe(Player::P0).payment_request.unwrap().generic, 1);
    pay(&mut g);
    let index = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == CastCostClause::Harmonize(object))
        .unwrap();
    act(
        &mut g,
        Action::ChooseCastCost {
            clause: index as u16,
        },
    );
    settle(&mut g);
    assert!(g.state.objects[helper as usize].tapped);
    assert_eq!(g.state.objects[spell as usize].zone, Zone::Exile);
    assert_eq!(g.state.objects[dragon as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].mana, [0; 6]);
}

#[test]
fn activation_x_life_accounts_for_life_spent_by_the_required_costed_mana_child() {
    const MANA: CardRules = CardRules {
        effect: Effect::Creature(1, 1),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                life: 1,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 1, 0, 0, 3,
            ]))],
            sorcery: false,
            ..EQUIP.activated[0]
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &X_LIFE),
        (Card::from_name("Goblin Piker").unwrap(), &MANA),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let dork = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    g.state.players[0].life = 4;
    g.state.players[0].mana = [0; 6];
    act(&mut g, activate(source));
    assert_eq!(
        g.x_choice().unwrap().intervals,
        vec![XInterval {
            minimum: 0,
            maximum: 3
        }]
    );
    reject(&mut g, Action::ChooseX { value: 4 });
    act(&mut g, Action::ChooseX { value: 2 });
    act(
        &mut g,
        Action::ActivateMana {
            card: dork,
            ability: 0,
            production: [0, 0, 1, 0, 0, 3],
            payment: [0; 6],
            counter_payment: Counters::default(),
        },
    );
    assert_eq!(g.state.players[0].life, 3);
    pay(&mut g);
    choose_life_row(&mut g, 2);
    assert_eq!(g.state.players[0].life, 1);
    settle(&mut g);
    assert_eq!(g.state.players[0].life, 3);
}

#[test]
fn activation_x_life_declaration_and_unpaid_costs_are_owned_and_restore_without_parent() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &X_LIFE)]);
    let mut parent = scenario(Player::P0);
    let source = put(&mut parent, Player::P0, "Grizzly Bears", Zone::Battlefield);
    parent.state.players[0].life = 7;
    parent.state.players[0].mana = [0, 0, 1, 0, 0, 3];
    act(&mut parent, activate(source));
    let saved = parent.snapshot();
    let revision = parent.revision();
    let action = Action::ChooseX { value: 3 };
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let sibling = parent.clone();
    drop(parent);
    assert!(raw.observe(Player::P1).x_choice.is_none());
    assert_eq!(
        raw.apply(Player::P1, action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(raw.snapshot(), saved);
    act(&mut raw, action.clone());
    indexed.apply_index(Player::P0, revision, index).unwrap();
    pay(&mut raw);
    pay(&mut indexed);
    choose_life_row(&mut raw, 3);
    choose_life_row(&mut indexed, 3);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(sibling.snapshot(), saved);
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(Player::P0, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    pay(&mut indexed);
    choose_life_row(&mut indexed, 3);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}

#[test]
fn target_color_reduction_keeps_named_tax_wide_until_the_final_payable_total() {
    const TAX_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::NamedActivationGenericIncrease(
            u32::MAX,
        )],
    }];
    const TAX: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: TAX_GROUPS,
        extras: &CardExtras {
            static_ability_groups: TAX_GROUPS,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &EQUIP),
        (Card::from_name("Ghost Vacuum").unwrap(), &TAX),
    ]);
    for p in [Player::P0, Player::P1] {
        for (mask, expected) in [(31u8, Some(u32::MAX - 1)), (15, Some(u32::MAX)), (7, None)] {
            let mut g = scenario(p);
            let equipment = put(&mut g, p, "Short Bow", Zone::Battlefield);
            let creature = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            color(&mut g, creature, mask);
            let tax = put(&mut g, p.other(), "Ghost Vacuum", Zone::Battlefield);
            g.set_entry_value(
                g.object_ref(tax),
                EntryChoiceValue::CardName(CardName::from_name("Short Bow").unwrap()),
            );
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, u32::MAX];
            if let Some(total) = expected {
                assert!(
                    g.legal_actions().contains(&activate(equipment)),
                    "wide tax with {mask} target colors must remain payable"
                );
                act(&mut g, activate(equipment));
                target(&mut g, creature);
                assert_eq!(g.observe(p).payment_request.unwrap().generic, total);
                pay(&mut g);
                settle(&mut g);
                assert_eq!(
                    g.state.objects[equipment as usize].attached_to,
                    Some(g.object_ref(creature))
                );
                assert_eq!(
                    g.state.players[p.index()].mana,
                    [0, 0, 0, 0, 0, u32::MAX - total]
                );
            } else {
                reject(&mut g, activate(equipment));
            }
        }
    }
}

#[test]
fn target_color_and_named_tax_total_remains_locked_when_the_tax_provider_funds_itself_away() {
    const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::NamedActivationGenericIncrease(2)],
    }];
    const TAX_MANA: CardRules = CardRules {
        effect: Effect::Creature(1, 1),
        static_groups: GROUPS,
        extras: &CardExtras {
            static_ability_groups: GROUPS,
            ..CardExtras::EMPTY
        },
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 0, 1,
            ]))],
            sorcery: false,
            ..EQUIP.activated[0]
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &EQUIP),
        (Card::from_name("Goblin Piker").unwrap(), &TAX_MANA),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let equipment = put(&mut g, p, "Short Bow", Zone::Battlefield);
        let creature = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        color(&mut g, creature, 31);
        let provider = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        g.set_entry_value(
            g.object_ref(provider),
            EntryChoiceValue::CardName(CardName::from_name("Short Bow").unwrap()),
        );
        g.state.players[p.index()].mana = [0; 6];
        act(&mut g, activate(equipment));
        target(&mut g, creature);
        assert_eq!(g.observe(p).payment_request.unwrap().generic, 1);
        act(
            &mut g,
            Action::ActivateMana {
                card: provider,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 0, 1],
            },
        );
        assert_eq!(g.state.objects[provider as usize].zone, Zone::Graveyard);
        assert_eq!(g.observe(p).payment_request.unwrap().generic, 1);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(
            g.state.objects[equipment as usize].attached_to,
            Some(g.object_ref(creature))
        );
    }
}
