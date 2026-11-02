//! Frozen mana recipes resolve quantities after costs and own late choices.
mod support;
use rustmage::Card;
use rustmage_core::CardName;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const TWO: Mana = [0, 0, 0, 0, 0, 2];
const fn ability(cost: ActivationCost) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost,
        targets: TargetPlan::NONE,
        program: &[],
        sorcery: false,
        once_per_turn: false,
    }
}
const BASE: ActivatedAbility = ability(ActivationCost::ZERO);
const TAX_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::NamedActivationGenericIncrease(2)],
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
fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn tax(g: &mut Game, p: Player, name: &str) {
    let source = put(g, p, "Ghost Vacuum", Zone::Battlefield);
    g.set_entry_value(
        g.object_ref(source),
        EntryChoiceValue::CardName(CardName::from_name(name).unwrap()),
    );
}
fn announce(card: CardId, production: Mana) -> Action {
    Action::AnnounceMana {
        card,
        ability: 0,
        counter_payment: Counters::default(),
        production,
    }
}
fn direct(card: CardId, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}
fn indexed(g: &mut Game, action: Action) {
    let actions = g.legal_actions();
    let index = actions
        .iter()
        .position(|a| *a == action)
        .expect("indexed action");
    g.apply_index(g.actor().unwrap(), g.revision(), index)
        .unwrap();
}

#[test]
fn power_mana_samples_after_counter_cost_and_uses_its_actual_departure_power() {
    const COST: ActivationCost = ActivationCost {
        tap: true,
        remove: Some((Some(CounterKind::PlusOne), 1)),
        ..ActivationCost::ZERO
    };
    const FIXED: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ActivatedAbility {
            cost: COST,
            program: &[Instruction::AddMana(ManaProduction::SourcePower {
                color: 4,
            })],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    const SACRIFICE: CardRules = CardRules {
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                sacrifice: true,
                ..COST
            },
            program: &[Instruction::AddMana(ManaProduction::SourcePower {
                color: 4,
            })],
            ..BASE
        }],
        ..FIXED
    };
    for rules in [&FIXED, &SACRIFICE] {
        let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, rules)]);
        for p in [Player::P0, Player::P1] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            g.add_counter(source, CounterKind::PlusOne, 2);
            g.state.players[p.index()].mana = [0; 6];
            let mut counters = Counters::default();
            counters.add(CounterKind::PlusOne, 1);
            act(
                &mut g,
                Action::ActivateMana {
                    card: source,
                    ability: 0,
                    payment: [0; 6],
                    counter_payment: counters,
                    production: [0, 0, 0, 0, 4, 0],
                },
            );
            assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 3, 0]);
            assert_eq!(
                g.object_view(source).zone,
                if rules.activated[0].cost.sacrifice {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}

#[test]
fn parent_cast_funding_uses_the_post_cost_amount_and_rejects_a_stale_larger_preview() {
    const SOURCE: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                remove: Some((Some(CounterKind::PlusOne), 1)),
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::SourcePower {
                color: 1,
            })],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &SOURCE)]);
    for counters in [1, 2] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        g.add_counter(source, CounterKind::PlusOne, counters);
        let spell = put(&mut g, Player::P0, "Divination", Zone::Hand);
        g.state.players[0].mana = [0; 6];
        if counters == 1 {
            reject(&mut g, Action::AnnounceCast { card: spell });
            continue;
        }
        act(&mut g, Action::AnnounceCast { card: spell });
        let mut payment = Counters::default();
        payment.add(CounterKind::PlusOne, 1);
        act(
            &mut g,
            Action::ActivateMana {
                card: source,
                ability: 0,
                payment: [0; 6],
                counter_payment: payment,
                production: [0, 4, 0, 0, 0, 0],
            },
        );
        assert_eq!(g.state.players[0].mana, [0, 3, 0, 0, 0, 0]);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
    }
}

