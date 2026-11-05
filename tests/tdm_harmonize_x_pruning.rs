//! Conservative mana capacity counts alternative ways to tap one exact source
//! once. Exact funding still determines X, colors and shared cost participants.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use std::time::Instant;
use support::*;

const ZERO: Mana = [0; 6];
const U3: Mana = [0, 3, 0, 0, 0, 0];
const BASE: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost::ZERO,
    targets: TargetPlan::NONE,
    program: &[],
    sorcery: false,
    once_per_turn: false,
};
const REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::CreaturePowerCount { minimum: 4 },
    turn: CostTurn::Own,
    guard: StepGuard::Always,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
}];
const PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        cost_reductions: &[REDUCTION],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const CONVERTER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::Choice {
            mask: 26,
            amount: 1,
        })],
        once_per_turn: true,
        ..BASE
    }],
    ..CardRules::EMPTY
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
        x_symbols: 1,
        harmonize_x_symbols: 1,
        harmonize_cost: Some(ManaCost {
            colored: [0, 0, 0, 0, 4, 0],
            generic: 0,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
fn goal(p: Player, generic: u32) -> FundingGoal {
    FundingGoal {
        actor: p,
        required: ZERO,
        generic,
        reserve: None,
        source: None,
    }
}
struct Root {
    game: Game,
    spell: CardId,
    providers: [CardId; 2],
    converter: CardId,
    lands: Vec<CardId>,
    secret: CardId,
}
fn root(p: Player) -> Root {
    let mut g = scenario(p);
    let mut lands = Vec::new();
    for name in [
        "Forest",
        "Forest",
        "Frontier Bivouac",
        "Frontier Bivouac",
        "Island",
        "Mountain",
        "Mountain",
    ] {
        lands.push(put(&mut g, p, name, Zone::Battlefield));
    }
    let providers = [
        put(&mut g, p, "Grizzly Bears", Zone::Battlefield),
        put(&mut g, p, "Grizzly Bears", Zone::Battlefield),
    ];
    let converter = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    let secret = put(&mut g, p, "Cancel", Zone::Hand);
    let spell = put(&mut g, p, "Spectral Denial", Zone::Graveyard);
    g.state.players[p.index()].mana = ZERO;
    Root {
        game: g,
        spell,
        providers,
        converter,
        lands,
        secret,
    }
}
fn scope() -> fixture_rules::Guard {
    fixture_rules::activate(&[
        (Card::GrizzlyBears, &PROVIDER),
        (Card::GoblinPiker, &CONVERTER),
        (Card::from_name("Spectral Denial").unwrap(), &X),
    ])
}
fn x_root(p: Player) -> Root {
    let mut r = root(p);
    act(&mut r.game, Action::AnnounceHarmonize { card: r.spell });
    assert!(matches!(r.game.state.pending, Pending::X { .. }));
    r
}

#[test]
fn pruning_root_shape_seven_physical_taps_have_capacity_seven() {
    let _rules = scope();
    for p in [Player::P0, Player::P1] {
        let r = root(p);
        let world = FundingState::from_game(&r.game, &goal(p, 8));
        assert_eq!(
            world.total_mana_upper(),
            7,
            "two tri-lands cannot each pay their intrinsic and printed tap alternatives"
        );
        assert!(!world.optimistic(&goal(p, 8)));
    }
}

#[test]
fn pruning_root_shape_keeps_full_x_domain_and_exact_helper_frontier() {
    let _rules = scope();
    for p in [Player::P0, Player::P1] {
        let r = x_root(p);
        let start = Instant::now();
        let domain = r.game.x_choice().unwrap();
        eprintln!(
            "root_x_domain player={p:?} seconds={:.9}",
            start.elapsed().as_secs_f64()
        );
        assert_eq!(
            domain.intervals,
            vec![XInterval {
                minimum: 0,
                maximum: 11
            }]
        );
        for (x, expected) in [
            (
                10,
                vec![
                    Some(r.providers[0]),
                    Some(r.providers[1]),
                    Some(r.converter),
                ],
            ),
            (11, vec![Some(r.providers[0]), Some(r.providers[1])]),
        ] {
            let mut a = r.game.state.announcing.as_ref().unwrap().clone();
            a.item.x = Some(x);
            let start = Instant::now();
            let actual = [
                None,
                Some(r.providers[0]),
                Some(r.providers[1]),
                Some(r.converter),
            ]
            .into_iter()
            .filter(|&id| {
                r.game
                    .harmonize_choice_fundable(&a, id.map(|id| r.game.object_ref(id)))
            })
            .collect::<Vec<_>>();
            eprintln!(
                "root_helper_domain player={p:?} x={x} seconds={:.9}",
                start.elapsed().as_secs_f64()
            );
            assert_eq!(actual, expected);
        }
    }
}

#[test]
fn pruning_intrinsic_and_printed_tap_options_share_one_capacity_and_still_pay_three() {
    const SOURCE: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed(U3))],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[(Card::Forest, &SOURCE)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Forest", Zone::Battlefield);
        let spell = put(&mut g, p, "Divination", Zone::Hand);
        g.state.players[p.index()].mana = ZERO;
        let world = FundingState::from_game(&g, &goal(p, 4));
        assert_eq!(world.total_mana_upper(), 3);
        act(&mut g, Action::AnnounceCast { card: spell });
        act(
            &mut g,
            Action::ActivateMana {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
                payment: ZERO,
                production: U3,
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
    }
}

#[test]
fn pruning_non_tap_sacrifice_can_follow_intrinsic_tap_and_remains_additive() {
    const SOURCE: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 2, 0, 0, 0, 0,
            ]))],
            once_per_turn: true,
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[(Card::Forest, &SOURCE)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Forest", Zone::Battlefield);
        let spell = put(&mut g, p, "Divination", Zone::Hand);
        g.state.players[p.index()].mana = ZERO;
        assert_eq!(
            FundingState::from_game(&g, &goal(p, 3)).total_mana_upper(),
            3,
            "intrinsic tap and a separate non-tap sacrifice can both contribute"
        );
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::TapMana { card: source });
        act(
            &mut g,
            Action::ActivateMana {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
                payment: ZERO,
                production: [0, 2, 0, 0, 0, 0],
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(g.state.players[p.index()].mana, ZERO);
    }
}

