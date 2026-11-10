//! Shared additional/alternative and target/scalar costs. No TDM definitions.
mod support;
use rustmage as _;
use rustmage_core::CardName;
use rustmage_core::PermanentType;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const FREE_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::FreeSpellCast(CastFilter::Subtype(
        "Dragon",
    ))],
}];
const FILTERED_FREE: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: FREE_GROUPS,
    extras: &CardExtras {
        static_ability_groups: FREE_GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const FREE_ANY_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    fragments: &[StaticAbilityFragment::FreeSpellCast(CastFilter::Any)],
    ..FREE_GROUPS[0]
}];
const FREE_ANY: CardRules = CardRules {
    static_groups: FREE_ANY_GROUPS,
    extras: &CardExtras {
        static_ability_groups: FREE_ANY_GROUPS,
        ..CardExtras::EMPTY
    },
    ..FILTERED_FREE
};
fn free(card: CardId) -> Action {
    Action::AnnounceFree {
        card,
        face: CastFace::Normal,
        flash_surcharge: false,
    }
}
fn finish_costs(g: &mut Game, material: Option<CardId>) {
    for _ in 0..16 {
        if g.state.announcing.is_none() {
            return;
        }
        match g.state.pending {
            Pending::PayCosts { .. } => pay(g),
            Pending::CastCosts { .. } => {
                let i = g
                    .observe(g.actor().unwrap())
                    .cast_costs
                    .unwrap()
                    .clauses
                    .iter()
                    .position(|r| !r.paid)
                    .unwrap();
                act(g, Action::ChooseCastCost { clause: i as u16 });
            }
            Pending::AdditionalCost { .. } => {
                let object = g.object_ref(material.expect("additional material"));
                act(g, Action::ChooseAdditionalCost { object });
            }
            _ => panic!("unexpected cost choice {:?}", g.state.pending),
        }
    }
    panic!("cost completion ceiling");
}

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
            creature_only: false,
            legendary: false,
            food_only: false,
            artifact_noncreature: false,
            enchantment_noncreature: false,
            colors: Some(mask),
            creature_types: None,
            all_creature_types: false,
            additional_creature_types: false,
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

const BEHOLD: BeholdCastSpec = BeholdCastSpec {
    subtype: "Dragon",
    decline_generic: 0,
    grants_flash: false,
};
const BEHOLD_SPELL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::IfBeheld {
                beheld: true,
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 4),
        ],
    ),
    extras: &CardExtras {
        behold_cast: Some(BEHOLD),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
fn cost_row(g: &mut Game, clause: CastCostClause) {
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
        .position(|row| row.clause == clause && !row.paid)
        .unwrap();
    act(
        g,
        Action::ChooseCastCost {
            clause: index as u16,
        },
    );
}
fn behold(g: &mut Game, id: CardId) {
    cost_row(g, CastCostClause::Behold("Dragon"));
    let object = g.object_ref(id);
    act(g, Action::ChooseBeholdCost { object });
}

#[test]
fn behold_optional_cast_declines_or_pays_with_owned_hand_or_controlled_permanent() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &BEHOLD_SPELL)]);
    for p in [Player::P0, Player::P1] {
        for witness in [None, Some(Zone::Hand), Some(Zone::Battlefield)] {
            let mut g = scenario(p);
            let dragon = witness.map(|zone| put(&mut g, p, "Shivan Dragon", zone));
            let spell = put(&mut g, p, "Opt", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: spell });
            act(
                &mut g,
                Action::ChooseCastBehold {
                    pay: dragon.is_some(),
                },
            );
            pay(&mut g);
            if let Some(dragon) = dragon {
                behold(&mut g, dragon);
            }
            assert_eq!(
                g.state.stack.last().unwrap().cast.unwrap().beheld,
                Some(dragon.is_some())
            );
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                20 + if dragon.is_some() { 4 } else { 0 }
            );
            if let Some(dragon) = dragon {
                assert_eq!(g.state.objects[dragon as usize].zone, witness.unwrap());
            }
        }
    }
}

#[test]
fn behold_instead_generic_cost_preserves_colored_mana_and_decline_fee() {
    const FEE: CardRules = CardRules {
        extras: &CardExtras {
            behold_cast: Some(BeholdCastSpec {
                decline_generic: 1,
                ..BEHOLD
            }),
            ..CardExtras::EMPTY
        },
        ..BEHOLD_SPELL
    };
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &FEE)]);
    let mut g = scenario(Player::P0);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    g.state.players[0].mana = [0, 1, 0, 0, 0, 0];
    reject(&mut g, Action::AnnounceCast { card: spell });
    g.state.players[0].mana[5] = 1;
    act(&mut g, Action::AnnounceCast { card: spell });
    reject(&mut g, Action::ChooseCastBehold { pay: true });
    act(&mut g, Action::ChooseCastBehold { pay: false });
    assert_eq!(g.observe(Player::P0).payment_request.unwrap().generic, 1);
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert_eq!(g.state.players[0].life, 20);
    let dragon = put(&mut g, Player::P0, "Shivan Dragon", Zone::Hand);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    g.state.players[0].mana = [0, 1, 0, 0, 0, 0];
    act(&mut g, Action::AnnounceCast { card: spell });
    reject(&mut g, Action::ChooseCastBehold { pay: false });
    act(&mut g, Action::ChooseCastBehold { pay: true });
    pay(&mut g);
    behold(&mut g, dragon);
    settle(&mut g);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert_eq!(g.state.players[0].life, 24);
}