#[test]
fn a_taxed_subtype_quantity_changes_when_another_mana_cost_sacrifices_a_counted_creature() {
    const COUNT: CardRules = CardRules {
        effect: Effect::Creature(1, 1),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::CountSubtype {
                color: 4,
                subtype: "Elf",
            })],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    const SAC: CardRules = CardRules {
        effect: Effect::Creature(1, 3),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed(TWO))],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (named("Ghost Vacuum"), &TAX),
        (Card::LlanowarElves, &COUNT),
        (named("Druid of the Cowl"), &SAC),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let fuel = put(&mut g, p, "Druid of the Cowl", Zone::Battlefield);
        put(&mut g, p.other(), "Druid of the Cowl", Zone::Battlefield);
        tax(&mut g, p.other(), "Llanowar Elves");
        g.state.players[p.index()].mana = [0; 6];
        act(&mut g, announce(source, [0, 0, 0, 0, 2, 0]));
        act(&mut g, direct(fuel, TWO));
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
        assert_eq!(g.state.pending, Pending::Priority);
    }
}

#[test]
fn an_all_zero_preview_keeps_the_color_unselected_until_the_post_cost_quantity_is_known() {
    const SOURCE: CardRules = CardRules {
        effect: Effect::Creature(0, 2),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::TapSourcePowerChoice {
                mask: 31,
            })],
            ..BASE
        }],
        extras: &CardExtras {
            activation_put_counters: &[(0, CounterKind::PlusOne, 1)],
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &SOURCE)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    g.state.players[0].mana = [0; 6];
    act(&mut g, direct(source, [0; 6]));
    assert!(matches!(g.state.pending, Pending::EffectMana { .. }));
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert!(g.observe(Player::P1).mana_choices.is_empty());
    reject(
        &mut g,
        Action::ChooseMana {
            production: [0, 2, 0, 0, 0, 0],
        },
    );
    let saved = g.snapshot();
    let mut raw = g.clone();
    let mut idx = g.clone();
    let mut restored = g.clone();
    restored.restore(&saved).unwrap();
    drop(g);
    let choice = Action::ChooseMana {
        production: [0, 1, 0, 0, 0, 0],
    };
    act(&mut raw, choice.clone());
    indexed(&mut idx, choice.clone());
    act(&mut restored, choice);
    assert_eq!(raw.snapshot(), idx.snapshot());
    assert_eq!(raw.snapshot(), restored.snapshot());
    assert_eq!(raw.state.players[0].mana, [0, 1, 0, 0, 0, 0]);
    assert_eq!(raw.state.pending, Pending::Priority);
}

#[test]
fn departure_uses_the_latest_exact_power_instead_of_the_announcement_or_a_new_incarnation() {
    const SOURCE: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ActivatedAbility {
            program: &[Instruction::AddMana(ManaProduction::SourcePower {
                color: 1,
            })],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    let _scope =
        fixture_rules::activate(&[(Card::GrizzlyBears, &SOURCE), (named("Ghost Vacuum"), &TAX)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    tax(&mut g, Player::P1, "Grizzly Bears");
    g.state.players[0].mana = TWO;
    act(&mut g, announce(source, [0, 2, 0, 0, 0, 0]));
    g.state.objects[source as usize].growth.power = 3;
    g.move_object(source, Zone::Graveyard);
    g.move_object(source, Zone::Battlefield);
    g.state.objects[source as usize].growth.power = 8;
    pay(&mut g);
    assert_eq!(g.state.players[0].mana, [0, 5, 0, 0, 0, 0]);
    assert!(!g.object_view(source).tapped);
}

#[test]
fn source_departure_and_new_control_do_not_change_the_original_mana_controller() {
    const SOURCE: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ActivatedAbility {
            program: &[Instruction::AddMana(ManaProduction::CountSubtype {
                color: 4,
                subtype: "Elf",
            })],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    let _scope =
        fixture_rules::activate(&[(Card::GrizzlyBears, &SOURCE), (named("Ghost Vacuum"), &TAX)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        for _ in 0..2 {
            put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        }
        tax(&mut g, p.other(), "Grizzly Bears");
        g.state.players[p.index()].mana = TWO;
        act(&mut g, announce(source, [0, 0, 0, 0, 1, 0]));
        g.move_object(source, Zone::Graveyard);
        g.move_object(source, Zone::Battlefield);
        g.state.objects[source as usize].controller = p.other();
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
    }
}

#[test]
fn a_fixed_combination_can_be_chosen_late_without_losing_its_exact_allocation() {
    const SOURCE: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Combination {
                mask: 3,
                amount: 2,
            })],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &SOURCE)]);
    let mut g = scenario(Player::P1);
    let source = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    g.state.players[1].mana = [0; 6];
    act(&mut g, direct(source, [0; 6]));
    assert!(matches!(g.state.pending, Pending::EffectMana { .. }));
    reject(
        &mut g,
        Action::ChooseMana {
            production: [0, 0, 2, 0, 0, 0],
        },
    );
    act(
        &mut g,
        Action::ChooseMana {
            production: [1, 1, 0, 0, 0, 0],
        },
    );
    assert_eq!(g.state.players[1].mana, [1, 1, 0, 0, 0, 0]);
}