#[test]
fn pruning_real_repeatable_positive_engine_retains_its_unbounded_upper() {
    const SOURCE: CardRules = CardRules {
        effect: Effect::Creature(2, 1),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                generic: 1,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 2, 0, 0, 0, 0,
            ]))],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[(Card::GoblinPiker, &SOURCE)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let spell = put(&mut g, p, "Divination", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        assert_eq!(
            FundingState::from_game(&g, &goal(p, u32::MAX)).total_mana_upper(),
            u128::MAX
        );
        act(&mut g, Action::AnnounceCast { card: spell });
        for payment in [[0, 0, 0, 0, 0, 1], [0, 1, 0, 0, 0, 0]] {
            act(
                &mut g,
                Action::ActivateMana {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                    payment,
                    production: [0, 2, 0, 0, 0, 0],
                },
            );
        }
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
    }
}

#[test]
fn pruning_linked_return_paths_keep_the_existing_optimistic_escape() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Forest", Zone::Battlefield);
        let returned = put(&mut g, p, "Gilded Lotus", Zone::Exile);
        g.state.players[p.index()].mana = ZERO;
        g.state.exile_links.push(ExileLink {
            ability_origin: None,
            source: g.object_ref(source),
            exiled: g.object_ref(returned),
            return_zone: Zone::Battlefield,
        });
        let world = FundingState::from_game(&g, &goal(p, u32::MAX));
        assert!(world.total_mana_upper() < u128::from(u32::MAX));
        assert!(
            world.optimistic(&goal(p, u32::MAX)),
            "current inventory is no bound on linked returns"
        );
    }
}

fn finish_root(r: &mut Root, p: Player, helper: CardId) {
    let object = r.game.object_ref(helper);
    act(
        &mut r.game,
        Action::ChooseHarmonizer {
            object: Some(object),
        },
    );
    for (i, &land) in r.lands.iter().enumerate() {
        let mut production = ZERO;
        production[if i < 4 {
            4
        } else if i == 4 {
            1
        } else {
            3
        }] = 1;
        let action = r
            .game
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(a,
            Action::ActivateMana { card, production: m, .. } if *card == land && *m == production)
                    || matches!(a, Action::TapMana { card } if *card == land)
            })
            .expect("exact land output");
        act(&mut r.game, action);
    }
    pay(&mut r.game);
    let clause = r
        .game
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| {
            !row.paid && row.clause == CastCostClause::Harmonize(r.game.object_ref(helper))
        })
        .unwrap();
    act(
        &mut r.game,
        Action::ChooseCastCost {
            clause: clause as u16,
        },
    );
    settle(&mut r.game);
    assert_eq!(r.game.state.players[p.index()].mana, ZERO);
    assert_eq!(r.game.state.players[p.index()].life, 31);
    assert_eq!(r.game.object_view(r.spell).zone, Zone::Exile);
    assert!(r.game.object_view(helper).tapped);
}