#[test]
fn behold_flash_route_requires_actual_payment_and_rejects_decline_off_sorcery_timing() {
    const FLASH: CardRules = CardRules {
        extras: &CardExtras {
            behold_cast: Some(BeholdCastSpec {
                grants_flash: true,
                ..BEHOLD
            }),
            ..CardExtras::EMPTY
        },
        ..BEHOLD_SPELL
    };
    let _scope = fixture_rules::activate(&[(Card::from_name("Divination").unwrap(), &FLASH)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.state.active = p.other();
        let spell = put(&mut g, p, "Divination", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: spell });
        let dragon = put(&mut g, p, "Shivan Dragon", Zone::Battlefield);
        g.state.objects[dragon as usize].gained_keywords = Keywords::HEXPROOF;
        act(&mut g, Action::AnnounceCast { card: spell });
        reject(&mut g, Action::ChooseCastBehold { pay: false });
        act(&mut g, Action::ChooseCastBehold { pay: true });
        pay(&mut g);
        behold(&mut g, dragon);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().beheld,
            Some(true)
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 24);
    }
}

#[test]
fn behold_reveals_only_the_paid_hand_card_and_copied_qualification_survives_departure() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &BEHOLD_SPELL)]);
    let mut g = scenario(Player::P0);
    let dragon = put(&mut g, Player::P0, "Shivan Dragon", Zone::Hand);
    let hidden = put(&mut g, Player::P0, "Forest", Zone::Hand);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: spell });
    act(&mut g, Action::ChooseCastBehold { pay: true });
    pay(&mut g);
    cost_row(&mut g, CastCostClause::Behold("Dragon"));
    assert!(g.observe(Player::P1).behold_cost.is_none());
    assert!(
        !g.observe(Player::P1)
            .objects
            .iter()
            .any(|o| o.id == dragon || o.id == hidden)
    );
    let object = g.object_ref(dragon);
    act(&mut g, Action::ChooseBeholdCost { object });
    assert!(g.observe(Player::P1).objects.iter().any(|o| o.id == dragon));
    assert!(!g.observe(Player::P1).objects.iter().any(|o| o.id == hidden));
    let template = g.state.stack.last().unwrap().clone();
    g.move_object(dragon, Zone::Graveyard);
    let copied = g.allocate_spell_copy(&template, Player::P1);
    assert_eq!(copied.cast.unwrap().beheld, Some(true));
    g.state.stack.push(copied);
    settle(&mut g);
    assert_eq!(g.state.players[0].life, 24);
    assert_eq!(g.state.players[1].life, 24);
}

#[test]
fn behold_candidate_is_exact_nontargeting_and_owned_across_raw_index_restore() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &BEHOLD_SPELL)]);
    let mut parent = scenario(Player::P1);
    let dragon = put(&mut parent, Player::P1, "Shivan Dragon", Zone::Battlefield);
    let other = put(&mut parent, Player::P0, "Shivan Dragon", Zone::Battlefield);
    let spell = put(&mut parent, Player::P1, "Opt", Zone::Hand);
    act(&mut parent, Action::AnnounceCast { card: spell });
    act(&mut parent, Action::ChooseCastBehold { pay: true });
    pay(&mut parent);
    cost_row(&mut parent, CastCostClause::Behold("Dragon"));
    let action = Action::ChooseBeholdCost {
        object: parent.object_ref(dragon),
    };
    let bad = Action::ChooseBeholdCost {
        object: parent.object_ref(other),
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
    let sibling = parent.clone();
    drop(parent);
    assert_eq!(
        raw.apply(Player::P0, action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(raw.snapshot(), saved);
    reject(&mut raw, bad);
    act(&mut raw, action.clone());
    indexed.apply_index(Player::P1, revision, index).unwrap();
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(sibling.snapshot(), saved);
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(Player::P1, revision, index),
        Err(Error::StaleRevision)
    );
    let mut stale = indexed.clone();
    stale.move_object(dragon, Zone::Hand);
    stale.move_object(dragon, Zone::Battlefield);
    reject(&mut stale, action.clone());
    act(&mut indexed, action);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}

#[test]
fn behold_source_card_cannot_supply_its_own_required_hand_witness_in_preflight() {
    const CREATURE: CardRules = CardRules {
        effect: Effect::Creature(5, 5),
        extras: &CardExtras {
            behold_cast: Some(BeholdCastSpec {
                grants_flash: true,
                ..BEHOLD
            }),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::from_name("Shivan Dragon").unwrap(), &CREATURE)]);
    let mut g = scenario(Player::P0);
    g.state.active = Player::P1;
    let source = put(&mut g, Player::P0, "Shivan Dragon", Zone::Hand);
    reject(&mut g, Action::AnnounceCast { card: source });
    let extra = put(&mut g, Player::P0, "Shivan Dragon", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: source });
    act(&mut g, Action::ChooseCastBehold { pay: true });
    pay(&mut g);
    behold(&mut g, extra);
    settle(&mut g);
    assert_eq!(g.state.objects[source as usize].zone, Zone::Battlefield);
    assert_eq!(g.state.objects[extra as usize].zone, Zone::Hand);
}

