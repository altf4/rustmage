//! Shared cost contracts only: no ordinary DFT definition or admission changes.
use rustmage as _;
mod support;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const ANY: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const ARTIFACTS: SacrificeFilter = SacrificeFilter { kinds: 16, ..ANY };
const PLAYER: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Player {
    opponent_only: false,
}));
const fn ability(
    cost: ActivationCost,
    targets: TargetPlan,
    program: &'static [Instruction],
) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost,
        targets,
        program,
        sorcery: false,
        once_per_turn: false,
    }
}
const LOTUS: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ability(
        ActivationCost {
            tap: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::SacrificeRange { minimum: 1 },
                filter: ARTIFACTS,
            }),
            ..ActivationCost::ZERO
        },
        PLAYER,
        &[Instruction::AddManaChosenScaled {
            players: PlayerSet::Targeted,
            amount: AmountValue::Multiple {
                value: &AmountValue::PaidAdditionalCostCount,
                factor: 3,
            },
        }],
    )],
    ..CardRules::EMPTY
};
const WINTER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        exhaust: true,
        ..ability(
            ActivationCost {
                generic: 2,
                colored: [0, 1, 1, 0, 0, 0],
                mana_component: true,
                tap: true,
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::ExileGraveyardRange { minimum: 0 },
                    filter: ARTIFACTS,
                }),
                ..ActivationCost::ZERO
            },
            TargetPlan::NONE,
            &[Instruction::GainLifeScaled(
                PlayerSet::Controller,
                AmountValue::PaidAdditionalCostSelectionCount,
            )],
        )
    }],
    extras: &CardExtras {
        exhaust_mask: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const UNION: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ability(
        ActivationCost {
            colored: [0, 0, 1, 0, 0, 0],
            mana_component: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::FilteredCost {
                    action: CostAction::Sacrifice,
                    any: &[CostMatch::Kinds(1), CostMatch::Subtype("Vehicle")],
                },
                filter: SacrificeFilter { other: true, ..ANY },
            }),
            ..ActivationCost::ZERO
        },
        TargetPlan::NONE,
        &[Instruction::Draw(PlayerSet::Controller, 1)],
    )],
    ..CardRules::EMPTY
};
const TAP_UNION: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ability(
        ActivationCost {
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::FilteredCost {
                    action: CostAction::TapPermanents(1),
                    any: &[CostMatch::Subtype("Mount"), CostMatch::Subtype("Vehicle")],
                },
                filter: SacrificeFilter { other: true, ..ANY },
            }),
            ..ActivationCost::ZERO
        },
        TargetPlan::NONE,
        &[Instruction::GainLife(PlayerSet::Controller, 1)],
    )],
    ..CardRules::EMPTY
};
const BLACK_MANA: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    activated: &[ability(
        ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        TargetPlan::NONE,
        &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 1, 0, 0, 0,
        ]))],
    )],
    ..CardRules::EMPTY
};
const BLOCK_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::GraveyardCardsCannotBeExiled],
}];
const EXILE_BLOCKER_MANA: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: BLOCK_GROUP,
    activated: &[ability(
        ActivationCost {
            tap: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        TargetPlan::NONE,
        &[Instruction::AddMana(ManaProduction::Fixed([
            0, 1, 1, 0, 0, 2,
        ]))],
    )],
    extras: &CardExtras {
        static_ability_groups: BLOCK_GROUP,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const SIDESWIPE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::CreatureOrVehicle {
            relation: 0,
        })),
        &[
            Instruction::Destroy,
            Instruction::IfSacrificedVehicle { skip: 1 },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::Sacrifice,
            filter: SacrificeFilter { kinds: 17, ..ANY },
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const COPIER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[ability(
        ActivationCost::ZERO,
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::CopyTargetAbility { group: 0 }],
    )],
    ..CardRules::EMPTY
};
fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn begin(g: &mut Game, source: CardId, target: Option<Target>) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if let Some(target) = target {
        act(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn select(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    act(g, Action::ChooseAdditionalCost { object });
}
fn reject_selection(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    reject(g, Action::ChooseAdditionalCost { object });
}
fn finish(g: &mut Game) {
    act(g, Action::FinishAdditionalCostGroup);
    pay(g);
}
fn drain(g: &mut Game) {
    for _ in 0..64 {
        if g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending == Pending::Priority
        {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishStackTargets
            )
        });
    }
    panic!("64-action cost resolution ceiling");
}
fn to_mana_choice(g: &mut Game) {
    for _ in 0..32 {
        if matches!(g.state.pending, Pending::EffectMana { .. }) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishStackTargets
            )
        });
    }
    panic!("missing targeted color choice");
}
fn available(g: &Game, id: CardId) -> bool {
    g.legal_actions().iter().any(|a| {
        matches!(a,
        Action::AnnounceActivate { card, ability:0, .. } | Action::Activate { card, ability:0, .. }
        if *card == id)
    })
}