#[test]
fn pruning_owned_x_root_preserves_exact_actions_privacy_rollback_and_branches() {
    let _rules = scope();
    for p in [Player::P0, Player::P1] {
        let r = x_root(p);
        let start = Instant::now();
        let space = r.game.legal_action_space();
        eprintln!(
            "root_action_space player={p:?} seconds={:.9}",
            start.elapsed().as_secs_f64()
        );
        assert_eq!(
            space.x_choice.as_ref().unwrap().intervals,
            vec![XInterval {
                minimum: 0,
                maximum: 11
            }]
        );
        assert_eq!(
            space.choices,
            std::iter::once(Action::Concede)
                .chain((0..=11).map(|value| Action::ChooseX { value }))
                .collect::<Vec<_>>()
        );
        assert_eq!(r.game.observe(p).x_choice, space.x_choice);
        assert!(r.game.observe(p.other()).x_choice.is_none());
        let action = Action::ChooseX { value: 11 };
        let revision = r.game.revision();
        let saved = r.game.snapshot();
        let index = space.choices.iter().position(|a| *a == action).unwrap();
        let make = || Root {
            game: r.game.clone(),
            spell: r.spell,
            providers: r.providers,
            converter: r.converter,
            lands: r.lands.clone(),
            secret: r.secret,
        };
        let mut raw = make();
        let mut indexed = make();
        let mut sibling = make();
        drop(r);
        assert_eq!(
            raw.game.apply(p.other(), action.clone()),
            Err(Error::WrongActor)
        );
        assert_eq!(raw.game.snapshot(), saved);
        reject(&mut raw.game, Action::ChooseX { value: 12 });
        raw.game.apply(p, action.clone()).unwrap();
        indexed.game.apply_index(p, revision, index).unwrap();
        act(&mut sibling.game, action.clone());
        let a = raw.providers[0];
        let b = sibling.providers[1];
        finish_root(&mut raw, p, a);
        finish_root(&mut indexed, p, a);
        finish_root(&mut sibling, p, b);
        assert_eq!(raw.game.snapshot(), indexed.game.snapshot());
        assert!(!raw.game.object_view(b).tapped && !sibling.game.object_view(a).tapped);
        for child in [&raw, &indexed, &sibling] {
            assert!(
                child
                    .game
                    .observe(p)
                    .objects
                    .iter()
                    .any(|o| o.id == child.secret)
            );
            assert!(
                !child
                    .game
                    .observe(p.other())
                    .objects
                    .iter()
                    .any(|o| o.id == child.secret)
            );
        }
        indexed.game.restore(&saved).unwrap();
        assert_eq!(
            indexed.game.apply_index(p, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed.game, action);
        finish_root(&mut indexed, p, a);
        assert_eq!(raw.game.snapshot(), indexed.game.snapshot());
    }
}

const BLANK_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::RemoveAttachedAbilities],
}];
const BLANK_AURA: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Creature))),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: 0,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    static_groups: BLANK_GROUPS,
    extras: &CardExtras {
        static_ability_groups: BLANK_GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn restoration_hidden_printed_producer_is_not_pruned_before_its_blanker_can_be_sacrificed() {
    const PRODUCER: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 4, 0, 0, 0, 0,
            ]))],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    const SAC: CardRules = CardRules {
        effect: Effect::Creature(2, 1),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::Sacrifice,
                    filter: SacrificeFilter {
                        subtype: None,
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 0,
                        nonland: false,
                        other: false,
                        tokens: None,
                        artifact_enchantment_or_token: false,
                    },
                }),
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed(ZERO))],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &PRODUCER),
        (Card::GoblinPiker, &SAC),
        (Card::from_name("Stop Cold").unwrap(), &BLANK_AURA),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let producer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let aura = put(&mut g, p, "Stop Cold", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(producer));
        let sac = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let spell = put(&mut g, p, "Divination", Zone::Hand);
        g.state.players[p.index()].mana = ZERO;
        assert_eq!(
            FundingState::from_game(&g, &goal(p, 3)).total_mana_upper(),
            0,
            "current availability deliberately cannot see the future restored producer"
        );
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell }),
            "ability-removal restoration must bypass current-inventory quantity pruning"
        );
        act(&mut g, Action::AnnounceCast { card: spell });
        act(
            &mut g,
            Action::AnnounceMana {
                card: sac,
                ability: 0,
                counter_payment: Counters::default(),
                production: ZERO,
            },
        );
        let object = g.object_ref(aura);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        act(
            &mut g,
            Action::ActivateMana {
                card: producer,
                ability: 0,
                counter_payment: Counters::default(),
                payment: ZERO,
                production: [0, 4, 0, 0, 0, 0],
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(g.state.players[p.index()].mana, [0, 1, 0, 0, 0, 0]);
    }
}