#[test]
fn behold_funding_does_not_pay_the_witness_before_a_mana_child_sacrifices_it() {
    const FLASH: CardRules = CardRules {
        extras: &CardExtras {
            behold_cast: Some(BeholdCastSpec {
                grants_flash: true,
                ..BEHOLD
            }),
            ..CardExtras::EMPTY
        },
        ..BEHOLD_SPELL
    };
    const MANA: CardRules = CardRules {
        effect: Effect::Creature(5, 5),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 1, 0, 0, 0, 2,
            ]))],
            sorcery: false,
            ..EQUIP.activated[0]
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Divination").unwrap(), &FLASH),
        (Card::from_name("Shivan Dragon").unwrap(), &MANA),
    ]);
    let mut g = scenario(Player::P0);
    g.state.active = Player::P1;
    let producer = put(&mut g, Player::P0, "Shivan Dragon", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Divination", Zone::Hand);
    g.state.players[0].mana = [0; 6];
    reject(&mut g, Action::AnnounceCast { card: spell });
    let extra = put(&mut g, Player::P0, "Shivan Dragon", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: spell });
    act(&mut g, Action::ChooseCastBehold { pay: true });
    act(
        &mut g,
        Action::ActivateMana {
            card: producer,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 1, 0, 0, 0, 2],
        },
    );
    assert_eq!(g.state.objects[producer as usize].zone, Zone::Graveyard);
    pay(&mut g);
    behold(&mut g, extra);
    settle(&mut g);
    assert_eq!(g.state.players[0].life, 24);
}

#[test]
fn behold_and_bargain_can_share_a_dragon_only_in_the_legal_payment_order() {
    const BARGAIN: CardRules = CardRules {
        extras: &CardExtras {
            bargain_reduction: Some(2),
            ..*BEHOLD_SPELL.extras
        },
        ..BEHOLD_SPELL
    };
    let _scope = fixture_rules::activate(&[(Card::from_name("Quick Study").unwrap(), &BARGAIN)]);
    let mut g = scenario(Player::P0);
    let dragon = put(&mut g, Player::P0, "Shivan Dragon", Zone::Battlefield);
    let mut recipe = CopyRecipe::new(Card::from_name("Shivan Dragon").unwrap());
    recipe.exceptions.artifact = true;
    g.set_copy_recipe(g.object_ref(dragon), recipe);
    let spell = put(&mut g, Player::P0, "Quick Study", Zone::Hand);
    g.state.players[0].mana = [0, 1, 0, 0, 0, 0];
    act(&mut g, Action::AnnounceCast { card: spell });
    act(&mut g, Action::ChooseCastBehold { pay: true });
    let sacrifice = g.object_ref(dragon);
    act(
        &mut g,
        Action::ChooseBargain {
            sacrifice: Some(sacrifice),
        },
    );
    act(&mut g, Action::BeginOrderedCosts);
    let bad = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == CastCostClause::Additional)
        .unwrap();
    reject(&mut g, Action::ChooseCastCost { clause: bad as u16 });
    behold(&mut g, dragon);
    cost_row(&mut g, CastCostClause::Additional);
    cost_row(&mut g, CastCostClause::Mana);
    pay(&mut g);
    assert_eq!(g.state.objects[dragon as usize].zone, Zone::Graveyard);
    assert_eq!(
        g.state.stack.last().unwrap().cast.unwrap().beheld,
        Some(true)
    );
    settle(&mut g);
    assert_eq!(g.state.players[0].life, 24);
}

#[test]
fn behold_and_granted_delve_share_the_ordered_mana_and_witness_frame() {
    const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::SpellDelve(CastFilter::Any)],
    }];
    const PROVIDER: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: GROUPS,
        extras: &CardExtras {
            static_ability_groups: GROUPS,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Quick Study").unwrap(), &BEHOLD_SPELL),
        (Card::from_name("Ghost Vacuum").unwrap(), &PROVIDER),
    ]);
    let mut g = scenario(Player::P1);
    put(&mut g, Player::P1, "Ghost Vacuum", Zone::Battlefield);
    let dragon = put(&mut g, Player::P1, "Shivan Dragon", Zone::Hand);
    let fuel = [
        put(&mut g, Player::P1, "Forest", Zone::Graveyard),
        put(&mut g, Player::P1, "Forest", Zone::Graveyard),
    ];
    let spell = put(&mut g, Player::P1, "Quick Study", Zone::Hand);
    g.state.players[1].mana = [0, 1, 0, 0, 0, 0];
    act(&mut g, Action::AnnounceCast { card: spell });
    act(&mut g, Action::ChooseCastBehold { pay: true });
    act(&mut g, Action::BeginOrderedCosts);
    behold(&mut g, dragon);
    cost_row(&mut g, CastCostClause::Mana);
    for id in fuel {
        let object = g.object_ref(id);
        act(&mut g, Action::ChooseDelver { object });
    }
    pay(&mut g);
    settle(&mut g);
    assert!(
        fuel.into_iter()
            .all(|id| g.state.objects[id as usize].zone == Zone::Exile)
    );
    assert_eq!(g.state.players[1].life, 24);
}