#[test]
fn typed_cost_union_matches_vehicle_and_creature_in_real_and_projected_worlds() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &UNION),
        (Card::LlanowarElves, &BLACK_MANA),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let vehicle = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
        let artifact = put(&mut g, p, "Soul-Guide Lantern", Zone::Battlefield);
        let foreign = put(&mut g, p.other(), "Clamorous Ironclad", Zone::Battlefield);
        let producer = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        assert!(
            available(&g, source),
            "tap producer, then sacrifice same producer or Vehicle"
        );
        begin(&mut g, source, None);
        let legal = g.legal_actions();
        for id in [vehicle, producer] {
            assert!(legal.contains(&Action::ChooseAdditionalCost {
                object: g.object_ref(id)
            }));
        }
        for id in [source, artifact, foreign] {
            reject_selection(&mut g, id);
        }
        select(&mut g, vehicle);
        choose(
            &mut g,
            |a| matches!(a, Action::ActivateMana { card, .. } if *card == producer),
        );
        pay(&mut g);
        assert_eq!(g.object_view(vehicle).zone, Zone::Graveyard);
        assert!(g.object_view(producer).tapped);
        drain(&mut g);
        assert_eq!(g.hand_count(p), 1);
    }
}

#[test]
fn tap_union_accepts_mount_or_vehicle_and_preserves_untapped_other_control_restrictions() {
    let _rules = fixture_rules::activate(&[(named("Clamorous Ironclad"), &TAP_UNION)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
        let mount = put(&mut g, p, "Gilded Ghoda", Zone::Battlefield);
        let vehicle = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
        let ordinary = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let foreign = put(&mut g, p.other(), "Gilded Ghoda", Zone::Battlefield);
        g.state.objects[vehicle as usize].tapped = true;
        begin(&mut g, source, None);
        for id in [source, vehicle, ordinary, foreign] {
            reject_selection(&mut g, id);
        }
        select(&mut g, mount);
        pay(&mut g);
        assert!(g.object_view(mount).tapped);
        assert!(!g.object_view(source).tapped);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].life, 21);
    }
}