#[test]
fn restoration_future_exhaust_discount_keeps_tap_capacity_conservative() {
    const LAND: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[ActivatedAbility {
            exhaust: true,
            cost: ActivationCost {
                generic: 1,
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 3, 0, 0,
            ]))],
            ..BASE
        }],
        extras: &CardExtras {
            exhaust_mask: 1,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    const SAC_AURA: CardRules = CardRules {
        activated: &[ActivatedAbility {
            once_per_turn: true,
            cost: ActivationCost {
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 1, 0, 0,
            ]))],
            ..BASE
        }],
        ..BLANK_AURA
    };
    let _rules = fixture_rules::activate(&[
        (Card::Forest, &LAND),
        (Card::from_name("Stop Cold").unwrap(), &SAC_AURA),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        let scholar = put(&mut g, p, "Boom Scholar", Zone::Battlefield);
        let aura = put(&mut g, p, "Stop Cold", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(scholar));
        let spell = put(&mut g, p, "Hill Giant", Zone::Hand);
        g.state.players[p.index()].mana = ZERO;
        let world = FundingState::from_game(&g, &goal(p, 4));
        assert!(
            world.total_mana_upper() >= 4,
            "ignoring future discounts while taking current net-tap max would wrongly give3"
        );
        act(&mut g, Action::AnnounceCast { card: spell });
        act(
            &mut g,
            Action::ActivateMana {
                card: aura,
                ability: 0,
                counter_payment: Counters::default(),
                payment: ZERO,
                production: [0, 0, 0, 1, 0, 0],
            },
        );
        act(
            &mut g,
            Action::ActivateMana {
                card: land,
                ability: 0,
                counter_payment: Counters::default(),
                payment: ZERO,
                production: [0, 0, 0, 3, 0, 0],
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, ZERO);
    }
}

#[test]
fn restoration_source_bound_copy_can_expose_a_larger_tap_output() {
    const PHYSICAL: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 2, 0, 0, 0, 0,
            ]))],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    const COPIED: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 1, 0, 0, 0, 0,
            ]))],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    const SOURCE: CardRules = CardRules {
        effect: Effect::Creature(2, 1),
        activated: &[ActivatedAbility {
            once_per_turn: true,
            cost: ActivationCost {
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 1, 0, 0, 0, 0,
            ]))],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::Forest, &PHYSICAL),
        (Card::Island, &COPIED),
        (Card::GoblinPiker, &SOURCE),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        let donor = put(&mut g, p, "Island", Zone::Outside);
        let source = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let recipe = g.copiable_recipe(g.object_ref(donor));
        g.apply_temporary_copy(
            g.object_ref(land),
            recipe,
            CopyDeadline::WhileSource {
                source: g.object_ref(source),
            },
        );
        let spell = put(&mut g, p, "Divination", Zone::Hand);
        g.state.players[p.index()].mana = ZERO;
        let world = FundingState::from_game(&g, &goal(p, 3));
        eprintln!(
            "source_bound_copy_current_upper={}",
            world.total_mana_upper()
        );
        assert!(
            world.optimistic(&goal(p, 3)),
            "expiring copy may expose a larger underlying producer"
        );
        act(&mut g, Action::AnnounceCast { card: spell });
        act(
            &mut g,
            Action::ActivateMana {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
                payment: ZERO,
                production: [0, 1, 0, 0, 0, 0],
            },
        );
        assert_eq!(g.state.objects[land as usize].card, Card::Forest);
        act(
            &mut g,
            Action::ActivateMana {
                card: land,
                ability: 0,
                counter_payment: Counters::default(),
                payment: ZERO,
                production: [0, 2, 0, 0, 0, 0],
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(g.state.players[p.index()].mana, ZERO);
    }
}