#[test]
fn behold_accepts_a_dragon_already_tapped_for_convoke_or_improvise() {
    const CONVOKE: CardRules = CardRules {
        extras: &CardExtras {
            has_convoke: true,
            ..*BEHOLD_SPELL.extras
        },
        ..BEHOLD_SPELL
    };
    const IMPROVISE: CardRules = CardRules {
        extras: &CardExtras {
            has_improvise: true,
            ..*BEHOLD_SPELL.extras
        },
        ..BEHOLD_SPELL
    };
    for artifact in [false, true] {
        let _scope = fixture_rules::activate(&[(
            Card::from_name("Quick Study").unwrap(),
            if artifact { &IMPROVISE } else { &CONVOKE },
        )]);
        let mut g = scenario(Player::P0);
        let dragon = put(&mut g, Player::P0, "Shivan Dragon", Zone::Battlefield);
        let bear = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        for id in [dragon, bear] {
            let mut recipe = CopyRecipe::new(g.state.objects[id as usize].card);
            recipe.exceptions.artifact = true;
            g.set_copy_recipe(g.object_ref(id), recipe);
        }
        let spell = put(&mut g, Player::P0, "Quick Study", Zone::Hand);
        g.state.players[0].mana = [0, 1, 0, 0, 0, 0];
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseCastBehold { pay: true });
        act(&mut g, Action::BeginOrderedCosts);
        cost_row(&mut g, CastCostClause::Mana);
        for id in [dragon, bear] {
            let object = g.object_ref(id);
            act(
                &mut g,
                if artifact {
                    Action::ChooseImproviser { object }
                } else {
                    Action::ChooseConvoker {
                        object,
                        contribution: ConvokeContribution::Generic,
                    }
                },
            );
        }
        pay(&mut g);
        assert!(g.state.objects[dragon as usize].tapped);
        behold(&mut g, dragon);
        settle(&mut g);
        assert_eq!(g.state.players[0].life, 24);
    }
}

#[test]
fn behold_and_granted_harmonize_keep_printed_mana_and_both_independent_clauses() {
    let _scope =
        fixture_rules::activate(&[(Card::from_name("Quick Study").unwrap(), &BEHOLD_SPELL)]);
    let mut g = scenario(Player::P0);
    let dragon = put(&mut g, Player::P0, "Shivan Dragon", Zone::Hand);
    let helper = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Quick Study", Zone::Graveyard);
    g.grant_graveyard_permission(
        Player::P0,
        g.object_ref(spell),
        GraveyardCastKind::Harmonize,
    );
    g.state.players[0].mana = [0, 1, 0, 0, 0, 0];
    act(
        &mut g,
        Action::AnnounceGrantedHarmonize {
            card: spell,
            face: CastFace::Normal,
        },
    );
    act(&mut g, Action::ChooseCastBehold { pay: true });
    let object = g.object_ref(helper);
    act(
        &mut g,
        Action::ChooseHarmonizer {
            object: Some(object),
        },
    );
    pay(&mut g);
    behold(&mut g, dragon);
    cost_row(&mut g, CastCostClause::Harmonize(object));
    settle(&mut g);
    assert!(g.state.objects[helper as usize].tapped);
    assert_eq!(g.state.objects[spell as usize].zone, Zone::Exile);
    assert_eq!(g.state.players[0].life, 24);
}

const MAY_BEHOLD: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayBehold {
                subtype: "Dragon",
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 4),
        ],
    }],
    ..CardRules::EMPTY
};
fn wait_hand(g: &mut Game) {
    for _ in 0..16 {
        if matches!(g.state.pending, Pending::HandCardChoice { .. }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("bounded resolving behold");
}

#[test]
fn optional_resolving_behold_hides_unmatched_hand_quality_and_choice_progression() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &MAY_BEHOLD)]);
    for p in [Player::P0, Player::P1] {
        let mut dragon = scenario(p);
        let mut other = scenario(p);
        put(&mut dragon, p, "Shivan Dragon", Zone::Hand);
        put(&mut other, p, "Opt", Zone::Hand);
        let source = put(&mut dragon, p, "Grizzly Bears", Zone::Hand);
        cast(&mut dragon, source, None);
        let source = put(&mut other, p, "Grizzly Bears", Zone::Hand);
        cast(&mut other, source, None);
        wait_hand(&mut dragon);
        wait_hand(&mut other);
        assert_eq!(
            serde_json::to_value(dragon.observe(p.other())).unwrap(),
            serde_json::to_value(other.observe(p.other())).unwrap()
        );
        assert_eq!(dragon.observe(p).hand_choice.unwrap().candidates.len(), 1);
        assert!(other.observe(p).hand_choice.unwrap().candidates.is_empty());
        act(&mut dragon, Action::ChooseHandCard { object: None });
        act(&mut other, Action::ChooseHandCard { object: None });
        settle(&mut dragon);
        settle(&mut other);
        assert_eq!(
            serde_json::to_value(dragon.observe(p.other())).unwrap(),
            serde_json::to_value(other.observe(p.other())).unwrap()
        );
        assert_eq!(dragon.state.players[p.index()].life, 20);
    }
}