#[test]
fn a_retained_late_choice_replays_its_concrete_output_and_quality_exactly_once() {
    const COPY: CardRules = CardRules {
        effect: Effect::Creature(0, 0),
        extras: &CardExtras {
            entry_copy: Some(EntryCopySpec {
                filters: &[BattlefieldFilter {
                    kinds: 17,
                    relation: 0,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                optional: true,
            }),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    const SOURCE: CardRules = CardRules {
        effect: Effect::Creature(1, 1),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Tagged {
                production: &ManaProduction::TapSourcePowerChoice { mask: 31 },
                quality: &ManaQuality {
                    restriction: ManaRestriction::Unrestricted,
                    spell_bonus: ManaSpellBonus::None,
                    retention: ManaRetention::Turn,
                },
            })],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    let _scope =
        fixture_rules::activate(&[(Card::GrizzlyBears, &COPY), (Card::LlanowarElves, &SOURCE)]);
    let mut g = scenario(Player::P0);
    let mana = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let remaining = put(&mut g, Player::P0, "Druid of the Cowl", Zone::Battlefield);
    let enchanted = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    let blueprint = put(&mut g, Player::P1, "Witness Protection", Zone::Battlefield);
    g.state.objects[blueprint as usize].attached_to = Some(g.object_ref(enchanted));
    g.change_object_form(
        g.object_ref(blueprint),
        AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        characteristic_layers::AnimationTypeMode::PreserveCardTypes,
    );
    let returning = put(&mut g, Player::P1, "Grizzly Bears", Zone::Exile);
    g.state.exile_links.push(ExileLink {
        source: g.object_ref(mana),
        exiled: g.object_ref(returning),
        return_zone: Zone::Battlefield,
    });
    let target = put(&mut g, Player::P0, "Savannah Lions", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Pacifism", Zone::Hand);
    g.state.players[0].mana = [0; 6];
    act(&mut g, Action::AnnounceCast { card: spell });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    act(&mut g, direct(mana, [0; 6]));
    assert_eq!(g.actor(), Some(Player::P1));
    let object = g.object_ref(blueprint);
    act(
        &mut g,
        Action::ChooseEntryCopy {
            object: Some(object),
        },
    );
    let object = g.object_ref(remaining);
    act(&mut g, Action::ChooseAuraAttachment { object });
    assert!(matches!(g.state.pending, Pending::EffectMana { .. }));
    let saved = g.snapshot();
    let mut idx = g.clone();
    idx.restore(&saved).unwrap();
    let choice = Action::ChooseMana {
        production: [1, 0, 0, 0, 0, 0],
    };
    act(&mut g, choice.clone());
    indexed(&mut idx, choice);
    assert_eq!(g.snapshot(), idx.snapshot());
    assert!(matches!(g.state.pending, Pending::ManaRollback { .. }));
    let receipt = g
        .state
        .cast_rollback
        .as_ref()
        .unwrap()
        .events
        .last()
        .unwrap();
    assert_eq!(receipt.production, [1, 0, 0, 0, 0, 0]);
    assert!(receipt.quality.is_some());
    let keep = Action::ChooseManaRollback { keep: true };
    act(&mut g, keep.clone());
    indexed(&mut idx, keep);
    assert_eq!(g.snapshot(), idx.snapshot());
    assert_eq!(g.state.players[0].mana, [1, 0, 0, 0, 0, 0]);
    assert_eq!(g.object_view(mana).zone, Zone::Graveyard);
    assert_eq!(g.object_view(returning).card, named("Witness Protection"));
    assert_eq!(g.state.pending, Pending::Priority);
    g.clear_step_mana(false, false);
    assert_eq!(
        g.state.players[0].mana,
        [1, 0, 0, 0, 0, 0],
        "retained frozen quality survives replay"
    );
}

#[test]
fn removing_a_negative_counter_can_raise_power_mana_enough_to_fund_the_parent() {
    const SOURCE: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                remove: Some((Some(CounterKind::MinusOne), 1)),
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::SourcePower {
                color: 4,
            })],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &SOURCE)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.add_counter(source, CounterKind::MinusOne, 1);
        let spell = put(&mut g, p, "Druid of the Cowl", Zone::Hand);
        let wrong_color = put(&mut g, p, "Divination", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        assert_eq!(stats(&g, source), (1, 1));
        reject(&mut g, Action::AnnounceCast { card: wrong_color });
        act(&mut g, Action::AnnounceCast { card: spell });
        let mut counters = Counters::default();
        counters.add(CounterKind::MinusOne, 1);
        act(
            &mut g,
            Action::ActivateMana {
                card: source,
                ability: 0,
                payment: [0; 6],
                counter_payment: counters,
                production: [0, 0, 0, 0, 1, 0],
            },
        );
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 2, 0]);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 0);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn granted_dynamic_mana_uses_conservative_bounds_and_exact_post_cost_search() {
    const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::AttachmentMana(
            ManaProduction::SourcePower { color: 4 },
        )],
    }];
    const AURA: AuraSpec = AuraSpec {
        kinds: 1,
        relation: 0,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    };
    const GRANTOR: CardRules = CardRules {
        effect: Effect::Aura(AURA),
        aura: Some(AURA),
        static_groups: GROUPS,
        extras: &CardExtras {
            static_ability_groups: GROUPS,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    const PLAIN: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        ..CardRules::EMPTY
    };
    const REMOVE: CardRules = CardRules {
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                remove: Some((Some(CounterKind::MinusOne), 1)),
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed([0; 6]))],
            ..BASE
        }],
        ..PLAIN
    };
    for (rules, can_remove) in [(&PLAIN, false), (&REMOVE, true)] {
        let _scope =
            fixture_rules::activate(&[(Card::GrizzlyBears, rules), (named("Stop Cold"), &GRANTOR)]);
        for p in [Player::P0, Player::P1] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let grantor = put(&mut g, p, "Stop Cold", Zone::Battlefield);
            g.state.objects[grantor as usize].attached_to = Some(g.object_ref(source));
            assert_eq!(
                g.granted_mana(source).len(),
                1,
                "the fixture installs a real Aura mana grant before funding"
            );
            g.add_counter(source, CounterKind::MinusOne, 1);
            let spell = put(&mut g, p, "Druid of the Cowl", Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            if !can_remove {
                reject(&mut g, Action::AnnounceCast { card: spell });
                assert_eq!(g.counter_count(source, CounterKind::MinusOne), 1);
                continue;
            }
            act(&mut g, Action::AnnounceCast { card: spell });
            let mut counters = Counters::default();
            counters.add(CounterKind::MinusOne, 1);
            act(
                &mut g,
                Action::ActivateMana {
                    card: source,
                    ability: 0,
                    payment: [0; 6],
                    counter_payment: counters,
                    production: [0; 6],
                },
            );
            let grantor = g.object_ref(grantor);
            act(
                &mut g,
                Action::TapGrantedMana {
                    card: source,
                    grantor,
                    production: [0, 0, 0, 0, 2, 0],
                },
            );
            assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 2, 0]);
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