#[test]
fn lotus_source_tap_self_sacrifice_tokens_and_redirected_sacrifice_freeze_actual_count() {
    let _rules = fixture_rules::activate(&[(named("Clamorous Ironclad"), &LOTUS)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
        let token = put(&mut g, p, "Soul-Guide Lantern", Zone::Battlefield);
        g.state.objects[token as usize].copy_kind = CopyKind::Token;
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        g.state.players[p.other().index()].mana = [0; 6];
        begin(&mut g, source, Some(Target::Player(p.other())));
        assert!(
            !g.legal_actions()
                .contains(&Action::FinishAdditionalCostGroup)
        );
        select(&mut g, source);
        select(&mut g, token);
        finish(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        let paid = g
            .state
            .stack
            .last()
            .unwrap()
            .paid_additional_cost
            .as_ref()
            .unwrap();
        assert_eq!((paid.selected, paid.count), (2, 2));
        assert!(paid.sacrificed_vehicle);
        assert!(
            !g.state.stack.is_empty(),
            "targeted mana production uses the stack"
        );
        to_mana_choice(&mut g);
        assert_eq!(g.actor(), Some(p), "ability controller chooses the color");
        let actions = g.legal_actions();
        assert_eq!(actions.len(), 5);
        for a in &actions {
            let Action::ChooseMana { production } = a else {
                panic!("color choice")
            };
            assert_eq!(production.iter().filter(|&&v| v != 0).count(), 1);
            assert_eq!(production.iter().sum::<u32>(), 6);
            assert_eq!(production[5], 0);
        }
        act(
            &mut g,
            Action::ChooseMana {
                production: [0, 0, 0, 6, 0, 0],
            },
        );
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.players[p.other().index()].mana, [0, 0, 0, 6, 0, 0]);
    }
}

#[test]
fn ranged_selection_is_incremental_owned_and_transactional() {
    let _rules = fixture_rules::activate(&[(named("Clamorous Ironclad"), &LOTUS)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
    let extra = put(&mut g, p, "Soul-Guide Lantern", Zone::Battlefield);
    let invalid = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    begin(&mut g, source, Some(Target::Player(p)));
    reject(&mut g, Action::FinishAdditionalCostGroup);
    reject_selection(&mut g, invalid);
    select(&mut g, source);
    reject_selection(&mut g, source);
    let action = Action::ChooseAdditionalCost {
        object: g.object_ref(extra),
    };
    let rev = g.revision();
    let index = g.legal_actions().iter().position(|a| a == &action).unwrap();
    let snapshot = g.snapshot();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    drop(g);
    raw.apply(p, action.clone()).unwrap();
    indexed.apply_index(p, rev, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    for viewer in [p, p.other()] {
        assert_eq!(
            serde_json::to_value(raw.observe(viewer)).unwrap(),
            serde_json::to_value(indexed.observe(viewer)).unwrap()
        );
    }
    indexed.restore(&snapshot).unwrap();
    assert_eq!(
        indexed.apply_index(p, rev, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    finish(&mut raw);
    finish(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    let mut stale = raw.clone();
    stale.restore(&snapshot).unwrap();
    let reference = stale.object_ref(extra);
    stale.move_object(extra, Zone::Exile);
    stale.move_object(extra, Zone::Battlefield);
    reject(
        &mut stale,
        Action::ChooseAdditionalCost { object: reference },
    );
}

#[test]
fn graveyard_x_allows_zero_and_keeps_declared_selection_under_prevention() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
        }
    }
    let _reset = Reset;
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &WINTER)]);
    for p in [Player::P0, Player::P1] {
        for prevent in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let first = put(&mut g, p, "Soul-Guide Lantern", Zone::Graveyard);
            let second = put(&mut g, p, "Clamorous Ironclad", Zone::Graveyard);
            let nonartifact = put(&mut g, p, "Hill Giant", Zone::Graveyard);
            let foreign = put(&mut g, p.other(), "Soul-Guide Lantern", Zone::Graveyard);
            begin(&mut g, source, None);
            assert!(
                g.legal_actions()
                    .contains(&Action::FinishAdditionalCostGroup)
            );
            for id in [nonartifact, foreign] {
                reject_selection(&mut g, id);
            }
            if prevent {
                select(&mut g, first);
                select(&mut g, second);
                zone_replacements::TEST_PREVENT_ZONE_MOVE
                    .with(|v| v.set(Some(g.object_ref(first))));
                g.state.zone_rules.get_or_insert_default();
            }
            finish(&mut g);
            let paid = g
                .state
                .stack
                .last()
                .unwrap()
                .paid_additional_cost
                .as_ref()
                .unwrap();
            assert_eq!(
                (paid.selected, paid.count),
                if prevent { (2, 1) } else { (0, 0) }
            );
            assert!(g.object_view(source).tapped);
            drain(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                if prevent { 22 } else { 20 }
            );
            assert!(!available(&g, source), "even X=0 consumes this exhaust use");
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
        }
    }
}

#[test]
fn graveyard_exile_prohibition_is_shared_by_funding_and_actual_payment() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &WINTER),
        (Card::HillGiant, &EXILE_BLOCKER_MANA),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let artifact = put(&mut g, p, "Soul-Guide Lantern", Zone::Graveyard);
        let blocker = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        assert!(available(&g, source));
        begin(&mut g, source, None);
        select(&mut g, artifact);
        act(&mut g, Action::FinishAdditionalCostGroup);
        choose(
            &mut g,
            |a| matches!(a, Action::ActivateMana {card,..} if *card == blocker),
        );
        assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
        pay(&mut g);
        assert_eq!(g.object_view(artifact).zone, Zone::Exile);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].life, 21);
    }
}

#[test]
fn sacrifice_vehicle_fact_is_predeparture_and_redirected_sacrifice_still_qualifies() {
    let _rules = fixture_rules::activate(&[(named("Shock"), &SIDESWIPE)]);
    for p in [Player::P0, Player::P1] {
        for vehicle in [false, true] {
            let mut g = scenario(p);
            put(&mut g, p, "Rest in Peace", Zone::Battlefield);
            let material = put(
                &mut g,
                p,
                if vehicle {
                    "Clamorous Ironclad"
                } else {
                    "Soul-Guide Lantern"
                },
                Zone::Battlefield,
            );
            let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let spell = put(&mut g, p, "Shock", Zone::Hand);
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
            select(&mut g, material);
            pay(&mut g);
            let paid = g
                .state
                .stack
                .last()
                .unwrap()
                .paid_additional_cost
                .as_ref()
                .unwrap();
            assert_eq!(paid.sacrificed_vehicle, vehicle);
            assert_eq!(paid.count, 1);
            assert_eq!(g.object_view(material).zone, Zone::Exile);
            drain(&mut g);
            assert_eq!(g.hand_count(p), if vehicle { 1 } else { 0 });
            assert_eq!(g.object_view(target).zone, Zone::Exile);
        }
    }
}