#[test]
fn optional_resolving_behold_is_source_independent_owned_and_transactional() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &MAY_BEHOLD)]);
    let mut parent = scenario(Player::P0);
    let dragon = put(&mut parent, Player::P0, "Shivan Dragon", Zone::Hand);
    let permanent = put(&mut parent, Player::P0, "Shivan Dragon", Zone::Battlefield);
    let source = put(&mut parent, Player::P0, "Grizzly Bears", Zone::Hand);
    cast(&mut parent, source, None);
    wait_hand(&mut parent);
    parent.move_object(source, Zone::Exile);
    let reference = parent.object_ref(permanent);
    let mut changed = parent.clone();
    changed.apply_temporary_copy(
        reference,
        CopyRecipe::new(Card::from_name("Hill Giant").unwrap()),
        CopyDeadline::EndOfTurn,
    );
    reject(
        &mut changed,
        Action::ChooseHandCard {
            object: Some(reference),
        },
    );
    let mut controlled = parent.clone();
    controlled.change_controller(reference, Player::P1);
    reject(
        &mut controlled,
        Action::ChooseHandCard {
            object: Some(reference),
        },
    );
    let action = Action::ChooseHandCard {
        object: Some(parent.object_ref(dragon)),
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
    assert!(raw.observe(Player::P1).hand_choice.is_none());
    assert_eq!(
        raw.apply(Player::P1, action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(raw.snapshot(), saved);
    act(&mut raw, action.clone());
    indexed.apply_index(Player::P0, revision, index).unwrap();
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.state.players[0].life, 24);
    assert!(
        raw.observe(Player::P1)
            .objects
            .iter()
            .any(|o| o.id == dragon)
    );
    act(&mut sibling, Action::ChooseHandCard { object: None });
    settle(&mut sibling);
    assert_eq!(sibling.state.players[0].life, 20);
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(Player::P0, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}

#[test]
fn filtered_free_cost_qualifies_the_selected_face_and_preserves_timing_and_controller() {
    const ADVENTURE: CardRules = CardRules {
        effect: Effect::Creature(5, 5),
        extras: &CardExtras {
            adventure_spell: Some(Card::Shock),
            omen_spell: Some(Card::Divination),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let dragon = Card::from_name("Shivan Dragon").unwrap();
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Ghost Vacuum").unwrap(), &FILTERED_FREE),
        (dragon, &ADVENTURE),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = put(&mut g, p, "Ghost Vacuum", Zone::Battlefield);
        let spell = put(&mut g, p, "Shivan Dragon", Zone::Hand);
        let other = put(&mut g, p, "Bear Cub", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        assert!(g.legal_actions().contains(&free(spell)));
        reject(&mut g, free(other));
        for face in [CastFace::Adventure, CastFace::Omen] {
            reject(
                &mut g,
                Action::AnnounceFree {
                    card: spell,
                    face,
                    flash_surcharge: false,
                },
            );
        }
        g.state.active = p.other();
        reject(&mut g, free(spell));
        g.state.active = p;
        g.state.objects[provider as usize].controller = p.other();
        reject(&mut g, free(spell));
        g.state.objects[provider as usize].controller = p;
        act(&mut g, free(spell));
        finish_costs(&mut g, None);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().method,
            CastMethod::Free
        );
        settle(&mut g);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Battlefield);
    }
}

#[test]
fn filtered_free_cost_requires_separate_exact_graveyard_exile_or_library_permission() {
    const TOP_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        fragments: &[StaticAbilityFragment::CastCreatureFromLibraryTop],
        ..FREE_GROUPS[0]
    }];
    const TOP: CardRules = CardRules {
        static_groups: TOP_GROUPS,
        extras: &CardExtras {
            static_ability_groups: TOP_GROUPS,
            ..CardExtras::EMPTY
        },
        ..FILTERED_FREE
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Ghost Vacuum").unwrap(), &FILTERED_FREE),
        (Card::GrizzlyBears, &TOP),
    ]);
    for p in [Player::P0, Player::P1] {
        for zone in [Zone::Graveyard, Zone::Exile, Zone::Library] {
            let mut g = scenario(p);
            put(&mut g, p, "Ghost Vacuum", Zone::Battlefield);
            let spell = put(&mut g, p, "Shivan Dragon", zone);
            g.state.players[p.index()].mana = [0; 6];
            reject(&mut g, free(spell));
            match zone {
                Zone::Graveyard => {
                    g.grant_graveyard_permission(p, g.object_ref(spell), GraveyardCastKind::Cast)
                }
                Zone::Exile => g.grant_exile_play_permission(p, g.object_ref(spell), false),
                Zone::Library => {
                    put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
                }
                _ => unreachable!(),
            }
            assert!(g.legal_actions().contains(&free(spell)));
            let mut stale = g.clone();
            stale.move_object(spell, Zone::Hand);
            stale.move_object(spell, zone);
            if zone != Zone::Library {
                reject(&mut stale, free(spell));
            }
            act(&mut g, free(spell));
            finish_costs(&mut g, None);
            let cast = g.state.stack.last().unwrap().cast.unwrap();
            assert_eq!(cast.origin, zone);
            assert_eq!(cast.method, CastMethod::Free);
            settle(&mut g);
            assert_eq!(g.state.objects[spell as usize].zone, Zone::Battlefield);
        }
    }
}

