//! Named generic activation increases across real and projected payment routes.
//! Dev-only immutable providers retain each fixture's physical card metadata.
mod support;
use rustmage::Card;
use rustmage_core::CardName;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const TWO: Mana = [0, 0, 0, 0, 0, 2];
const BLUE: Mana = [0, 3, 0, 0, 0, 0];
const COUNTER: &[Instruction] = &[Instruction::PutCounters {
    subject: Subject::Source,
    kind: CounterKind::PlusOne,
    amount: 1,
}];
const fn ability(cost: ActivationCost, program: &'static [Instruction]) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost,
        targets: TargetPlan::NONE,
        program,
        sorcery: false,
        once_per_turn: false,
    }
}
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
const SOURCE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ability(ActivationCost::ZERO, COUNTER)],
    ..CardRules::EMPTY
};
const PLAIN: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    ..CardRules::EMPTY
};
const MANA: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    activated: &[ability(
        ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        &[Instruction::AddMana(ManaProduction::Fixed(TWO))],
    )],
    ..CardRules::EMPTY
};
fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn tax(g: &mut Game, owner: Player, name: &str) -> CardId {
    let id = put(g, owner, "Ghost Vacuum", Zone::Battlefield);
    g.set_entry_value(
        g.object_ref(id),
        EntryChoiceValue::CardName(CardName::from_name(name).unwrap()),
    );
    id
}
fn start(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn activate_mana(card: CardId, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}

#[test]
fn named_tax_applies_to_both_controllers_and_a_zero_mana_printed_activation() {
    let _scope = fixture_rules::activate(&[
        (named("Ghost Vacuum"), &TAX),
        (Card::GrizzlyBears, &SOURCE),
        (Card::GoblinPiker, &SOURCE),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let other = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        tax(&mut g, p.other(), "Grizzly Bears");
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, start(source));
        assert!(g.legal_actions().contains(&start(other)));
        g.state.players[p.index()].mana = TWO;
        act(&mut g, start(source));
        assert_eq!(g.mana_payment_domain().unwrap().generic, 2);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn increase_precedes_reduction_and_the_raw_alternative_receipt_survives_owned_branches() {
    const REDUCTIONS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::OtherControlledExhaustGenericReduction(2)],
    }];
    const REDUCER: CardRules = CardRules {
        effect: Effect::Creature(2, 1),
        static_groups: REDUCTIONS,
        extras: &CardExtras {
            static_ability_groups: REDUCTIONS,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    const ALTERNATIVES: CardRules = CardRules {
        activated: &[ActivatedAbility {
            exhaust: true,
            cost: ActivationCost {
                generic: 1,
                alternatives: &[ManaCost {
                    colored: [0; 6],
                    generic: 5,
                }],
                mana_component: true,
                ..ActivationCost::ZERO
            },
            ..ability(ActivationCost::ZERO, COUNTER)
        }],
        extras: &CardExtras {
            exhaust_mask: 1,
            ..CardExtras::EMPTY
        },
        ..SOURCE
    };
    let _scope = fixture_rules::activate(&[
        (named("Ghost Vacuum"), &TAX),
        (Card::GrizzlyBears, &ALTERNATIVES),
        (Card::GoblinPiker, &REDUCER),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let reducer = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    let provider = tax(&mut g, Player::P0, "Grizzly Bears");
    g.state.players[0].mana = [0, 0, 0, 0, 0, 5];
    act(&mut g, start(source));
    let options: Vec<_> = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cost_options()
        .collect();
    assert_eq!(
        options.iter().map(|c| c.generic).collect::<Vec<_>>(),
        vec![1, 5]
    );
    act(&mut g, Action::ChooseActivationCost { cost: 1 });
    let a = g.state.announcing.as_ref().unwrap();
    assert_eq!(a.activation_choice.as_ref().unwrap().base.generic, 5);
    assert_eq!(a.total_cost.unwrap().generic, 5);
    let saved = g.snapshot();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    let mut restored = g.clone();
    restored.restore(&saved).unwrap();
    drop(g);
    for (n, branch) in [&mut raw, &mut indexed, &mut restored]
        .into_iter()
        .enumerate()
    {
        branch.move_object(provider, Zone::Graveyard);
        branch.move_object(reducer, Zone::Graveyard);
        assert_eq!(
            branch.mana_payment_domain().unwrap().generic,
            5,
            "the total is already locked"
        );
        let actions = branch.legal_actions();
        let i = actions
            .iter()
            .position(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        if n == 1 {
            branch
                .apply_index(branch.actor().unwrap(), branch.revision(), i)
                .unwrap();
        } else {
            act(branch, actions[i].clone());
        }
        settle(branch);
        assert_eq!(branch.counter_count(source, CounterKind::PlusOne), 1);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.snapshot(), restored.snapshot());
}

#[test]
fn x_is_added_with_the_increase_before_a_counter_based_reduction_saturates() {
    const X_SOURCE: CardRules = CardRules {
        activated: &[ability(
            ActivationCost::ZERO,
            &[
                Instruction::ActivationX,
                Instruction::PutCountersScaled {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: AmountValue::ChosenX,
                },
            ],
        )],
        extras: &CardExtras {
            activation_counter_discounts: &[(0, CounterKind::Charge)],
            ..CardExtras::EMPTY
        },
        ..SOURCE
    };
    let _scope = fixture_rules::activate(&[
        (named("Ghost Vacuum"), &TAX),
        (Card::GrizzlyBears, &X_SOURCE),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    tax(&mut g, Player::P0, "Grizzly Bears");
    g.add_counter(source, CounterKind::Charge, 5);
    g.state.players[0].mana = TWO;
    act(&mut g, start(source));
    assert_eq!(g.x_choice().unwrap().intervals[0].maximum, 5);
    act(&mut g, Action::ChooseX { value: 5 });
    assert_eq!(g.mana_payment_domain().unwrap().generic, 2);
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 5);
}

#[test]
fn hand_and_graveyard_activations_use_owner_authority_and_their_own_source_name() {
    const HAND: CardRules = CardRules {
        activated: &[ActivatedAbility {
            origin: ActivationOrigin::Hand,
            cost: ActivationCost {
                discard: true,
                ..ActivationCost::ZERO
            },
            ..ability(
                ActivationCost::ZERO,
                &[Instruction::Draw(PlayerSet::Controller, 1)],
            )
        }],
        ..SOURCE
    };
    const GRAVE: CardRules = CardRules {
        activated: &[ActivatedAbility {
            origin: ActivationOrigin::Graveyard,
            cost: ActivationCost {
                exile_self: true,
                ..ActivationCost::ZERO
            },
            ..ability(
                ActivationCost::ZERO,
                &[Instruction::Draw(PlayerSet::Controller, 1)],
            )
        }],
        ..SOURCE
    };
    for (origin, rules) in [
        (ActivationOrigin::Hand, &HAND),
        (ActivationOrigin::Graveyard, &GRAVE),
    ] {
        let _scope =
            fixture_rules::activate(&[(named("Ghost Vacuum"), &TAX), (Card::GrizzlyBears, rules)]);
        let mut g = scenario(Player::P1);
        let zone = if origin == ActivationOrigin::Hand {
            Zone::Hand
        } else {
            Zone::Graveyard
        };
        let source = put(&mut g, Player::P1, "Grizzly Bears", zone);
        g.state.objects[source as usize].controller = Player::P0;
        tax(&mut g, Player::P0, "Grizzly Bears");
        g.state.players[1].mana = TWO;
        act(&mut g, start(source));
        assert_eq!(g.mana_payment_domain().unwrap().generic, 2);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[1].cards_drawn_this_turn, 1);
        assert_eq!(
            g.object_view(source).zone,
            if zone == Zone::Hand {
                Zone::Graveyard
            } else {
                Zone::Exile
            }
        );
    }
}

#[test]
fn intrinsic_mana_pays_the_named_tax_through_a_real_mana_window() {
    let _scope = fixture_rules::activate(&[(named("Ghost Vacuum"), &TAX)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    tax(&mut g, Player::P1, "Llanowar Elves");
    g.state.players[0].mana = [0; 6];
    reject(&mut g, Action::TapMana { card: source });
    g.state.players[0].mana = TWO;
    choose(
        &mut g,
        |a| matches!(a, Action::AnnounceMana { card, ability, .. } if *card == source && *ability > 60000),
    );
    assert_eq!(g.mana_payment_domain().unwrap().generic, 2);
    pay(&mut g);
    assert!(g.object_view(source).tapped);
    assert!(g.state.stack.is_empty());
    assert_eq!(g.state.players[0].mana, [0, 0, 0, 0, 1, 0]);
}

#[test]
fn a_granted_mana_packet_keeps_quality_and_cost_after_its_provider_leaves_during_nested_payment() {
    const GRANTS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::AttachmentMana(
            ManaProduction::Tagged {
                production: &ManaProduction::Fixed([8, 0, 0, 0, 0, 0]),
                quality: &ManaQuality {
                    restriction: ManaRestriction::ArtifactSpellOrActivation,
                    spell_bonus: ManaSpellBonus::None,
                    retention: ManaRetention::Step,
                },
            },
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
        effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Creature))),
        aura: Some(AURA),
        static_groups: GRANTS,
        activated: &[ability(
            ActivationCost {
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            &[Instruction::AddMana(ManaProduction::Fixed(TWO))],
        )],
        extras: &CardExtras {
            static_ability_groups: GRANTS,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (named("Ghost Vacuum"), &TAX),
        (Card::GrizzlyBears, &PLAIN),
        (named("Stop Cold"), &GRANTOR),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let grantor = put(&mut g, Player::P0, "Stop Cold", Zone::Battlefield);
    g.state.objects[grantor as usize].attached_to = Some(g.object_ref(source));
    assert_eq!(
        g.granted_mana(source).len(),
        1,
        "the fixture installs a real Aura mana grant before funding"
    );
    tax(&mut g, Player::P1, "Grizzly Bears");
    let spell = put(&mut g, Player::P0, "Broadcast Rambler", Zone::Hand);
    g.state.players[0].mana = [0; 6];
    act(&mut g, Action::AnnounceCast { card: spell });
    let action = Action::AnnounceGrantedMana {
        card: source,
        grantor: g.object_ref(grantor),
        production: [8, 0, 0, 0, 0, 0],
    };
    act(&mut g, action);
    assert_eq!(
        g.state
            .announcing
            .as_ref()
            .unwrap()
            .total_cost
            .unwrap()
            .generic,
        2
    );
    act(&mut g, activate_mana(grantor, TWO));
    assert_eq!(g.object_view(grantor).zone, Zone::Graveyard);
    assert!(g.granted_mana(source).is_empty());
    pay(&mut g);
    assert!(g.object_view(source).tapped);
    assert_eq!(
        g.available_mana(Player::P0, mana_lots::Purpose::Other),
        [0; 6]
    );
    assert_eq!(g.state.players[0].mana, [8, 0, 0, 0, 0, 0]);
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
    assert_eq!(
        g.available_mana(Player::P0, mana_lots::Purpose::Other),
        [0; 6]
    );
}

#[test]
fn borrowed_mana_uses_the_borrowers_name_and_preserves_donor_slot_identity() {
    const BORROW: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::BorrowGraveyardActivations("Elf")],
    }];
    const BORROWER: CardRules = CardRules {
        static_groups: BORROW,
        extras: &CardExtras {
            static_ability_groups: BORROW,
            ..CardExtras::EMPTY
        },
        ..PLAIN
    };
    const DONOR: CardRules = CardRules {
        activated: &[ability(
            ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            &[Instruction::AddMana(ManaProduction::Fixed(BLUE))],
        )],
        ..MANA
    };
    let _scope = fixture_rules::activate(&[
        (named("Ghost Vacuum"), &TAX),
        (Card::GrizzlyBears, &BORROWER),
        (Card::LlanowarElves, &DONOR),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let donor = put(&mut g, Player::P0, "Llanowar Elves", Zone::Graveyard);
    tax(&mut g, Player::P1, "Grizzly Bears");
    g.state.players[0].mana = TWO;
    g.refresh_graveyard_borrowing();
    let donor_reference = g.object_ref(donor);
    act(
        &mut g,
        Action::AnnounceGraveyardBorrowedMana {
            card: source,
            graveyard: donor_reference,
            ability: 0,
            counter_payment: Counters::default(),
            production: BLUE,
        },
    );
    let a = g.state.announcing.as_ref().unwrap();
    assert_eq!(a.item.source.unwrap().card, Card::LlanowarElves);
    assert!(a.item.borrowed_slot.is_some());
    assert!(a.item.attachment_trigger.is_none());
    assert_eq!(g.mana_payment_domain().unwrap().generic, 2);
    pay(&mut g);
    assert_eq!(g.state.players[0].mana, BLUE);
    assert!(g.object_view(source).tapped);
}

#[test]
fn independent_self_granted_mana_instances_pay_tax_and_keep_once_per_turn_history_in_nested_frames()
{
    const SELF: CardRules = CardRules {
        activated: &[ActivatedAbility {
            once_per_turn: true,
            ..ability(
                ActivationCost::ZERO,
                &[Instruction::AddMana(ManaProduction::Fixed(BLUE))],
            )
        }],
        extras: &CardExtras {
            self_granted_activation_templates: &[0],
            ..CardExtras::EMPTY
        },
        ..PLAIN
    };
    let _scope =
        fixture_rules::activate(&[(named("Ghost Vacuum"), &TAX), (Card::GrizzlyBears, &SELF)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let item = g.activation_packet(Player::P0, source, 0);
    g.grant_self_ability(&item, SelfGrantedAbilityKind::Activated { ability: 0 });
    g.grant_self_ability(&item, SelfGrantedAbilityKind::Activated { ability: 0 });
    let rows = g.self_granted_ability_views().unwrap();
    let stamps: Vec<_> = rows.iter().map(|r| r.timestamp).collect();
    assert_eq!(stamps.len(), 2);
    tax(&mut g, Player::P1, "Grizzly Bears");
    let spell = put(&mut g, Player::P0, "Divination", Zone::Hand);
    g.state.players[0].mana = TWO;
    act(&mut g, Action::AnnounceCast { card: spell });
    let first = Action::AnnounceSelfGrantedMana {
        source: g.object_ref(source),
        grant_timestamp: stamps[0],
        counter_payment: Counters::default(),
        production: BLUE,
    };
    act(&mut g, first.clone());
    pay(&mut g);
    reject(&mut g, first);
    let source_reference = g.object_ref(source);
    act(
        &mut g,
        Action::AnnounceSelfGrantedMana {
            source: source_reference,
            grant_timestamp: stamps[1],
            counter_payment: Counters::default(),
            production: BLUE,
        },
    );
    pay(&mut g);
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
    assert_eq!(g.state.players[0].mana, [0, 1, 0, 0, 0, 0]);
}

#[test]
fn crew_and_saddle_reserve_selected_contributors_against_mana_and_tap_only_after_all_costs() {
    let _scope = fixture_rules::activate(&[
        (named("Ghost Vacuum"), &TAX),
        (Card::GrizzlyBears, &PLAIN),
        (Card::LlanowarElves, &MANA),
    ]);
    for (name, crew) in [("Broadcast Rambler", true), ("Brightfield Mustang", false)] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, name, Zone::Battlefield);
        let mana_creature = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
        tax(&mut g, Player::P1, name);
        g.state.players[0].mana = [0; 6];
        reject(&mut g, start(source));
        let participant = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        act(&mut g, start(source));
        let wrong = if crew {
            Action::ChooseCrewer {
                object: g.object_ref(mana_creature),
            }
        } else {
            Action::ChooseSaddler {
                object: g.object_ref(mana_creature),
            }
        };
        reject(&mut g, wrong);
        let choose = if crew {
            Action::ChooseCrewer {
                object: g.object_ref(participant),
            }
        } else {
            Action::ChooseSaddler {
                object: g.object_ref(participant),
            }
        };
        act(&mut g, choose);
        act(
            &mut g,
            if crew {
                Action::FinishCrew
            } else {
                Action::FinishSaddle
            },
        );
        assert!(matches!(g.state.pending, Pending::PayCosts { .. }));
        assert!(!g.object_view(participant).tapped);
        assert_eq!(g.observe(Player::P1).crew_choice, None);
        act(&mut g, activate_mana(mana_creature, TWO));
        assert!(!g.object_view(participant).tapped);
        pay(&mut g);
        assert!(g.object_view(participant).tapped);
        settle(&mut g);
        assert!(if crew {
            g.is_creature(source)
        } else {
            g.is_saddled(g.object_ref(source))
        });
    }
}

#[test]
fn suppressing_the_provider_removes_the_tax_and_aura_departure_restores_it() {
    const CREATURE_TAX: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        ..TAX
    };
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &CREATURE_TAX),
        (Card::GoblinPiker, &SOURCE),
    ]);
    let mut g = scenario(Player::P0);
    let provider = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let source = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    g.set_entry_value(
        g.object_ref(provider),
        EntryChoiceValue::CardName(CardName::from_name("Goblin Piker").unwrap()),
    );
    let aura = put(&mut g, Player::P0, "Stop Cold", Zone::Hand);
    cast(&mut g, aura, Some(Target::Card(provider)));
    settle(&mut g);
    g.state.players[0].mana = [0; 6];
    assert_eq!(g.named_activation_generic_increase(source), 0);
    act(&mut g, start(source));
    pay(&mut g);
    settle(&mut g);
    g.move_object(aura, Zone::Graveyard);
    assert_eq!(g.named_activation_generic_increase(source), 2);
    reject(&mut g, start(source));
}

#[test]
fn source_max_speed_graveyard_mana_pays_the_tax_atomically_and_keeps_owner_authority() {
    const MAX: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::SourceMaxSpeedActivation(0)],
    }];
    const GRAVE_MANA: CardRules = CardRules {
        static_groups: MAX,
        activated: &[ActivatedAbility {
            origin: ActivationOrigin::Graveyard,
            ..ability(
                ActivationCost {
                    exile_self: true,
                    ..ActivationCost::ZERO
                },
                &[Instruction::AddMana(ManaProduction::Fixed(BLUE))],
            )
        }],
        extras: &CardExtras {
            static_ability_groups: MAX,
            ..CardExtras::EMPTY
        },
        ..PLAIN
    };
    let _scope = fixture_rules::activate(&[
        (named("Ghost Vacuum"), &TAX),
        (Card::GrizzlyBears, &GRAVE_MANA),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    g.state.objects[source as usize].controller = Player::P1;
    g.state.players[0].speed = 4;
    tax(&mut g, Player::P1, "Grizzly Bears");
    let spell = put(&mut g, Player::P0, "Divination", Zone::Hand);
    g.state.players[0].mana = TWO;
    act(&mut g, Action::AnnounceCast { card: spell });
    act(
        &mut g,
        Action::AnnounceMana {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
            production: BLUE,
        },
    );
    assert_eq!(g.mana_payment_domain().unwrap().generic, 2);
    pay(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Exile);
    assert_eq!(g.state.creatures_died_this_turn, 0);
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
    assert_eq!(g.state.players[0].mana, [0; 6]);
}

#[test]
fn tax_total_locks_after_the_last_target_before_mana_removes_its_provider() {
    const TAX_MANA: CardRules = CardRules {
        activated: &[ability(
            ActivationCost {
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            &[Instruction::AddMana(ManaProduction::Fixed(TWO))],
        )],
        ..TAX
    };
    const TARGET_SOURCE: CardRules = CardRules {
        activated: &[ActivatedAbility {
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            ..ability(
                ActivationCost::ZERO,
                &[Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                }],
            )
        }],
        ..PLAIN
    };
    let _scope = fixture_rules::activate(&[
        (named("Ghost Vacuum"), &TAX_MANA),
        (Card::GrizzlyBears, &TARGET_SOURCE),
        (Card::GoblinPiker, &PLAIN),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let target = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    let provider = tax(&mut g, Player::P0, "Grizzly Bears");
    g.state.players[0].mana = [0; 6];
    act(&mut g, start(source));
    assert!(g.state.announcing.as_ref().unwrap().total_cost.is_none());
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    assert_eq!(
        g.state
            .announcing
            .as_ref()
            .unwrap()
            .total_cost
            .unwrap()
            .generic,
        2
    );
    act(&mut g, activate_mana(provider, TWO));
    assert_eq!(g.named_activation_generic_increase(source), 0);
    assert_eq!(g.mana_payment_domain().unwrap().generic, 2);
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.counter_count(target, CounterKind::PlusOne), 1);
}

#[test]
fn a_current_copied_name_controls_the_tax_instead_of_the_source_definition() {
    let _scope =
        fixture_rules::activate(&[(named("Ghost Vacuum"), &TAX), (Card::GrizzlyBears, &SOURCE)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    tax(&mut g, Player::P1, "Goblin Piker");
    g.state.players[0].mana = [0; 6];
    assert!(g.legal_actions().contains(&start(source)));
    let mut recipe = g.copiable_recipe(g.object_ref(source));
    recipe.overrides = Some(CopyOverrides {
        name: Some("Goblin Piker"),
        ..CopyOverrides::NONE
    });
    g.set_copy_recipe(g.object_ref(source), recipe);
    assert_eq!(g.object_name(source), "Goblin Piker");
    assert_eq!(g.named_activation_generic_increase(source), 2);
    reject(&mut g, start(source));
    g.state.players[0].mana = TWO;
    act(&mut g, start(source));
    assert_eq!(
        g.state
            .announcing
            .as_ref()
            .unwrap()
            .item
            .source
            .unwrap()
            .card,
        Card::GrizzlyBears
    );
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
}

#[test]
fn a_split_source_in_hand_matches_either_structured_rules_name() {
    const HAND: CardRules = CardRules {
        activated: &[ActivatedAbility {
            origin: ActivationOrigin::Hand,
            ..ability(
                ActivationCost {
                    discard: true,
                    ..ActivationCost::ZERO
                },
                &[Instruction::Draw(PlayerSet::Controller, 1)],
            )
        }],
        ..CardRules::EMPTY
    };
    let split = named("Cease // Desist");
    let _scope = fixture_rules::activate(&[(named("Ghost Vacuum"), &TAX), (split, &HAND)]);
    for chosen in ["Cease", "Desist"] {
        let mut g = scenario(Player::P0);
        // This is a synthetic engine ability on a real split card's physical
        // identity/types/cost; no catalogue support flag is changed.
        let source = put(&mut g, Player::P0, "Divination", Zone::Hand);
        g.state.objects[source as usize].card = split;
        tax(&mut g, Player::P1, chosen);
        g.state.players[0].mana = TWO;
        assert_eq!(g.named_activation_generic_increase(source), 2);
        act(&mut g, start(source));
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    }
}

#[test]
fn a_projected_new_incarnation_uses_the_same_power_up_reduction_and_tax_as_actual_entry() {
    use std::collections::HashSet;
    const POWER_UP_MANA: CardRules = CardRules {
        activated: &[ActivatedAbility {
            once_per_turn: true,
            ..ability(
                ActivationCost {
                    generic: 2,
                    mana_component: true,
                    ..ActivationCost::ZERO
                },
                &[Instruction::AddMana(ManaProduction::Fixed(BLUE))],
            )
        }],
        extras: &CardExtras {
            power_up_abilities: &[0],
            ..CardExtras::EMPTY
        },
        ..PLAIN
    };
    let _scope = fixture_rules::activate(&[
        (named("Ghost Vacuum"), &TAX),
        (Card::GrizzlyBears, &POWER_UP_MANA),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    tax(&mut g, Player::P1, "Grizzly Bears");
    g.state.players[0].mana = TWO;
    let goal = FundingGoal {
        actor: Player::P0,
        required: [0, 1, 0, 0, 0, 0],
        generic: 2,
        source: None,
        reserve: None,
    };
    let world = FundingState::from_game(&g, &goal);
    let movement = g.entry_move(source, Zone::Battlefield, false);
    assert!(
        world.visit_entry_moves(vec![movement], true, None, &mut |next| {
            assert!(next.entered_this_turn(next.reference(source)));
            // Generic2 + tax2 - the newly entered Bear's {1}{G} = generic2.
            // It produces three blue, exactly enough for this final payment.
            next.reachable(&goal, &mut HashSet::new())
        })
    );
    g.move_object(source, Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 0, 1, 1];
    cast(&mut g, source, None);
    settle(&mut g);
    assert!(g.entered_this_turn(g.object_ref(source)));
    g.state.players[0].mana = TWO;
    act(
        &mut g,
        Action::AnnounceMana {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
            production: BLUE,
        },
    );
    assert_eq!(g.mana_payment_domain().unwrap().generic, 2);
    pay(&mut g);
    assert_eq!(g.state.players[0].mana, BLUE);
}

#[test]
fn a_copied_vehicle_sacrificed_for_its_tax_uses_its_exact_departure_subtype_after_slot_reuse() {
    const VEHICLE: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[ability(
            ActivationCost::ZERO,
            &[Instruction::Crew { minimum: 4 }],
        )],
        ..CardRules::EMPTY
    };
    const PILOT: CardRules = CardRules {
        keywords: Keywords::CREW_SADDLE_PLUS_TWO,
        ..PLAIN
    };
    const FUEL: CardRules = CardRules {
        activated: &[ability(
            ActivationCost {
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::Sacrifice,
                    filter: SacrificeFilter {
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 16,
                        nonland: false,
                        other: true,
                        tokens: None,
                        artifact_enchantment_or_token: false,
                        subtype: None,
                    },
                }),
                ..ActivationCost::ZERO
            },
            &[Instruction::AddMana(ManaProduction::Fixed(TWO))],
        )],
        ..MANA
    };
    let _scope = fixture_rules::activate(&[
        (named("Ghost Vacuum"), &TAX),
        (named("Broadcast Rambler"), &VEHICLE),
        (Card::GrizzlyBears, &PLAIN),
        (Card::GoblinPiker, &PILOT),
        (Card::LlanowarElves, &FUEL),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    g.apply_temporary_copy(
        g.object_ref(source),
        CopyRecipe::new(named("Broadcast Rambler")),
        CopyDeadline::EndOfTurn,
    );
    let old_source = g.object_ref(source);
    let pilot = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    let fuel = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    tax(&mut g, Player::P1, "Broadcast Rambler");
    g.state.players[0].mana = [0; 6];
    act(&mut g, start(source));
    let participant = g.object_ref(pilot);
    act(
        &mut g,
        Action::ChooseCrewer {
            object: participant,
        },
    );
    act(&mut g, Action::FinishCrew);
    act(
        &mut g,
        Action::AnnounceMana {
            card: fuel,
            ability: 0,
            counter_payment: Counters::default(),
            production: TWO,
        },
    );
    act(&mut g, Action::ChooseAdditionalCost { object: old_source });
    pay(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert_eq!(g.state.objects[source as usize].card, Card::GrizzlyBears);
    // A returned physical Bear is a new source incarnation. It supplies neither
    // the old Vehicle qualifier nor an automatically selected contribution.
    g.move_object(source, Zone::Battlefield);
    assert_ne!(g.object_ref(source), old_source);
    assert!(!g.has_subtype(source, "Vehicle"));
    pay(&mut g);
    let crew = g.state.stack.iter().find(|s| s.crew.is_some()).unwrap();
    assert_eq!(crew.source.unwrap().object, old_source);
    assert_eq!(crew.crew.as_ref().unwrap().paid_power, Some(4));
    assert_eq!(crew.crew.as_ref().unwrap().contributors, vec![participant]);
    assert!(g.object_view(pilot).tapped);
}

#[test]
fn losing_vehicle_before_departure_does_not_keep_the_announcement_subtype_for_assistance() {
    const VEHICLE: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[ability(
            ActivationCost::ZERO,
            &[Instruction::Crew { minimum: 4 }],
        )],
        ..CardRules::EMPTY
    };
    const PILOT: CardRules = CardRules {
        keywords: Keywords::CREW_SADDLE_PLUS_TWO,
        ..PLAIN
    };
    let _scope = fixture_rules::activate(&[
        (named("Ghost Vacuum"), &TAX),
        (named("Broadcast Rambler"), &VEHICLE),
        (Card::GrizzlyBears, &PLAIN),
        (Card::GoblinPiker, &PILOT),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    g.apply_temporary_copy(
        g.object_ref(source),
        CopyRecipe::new(named("Broadcast Rambler")),
        CopyDeadline::EndOfTurn,
    );
    let original = g.object_ref(source);
    let pilot = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    tax(&mut g, Player::P1, "Broadcast Rambler");
    g.state.players[0].mana = TWO;
    act(&mut g, start(source));
    let participant = g.object_ref(pilot);
    act(
        &mut g,
        Action::ChooseCrewer {
            object: participant,
        },
    );
    act(&mut g, Action::FinishCrew);
    // Model a later legitimate copy/type change, followed by departure and slot
    // reuse. Creature-subtype grants deliberately cannot manufacture Vehicle.
    g.apply_temporary_copy(
        original,
        CopyRecipe::new(Card::GrizzlyBears),
        CopyDeadline::EndOfTurn,
    );
    assert!(!g.has_subtype(source, "Vehicle"));
    g.move_object(source, Zone::Graveyard);
    g.move_object(source, Zone::Battlefield);
    assert_ne!(g.object_ref(source), original);
    g.apply_temporary_copy(
        g.object_ref(source),
        CopyRecipe::new(named("Broadcast Rambler")),
        CopyDeadline::EndOfTurn,
    );
    assert!(g.has_subtype(source, "Vehicle"));
    assert!(
        !g.state
            .announcing
            .as_ref()
            .unwrap()
            .item
            .source
            .unwrap()
            .facts
            .has_subtype("Vehicle")
    );
    assert!(
        g.mana_payment_domain().is_none(),
        "latest departure, not old announcement or new incarnation, controls the assistance"
    );
    reject(&mut g, Action::PayCosts { payment: TWO });
    assert!(!g.object_view(pilot).tapped);
    assert_eq!(g.state.players[0].mana, TWO);
}

#[test]
fn exact_selected_assistance_keeps_the_bonus_above_i128_until_negative_terms_cancel() {
    const VEHICLE: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[ability(
            ActivationCost::ZERO,
            &[Instruction::Crew { minimum: 3 }],
        )],
        ..CardRules::EMPTY
    };
    const PILOT: CardRules = CardRules {
        keywords: Keywords::CREW_SADDLE_PLUS_TWO,
        ..PLAIN
    };
    let _scope = fixture_rules::activate(&[
        (named("Ghost Vacuum"), &TAX),
        (named("Broadcast Rambler"), &VEHICLE),
        (Card::GrizzlyBears, &PILOT),
        (Card::GoblinPiker, &PLAIN),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Broadcast Rambler", Zone::Battlefield);
    let pilot = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let negative = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    g.state.objects[pilot as usize].growth.power = i128::MAX - 2;
    g.state.objects[negative as usize].growth.power = -i128::MAX;
    tax(&mut g, Player::P1, "Broadcast Rambler");
    g.state.players[0].mana = TWO;
    act(&mut g, start(source));
    let first = g.object_ref(pilot);
    let second = g.object_ref(negative);
    act(&mut g, Action::ChooseCrewer { object: first });
    act(&mut g, Action::ChooseCrewer { object: second });
    assert_eq!(g.crew_choice(Player::P0).unwrap().selected_power, Some(4));
    act(&mut g, Action::FinishCrew);
    pay(&mut g);
    assert_eq!(
        g.state
            .stack
            .last()
            .unwrap()
            .crew
            .as_ref()
            .unwrap()
            .paid_power,
        Some(4)
    );
}