#[test]
fn copied_targeted_mana_preserves_paid_count_after_source_departure() {
    let _rules = fixture_rules::activate(&[
        (named("Clamorous Ironclad"), &LOTUS),
        (Card::HillGiant, &COPIER),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
    let copier = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    g.state.players[p.index()].mana = [0; 6];
    g.state.players[p.other().index()].mana = [0; 6];
    begin(&mut g, source, Some(Target::Player(p.other())));
    select(&mut g, source);
    finish(&mut g);
    let target = Target::Stack(g.state.stack.last().unwrap().id);
    choose(
        &mut g,
        |a| matches!(a, Action::Activate {card,ability:0,target:t,..} if *card == copier && *t == Some(target)),
    );
    to_mana_choice(&mut g);
    let copy = &g.state.resolving.as_ref().unwrap().item;
    assert!(copy.is_copy);
    assert_eq!(copy.paid_additional_cost.as_ref().unwrap().count, 1);
    act(
        &mut g,
        Action::ChooseMana {
            production: [3, 0, 0, 0, 0, 0],
        },
    );
    to_mana_choice(&mut g);
    act(
        &mut g,
        Action::ChooseMana {
            production: [0, 3, 0, 0, 0, 0],
        },
    );
    drain(&mut g);
    assert_eq!(g.state.players[p.other().index()].mana, [3, 3, 0, 0, 0, 0]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}

#[test]
fn a_departing_source_cannot_pay_a_second_distinct_departure_cost() {
    const TWO_COSTS: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[ability(
            ActivationCost {
                sacrifice: true,
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::SacrificeRange { minimum: 1 },
                    filter: ARTIFACTS,
                }),
                ..ActivationCost::ZERO
            },
            TargetPlan::NONE,
            &[Instruction::GainLife(PlayerSet::Controller, 1)],
        )],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[(named("Clamorous Ironclad"), &TWO_COSTS)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
    assert!(!available(&g, source));
    let extra = put(&mut g, p, "Soul-Guide Lantern", Zone::Battlefield);
    assert!(available(&g, source));
    begin(&mut g, source, None);
    reject_selection(&mut g, source);
    select(&mut g, extra);
    finish(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert_eq!(g.object_view(extra).zone, Zone::Graveyard);
    drain(&mut g);
    assert_eq!(g.state.players[p.index()].life, 21);
}

#[test]
fn an_all_illegal_sideswipe_target_fizzles_without_its_conditional_draw() {
    let _rules = fixture_rules::activate(&[(named("Shock"), &SIDESWIPE)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let material = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
    let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let spell = put(&mut g, p, "Shock", Zone::Hand);
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
    select(&mut g, material);
    pay(&mut g);
    assert!(
        g.state
            .stack
            .last()
            .unwrap()
            .paid_additional_cost
            .as_ref()
            .unwrap()
            .sacrificed_vehicle
    );
    g.move_object(target, Zone::Exile);
    drain(&mut g);
    assert_eq!(g.hand_count(p), 0);
    assert_eq!(g.object_view(material).zone, Zone::Graveyard);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
}

#[test]
fn disjunctive_external_mana_cost_has_the_same_projected_and_real_destination() {
    const SACRIFICE_MANA: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ability(
            ActivationCost {
                tap: true,
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::FilteredCost {
                        action: CostAction::Sacrifice,
                        any: &[CostMatch::Kinds(1), CostMatch::Subtype("Vehicle")],
                    },
                    filter: SacrificeFilter { other: true, ..ANY },
                }),
                ..ActivationCost::ZERO
            },
            TargetPlan::NONE,
            &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 1, 0, 0, 0,
            ]))],
        )],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &SACRIFICE_MANA)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let producer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let material = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
    let spell = put(&mut g, p, "Short Bow", Zone::Hand);
    g.state.players[p.index()].mana = [0; 6];
    assert!(
        g.legal_actions()
            .contains(&Action::AnnounceCast { card: spell })
    );
    act(&mut g, Action::AnnounceCast { card: spell });
    choose(
        &mut g,
        |a| matches!(a, Action::AnnounceMana {card,..} if *card == producer),
    );
    select(&mut g, material);
    pay(&mut g);
    assert_eq!(g.object_view(material).zone, Zone::Graveyard);
    assert_eq!(g.state.players[p.index()].mana, [0, 0, 1, 0, 0, 0]);
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
}