#[test]
fn filtered_free_cost_keeps_harmonize_and_flashback_as_competing_alternatives() {
    const ALTERNATIVES: CardRules = CardRules {
        effect: Effect::Creature(5, 5),
        extras: &CardExtras {
            harmonize_cost: Some(ManaCost {
                colored: [0; 6],
                generic: 1,
            }),
            flashback_cost: Some(ManaCost {
                colored: [0; 6],
                generic: 1,
            }),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Ghost Vacuum").unwrap(), &FILTERED_FREE),
        (Card::from_name("Shivan Dragon").unwrap(), &ALTERNATIVES),
    ]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Ghost Vacuum", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Shivan Dragon", Zone::Graveyard);
    g.state.players[0].mana = [0; 6];
    g.grant_graveyard_permission(
        Player::P0,
        g.object_ref(spell),
        GraveyardCastKind::Harmonize,
    );
    reject(&mut g, free(spell));
    g.grant_graveyard_permission(
        Player::P0,
        g.object_ref(spell),
        GraveyardCastKind::Flashback,
    );
    reject(&mut g, free(spell));
    g.grant_graveyard_permission(Player::P0, g.object_ref(spell), GraveyardCastKind::Cast);
    act(&mut g, free(spell));
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseHarmonizer { .. }))
    );
    finish_costs(&mut g, None);
    settle(&mut g);
    assert_eq!(g.state.objects[spell as usize].zone, Zone::Battlefield);
}

#[test]
fn free_cost_retains_taxes_mandatory_additional_costs_and_zero_printed_x() {
    const TAX_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        fragments: &[StaticAbilityFragment::OpponentNoncreatureSpellTax(2)],
        ..FREE_GROUPS[0]
    }];
    const TAX: CardRules = CardRules {
        static_groups: TAX_GROUPS,
        extras: &CardExtras {
            static_ability_groups: TAX_GROUPS,
            ..CardExtras::EMPTY
        },
        ..FILTERED_FREE
    };
    const SAC: AdditionalCost = AdditionalCost {
        kind: EffectChoiceKind::Sacrifice,
        filter: SacrificeFilter {
            keywords: Keywords::NONE,
            source_only: false,
            kinds: 1,
            nonland: false,
            other: false,
            tokens: None,
            artifact_enchantment_or_token: false,
            subtype: None,
        },
    };
    const X: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::GainLifeScaled(
                PlayerSet::Controller,
                AmountValue::ChosenX,
            )],
        ),
        extras: &CardExtras {
            additional_cost: Some(SAC),
            x_symbols: 1,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Ghost Vacuum").unwrap(), &FREE_ANY),
        (Card::GrizzlyBears, &TAX),
        (Card::from_name("Spectral Denial").unwrap(), &X),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Ghost Vacuum", Zone::Battlefield);
        put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, "Spectral Denial", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(&mut g, free(spell));
        g.state.players[p.index()].mana[5] = 2;
        reject(&mut g, free(spell));
        let material = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        act(&mut g, free(spell));
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseX { .. }))
        );
        finish_costs(&mut g, Some(material));
        assert_eq!(g.state.stack.last().unwrap().x, Some(0));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.objects[material as usize].zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20);
    }
}

#[test]
fn filtered_free_graveyard_route_preserves_its_life_and_sacrifice_costs_independently() {
    const SAC: AdditionalCost = AdditionalCost {
        kind: EffectChoiceKind::Sacrifice,
        filter: SacrificeFilter {
            keywords: Keywords::NONE,
            source_only: false,
            kinds: 16,
            nonland: false,
            other: false,
            tokens: None,
            artifact_enchantment_or_token: false,
            subtype: None,
        },
    };
    const ORIGIN_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        fragments: &[
            StaticAbilityFragment::SelfGraveyardCastWithAdditionalCosts {
                life: 2,
                additional: SAC,
            },
        ],
        ..FREE_GROUPS[0]
    }];
    const ORIGIN: CardRules = CardRules {
        effect: Effect::Creature(5, 5),
        static_groups: ORIGIN_GROUPS,
        extras: &CardExtras {
            static_ability_groups: ORIGIN_GROUPS,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Ghost Vacuum").unwrap(), &FILTERED_FREE),
        (Card::from_name("Shivan Dragon").unwrap(), &ORIGIN),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = put(&mut g, p, "Ghost Vacuum", Zone::Battlefield);
        let spell = put(&mut g, p, "Shivan Dragon", Zone::Graveyard);
        g.state.players[p.index()].mana = [0; 6];
        let route = Action::AnnounceFreeGraveyardAdditionalCosts {
            card: spell,
            face: CastFace::Normal,
            flash_surcharge: false,
        };
        g.state.players[p.index()].life = 1;
        reject(&mut g, route.clone());
        g.state.players[p.index()].life = 20;
        assert!(!g.legal_actions().contains(&free(spell)));
        act(&mut g, route);
        finish_costs(&mut g, Some(provider));
        assert_eq!(g.state.players[p.index()].life, 18);
        assert_eq!(g.state.objects[provider as usize].zone, Zone::Graveyard);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().method,
            CastMethod::Free
        );
        settle(&mut g);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Battlefield);
        // An independent ordinary permission does not inherit the self-route costs.
        let next = put(&mut g, p, "Shivan Dragon", Zone::Graveyard);
        put(&mut g, p, "Ghost Vacuum", Zone::Battlefield);
        g.grant_graveyard_permission(p, g.object_ref(next), GraveyardCastKind::Cast);
        act(&mut g, free(next));
        finish_costs(&mut g, None);
        assert_eq!(g.state.players[p.index()].life, 18);
    }
}

#[test]
fn filtered_free_graveyard_routes_pay_counter_cost_or_exact_permanent_type_allowance() {
    const COUNTER: CardRules = CardRules {
        effect: Effect::Creature(5, 5),
        extras: &CardExtras {
            graveyard_counter_cost: Some(1),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    const TYPES_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        fragments: &[StaticAbilityFragment::GraveyardPermanentTypes],
        ..FREE_GROUPS[0]
    }];
    const TYPES: CardRules = CardRules {
        static_groups: TYPES_GROUPS,
        extras: &CardExtras {
            static_ability_groups: TYPES_GROUPS,
            ..CardExtras::EMPTY
        },
        ..FILTERED_FREE
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Ghost Vacuum").unwrap(), &FILTERED_FREE),
        (Card::from_name("Shivan Dragon").unwrap(), &COUNTER),
        (Card::GrizzlyBears, &TYPES),
    ]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Ghost Vacuum", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Shivan Dragon", Zone::Graveyard);
    g.state.players[0].mana = [0; 6];
    let route = Action::AnnounceFreeCounterGraveyard {
        card: spell,
        face: CastFace::Normal,
        flash_surcharge: false,
    };
    reject(&mut g, route.clone());
    let helper = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.add_counter(helper, CounterKind::PlusOne, 1);
    act(&mut g, route);
    let object = g.object_ref(helper);
    act(
        &mut g,
        Action::ChooseCreatureCounterCost {
            object,
            counter: CounterKind::PlusOne,
        },
    );
    finish_costs(&mut g, None);
    assert_eq!(g.counter_count(helper, CounterKind::PlusOne), 0);
    settle(&mut g);
    let provider = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let next = put(&mut g, Player::P0, "Shivan Dragon", Zone::Graveyard);
    let permission = g.object_ref(provider);
    act(
        &mut g,
        Action::AnnounceFreeGraveyardPermanent {
            card: next,
            face: CastFace::Normal,
            flash_surcharge: false,
            permission,
            permanent_type: PermanentType::Creature,
        },
    );
    finish_costs(&mut g, None);
    settle(&mut g);
    let third = put(&mut g, Player::P0, "Shivan Dragon", Zone::Graveyard);
    reject(
        &mut g,
        Action::AnnounceFreeGraveyardPermanent {
            card: third,
            face: CastFace::Normal,
            flash_surcharge: false,
            permission,
            permanent_type: PermanentType::Creature,
        },
    );
}

#[test]
fn filtered_free_cost_can_replace_paid_resolution_cost_but_not_a_required_alternative() {
    const EXILE: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[
                Instruction::ExileTopMayCastPaid,
                Instruction::GainLife(PlayerSet::Controller, 1),
            ],
        ),
        ..CardRules::EMPTY
    };
    const GRAVE: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[
                Instruction::CastGraveyard {
                    kinds: u8::MAX,
                    exile_instants_sorceries: false,
                },
                Instruction::GainLife(PlayerSet::Controller, 1),
            ],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Ghost Vacuum").unwrap(), &FILTERED_FREE),
        (Card::from_name("Opt").unwrap(), &EXILE),
        (Card::Divination, &GRAVE),
    ]);
    for zone in [Zone::Exile, Zone::Graveyard] {
        let mut g = scenario(Player::P0);
        put(&mut g, Player::P0, "Ghost Vacuum", Zone::Battlefield);
        let spell = put(
            &mut g,
            Player::P0,
            "Shivan Dragon",
            if zone == Zone::Exile {
                Zone::Library
            } else {
                zone
            },
        );
        let offer = put(
            &mut g,
            Player::P0,
            if zone == Zone::Exile {
                "Opt"
            } else {
                "Divination"
            },
            Zone::Hand,
        );
        cast(&mut g, offer, None);
        g.state.players[0].mana = [0; 6];
        for _ in 0..8 {
            if matches!(g.state.pending, Pending::ResolutionCasts { .. }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert!(g.state.resolving.is_some());
        assert_eq!(g.state.objects[spell as usize].zone, zone);
        let object = g.object_ref(spell);
        let action = Action::CastFreeDuringResolution {
            object,
            face: CastFace::Normal,
        };
        let mut forced = g.clone();
        let grant = forced.state.resolution_free_casts.as_mut().unwrap();
        grant.paid = None;
        grant.alternative_cost = Some(ManaCost {
            colored: [0; 6],
            generic: 1,
        });
        reject(&mut forced, action.clone());
        act(&mut g, action);
        finish_costs(&mut g, None);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().method,
            CastMethod::Free
        );
        assert_eq!(
            g.state
                .resolution_free_casts
                .as_ref()
                .unwrap()
                .casts_remaining,
            Some(0)
        );
        act(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[offer as usize].zone, Zone::Graveyard);
        assert_eq!(
            g.state.players[0].life, 21,
            "paid-offer parent resumed exactly once"
        );
    }
}

#[test]
fn filtered_free_behold_choices_keep_the_alternative_cost_and_fee_on_owned_branches() {
    const CREATURE: CardRules = CardRules {
        effect: Effect::Creature(5, 5),
        extras: &CardExtras {
            behold_cast: Some(BeholdCastSpec {
                decline_generic: 1,
                ..BEHOLD
            }),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Ghost Vacuum").unwrap(), &FILTERED_FREE),
        (Card::from_name("Shivan Dragon").unwrap(), &CREATURE),
    ]);
    let mut parent = scenario(Player::P0);
    put(&mut parent, Player::P0, "Ghost Vacuum", Zone::Battlefield);
    let spell = put(&mut parent, Player::P0, "Shivan Dragon", Zone::Hand);
    let witness = put(&mut parent, Player::P0, "Shivan Dragon", Zone::Hand);
    parent.state.players[0].mana = [0, 0, 0, 0, 0, 1];
    act(&mut parent, free(spell));
    let action = Action::ChooseCastBehold { pay: true };
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
    assert_eq!(
        raw.apply(Player::P1, action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(raw.snapshot(), saved);
    act(&mut raw, action.clone());
    indexed.apply_index(Player::P0, revision, index).unwrap();
    for g in [&mut raw, &mut indexed] {
        pay(g);
        behold(g, witness);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.state.players[0].mana[5], 1);
    assert_eq!(
        raw.state.stack.last().unwrap().cast.unwrap().method,
        CastMethod::Free
    );
    act(&mut sibling, Action::ChooseCastBehold { pay: false });
    finish_costs(&mut sibling, None);
    assert_eq!(sibling.state.players[0].mana[5], 0);
    assert_eq!(
        sibling.state.stack.last().unwrap().cast.unwrap().beheld,
        Some(false)
    );
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(Player::P0, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    pay(&mut indexed);
    behold(&mut indexed, witness);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}

#[test]
fn free_alternative_stays_locked_when_its_provider_is_the_required_sacrificed_mana_child() {
    const TAX_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        fragments: &[StaticAbilityFragment::OpponentNoncreatureSpellTax(2)],
        ..FREE_GROUPS[0]
    }];
    const TAX: CardRules = CardRules {
        static_groups: TAX_GROUPS,
        extras: &CardExtras {
            static_ability_groups: TAX_GROUPS,
            ..CardExtras::EMPTY
        },
        ..FILTERED_FREE
    };
    const PRODUCER: CardRules = CardRules {
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 0, 2,
            ]))],
            sorcery: false,
            ..EQUIP.activated[0]
        }],
        ..FREE_ANY
    };
    const EMPTY: CardRules = CardRules {
        effect: Effect::Spell(TargetPlan::NONE, &[]),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Ghost Vacuum").unwrap(), &PRODUCER),
        (Card::GrizzlyBears, &TAX),
        (Card::from_name("Opt").unwrap(), &EMPTY),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = put(&mut g, p, "Ghost Vacuum", Zone::Battlefield);
        put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, "Opt", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        act(&mut g, free(spell));
        assert_eq!(g.observe(p).payment_request.unwrap().generic, 2);
        act(
            &mut g,
            Action::ActivateMana {
                card: provider,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 0, 2],
            },
        );
        assert_eq!(g.state.objects[provider as usize].zone, Zone::Graveyard);
        assert_eq!(g.observe(p).payment_request.unwrap().generic, 2);
        pay(&mut g);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().method,
            CastMethod::Free
        );
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
    }
}
