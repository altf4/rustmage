//! Result continuations and sampled mill quantities. These typed scenario
//! definitions exercise shared rules without changing any card admission.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const ANY_CREATURE: TargetRule = TargetRule::Creature;
const GRAVE_CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 1,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const OWN_NONLAND: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 0,
    relation: 0,
    other: false,
    nonland: true,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const LANDS: AmountValue = AmountValue::Stat(StatValue::Count(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 2,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
}));
const AINOK: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(3)),
            Instruction::RecoverMilledOrSkip {
                filter: RecoveryFilter::Land,
                skip: 1,
            },
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
const PROWLER: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayMillAmount(LANDS),
            Instruction::Mill(PlayerSet::Controller, MillAmount::Amount(LANDS)),
        ],
    }],
    ..CardRules::EMPTY
};
const CLIMBER: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Player {
                opponent_only: false,
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::Mill(
            PlayerSet::Targeted,
            MillAmount::Amount(AmountValue::SourcePower),
        )],
    }],
    ..CardRules::EMPTY
};
const LIE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(GRAVE_CREATURE),
            TargetSpec::one(ANY_CREATURE),
        ]),
        &[
            Instruction::ReturnToHand,
            Instruction::SelectGroup(1),
            Instruction::DamageByTargetPower { power_group: 0 },
        ],
    ),
    ..CardRules::EMPTY
};
const HAND_POWER: DynamicStats = DynamicStats {
    power: Some(StatExpr {
        value: StatValue::ControllerHandSize,
        multiplier: 1,
        offset: 0,
    }),
    toughness: None,
};
const CDA: CardRules = CardRules {
    effect: Effect::Creature(0, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Cda,
        fragments: &[StaticAbilityFragment::DynamicStats(0, HAND_POWER)],
    }],
    extras: &CardExtras {
        dynamic_stats: Some(HAND_POWER),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const BOUNCE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: OWN_NONLAND,
            min: 0,
            max: 1,
        }),
        &[Instruction::ReturnToHandDrawForTokens],
    ),
    ..CardRules::EMPTY
};
const ENVOY: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::IfAmountAtLeast {
                value: AmountValue::Stat(StatValue::CounterBearingCreatures),
                minimum: 1,
                skip: 1,
            },
            Instruction::DrawRemembered(AmountValue::Fixed(1)),
            Instruction::IfNoRememberedDraws { skip: 1 },
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
const SHRIEKER: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        program: &[
            Instruction::DamageRememberPlayers(2),
            Instruction::GainLife(PlayerSet::Controller, 2),
            Instruction::DiscardDamagedPlayers(1),
        ],
    }],
    ..CardRules::EMPTY
};
const PREVENTER: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 1,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(ANY_CREATURE)),
        program: &[Instruction::PreventTargetDamageUntilSourceLeaves],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const DAMAGE_DOUBLE: ReplacementSpec = ReplacementSpec::DoubleOpponentDamage;
const DOUBLER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, DAMAGE_DOUBLE)],
    }],
    extras: &CardExtras {
        replacement_effects: &[DAMAGE_DOUBLE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const YATHAN: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersIfCast,
            targets: TargetPlan::NONE,
            program: &[Instruction::MillThenReflexive {
                amount: 4,
                ability: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: 3,
            })),
            program: &[Instruction::ReturnToBattlefield],
        },
    ],
    ..CardRules::EMPTY
};
fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn give(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        act(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn ready(g: &mut Game, target: Option<Target>, predicate: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if predicate(g) {
            return;
        }
        if let Some(target) = target {
            let action = Action::ChooseTarget { target };
            if g.legal_actions().contains(&action) {
                act(g, action);
                continue;
            }
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishTargets
                    | Action::FinishStackTargets
                    | Action::ChooseReplacement { .. }
            ) || matches!(a, Action::ChooseTarget { target:t } if Some(*t)==target)
        });
    }
    panic!("64-action result scenario budget");
}
fn drain(g: &mut Game) {
    ready(g, None, |g| {
        g.outcome().is_some()
            || (g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none()
                && g.state.pending == Pending::Priority)
    });
}
fn clear_library(g: &mut Game, p: Player) {
    for id in g.state.players[p.index()].library.clone() {
        g.move_object(id, Zone::Outside);
    }
}
fn cast_creature(g: &mut Game, p: Player) -> CardId {
    give(g, p);
    let id = put(g, p, "Bear Cub", Zone::Hand);
    cast(g, id, None);
    id
}
fn spell_targets(g: &mut Game, p: Player, targets: &[Target]) -> CardId {
    give(g, p);
    let id = put(g, p, "Opt", Zone::Hand);
    announce(g, id, targets);
    id
}
fn token_elf(g: &mut Game, p: Player) -> CardId {
    g.stage_copy_creation(p, CopyRecipe::new(card("Llanowar Elves")), 1);
    assert!(!g.advance_entries());
    g.state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, object)| {
            (object.zone == Zone::Battlefield && object.controller == p && object.is_token())
                .then_some(id as CardId)
        })
        .unwrap()
}
fn attack(g: &mut Game, id: CardId) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.priority = g.state.active;
    act(g, Action::Attack { card: id });
    act(g, Action::FinishAttackers);
}

#[test]
fn ainok_recovery_choice_and_decline_have_distinct_counter_outcomes() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &AINOK)]);
    for p in [Player::P0, Player::P1] {
        for recover in [false, true] {
            let mut g = scenario(p);
            let top = put(&mut g, p, "Forest", Zone::Library);
            let source = cast_creature(&mut g, p);
            ready(&mut g, None, |g| {
                matches!(g.state.pending, Pending::RecoverMilled { .. })
            });
            let before = g.object_ref(top);
            let illegal = g.object_ref(source);
            reject(
                &mut g,
                Action::RecoverMilled {
                    object: Some(illegal),
                },
            );
            act(
                &mut g,
                Action::RecoverMilled {
                    object: recover.then_some(before),
                },
            );
            drain(&mut g);
            assert_eq!(
                g.counter_count(source, CounterKind::PlusOne),
                u64::from(!recover)
            );
            assert_eq!(
                g.object_view(top).zone,
                if recover { Zone::Hand } else { Zone::Graveyard }
            );
        }
    }
}
#[test]
fn ainok_no_matching_mill_uses_fallback_and_absent_source_stays_absent() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &AINOK)]);
    for absent in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        clear_library(&mut g, p);
        put(&mut g, p, "Llanowar Elves", Zone::Library);
        let id = cast_creature(&mut g, p);
        ready(&mut g, None, |g| {
            g.object_view(id).zone == Zone::Battlefield && g.state.pending == Pending::Priority
        });
        if absent {
            g.move_object(id, Zone::Exile);
        }
        drain(&mut g);
        assert_eq!(
            g.counter_count(id, CounterKind::PlusOne),
            u64::from(!absent)
        );
        assert_eq!(g.library_card_count(p), 0);
    }
}
#[test]
fn ainok_can_recover_land_redirected_to_public_exile() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &AINOK)]);
    let p = Player::P1;
    let mut g = scenario(p);
    put(&mut g, p, "Rest in Peace", Zone::Battlefield);
    let land = put(&mut g, p, "Forest", Zone::Library);
    let source = cast_creature(&mut g, p);
    ready(&mut g, None, |g| {
        matches!(g.state.pending, Pending::RecoverMilled { .. })
    });
    assert_eq!(g.object_view(land).zone, Zone::Exile);
    let reference = g.object_ref(land);
    act(
        &mut g,
        Action::RecoverMilled {
            object: Some(reference),
        },
    );
    drain(&mut g);
    assert_eq!(g.object_view(land).zone, Zone::Hand);
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
}
#[test]
fn ainok_owned_recovery_rejects_stale_and_preserves_branch_restore() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &AINOK)]);
    let p = Player::P1;
    let mut parent = scenario(p);
    let land = put(&mut parent, p, "Forest", Zone::Library);
    let id = cast_creature(&mut parent, p);
    ready(&mut parent, None, |g| {
        matches!(g.state.pending, Pending::RecoverMilled { .. })
    });
    let action = Action::RecoverMilled {
        object: Some(parent.object_ref(land)),
    };
    let rev = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut sibling = parent.clone();
    drop(parent);
    assert_eq!(raw.apply(p.other(), action.clone()), Err(Error::WrongActor));
    assert_eq!(raw.snapshot(), saved);
    raw.apply(p, action.clone()).unwrap();
    indexed.apply_index(p, rev, index).unwrap();
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.counter_count(id, CounterKind::PlusOne), 0);
    act(&mut sibling, Action::RecoverMilled { object: None });
    drain(&mut sibling);
    assert_eq!(sibling.counter_count(id, CounterKind::PlusOne), 1);
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(p, rev, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
#[test]
fn dynamic_optional_mill_requires_complete_current_land_count_and_allows_zero() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &PROWLER)]);
    for p in [Player::P0, Player::P1] {
        for (lands, library, accept) in [(3, 3, true), (3, 8, false), (3, 2, true), (0, 0, true)] {
            let mut g = scenario(p);
            clear_library(&mut g, p);
            for _ in 0..library {
                put(&mut g, p, "Forest", Zone::Library);
            }
            for _ in 0..lands {
                put(&mut g, p, "Forest", Zone::Battlefield);
            }
            cast_creature(&mut g, p);
            if library >= lands {
                ready(&mut g, None, |g| {
                    matches!(g.state.pending, Pending::EffectOptional { .. })
                });
                act(&mut g, Action::ChooseOptional { accept });
            }
            drain(&mut g);
            assert_eq!(
                g.library_card_count(p),
                if library >= lands && accept {
                    library - lands
                } else {
                    library
                }
            );
            assert!(!g.state.players[p.index()].attempted_empty_draw);
        }
    }
}
#[test]
fn dynamic_mill_samples_lands_at_resolution_after_response() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &PROWLER)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let land = put(&mut g, p, "Forest", Zone::Battlefield);
    let source = cast_creature(&mut g, p);
    ready(&mut g, None, |g| {
        g.object_view(source).zone == Zone::Battlefield && g.state.pending == Pending::Priority
    });
    g.move_object(land, Zone::Hand);
    put(&mut g, p.other(), "Forest", Zone::Battlefield);
    ready(&mut g, None, |g| {
        matches!(g.state.pending, Pending::EffectOptional { .. })
    });
    act(&mut g, Action::ChooseOptional { accept: true });
    drain(&mut g);
    assert_eq!(g.library_card_count(p), 8);
}
#[test]
fn attack_mill_uses_current_power_and_exact_departure_lki_floored_at_zero() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &CLIMBER)]);
    for (power, depart, reenter, expected) in [
        (4, false, false, 4),
        (5, true, false, 5),
        (5, true, true, 5),
        (-2, false, false, 0),
    ] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        attack(&mut g, source);
        ready(&mut g, Some(Target::Player(p.other())), |g| {
            g.state.pending == Pending::Priority && !g.state.stack.is_empty()
        });
        g.state.objects[source as usize].growth.power = power - 1;
        if depart {
            g.move_object(source, Zone::Exile);
            if reenter {
                g.move_object(source, Zone::Battlefield);
                g.state.objects[source as usize].growth.power = 30;
            }
        }
        ready(&mut g, None, |g| {
            g.state.stack.is_empty()
                && g.state.resolving.is_none()
                && g.state.pending == Pending::Priority
        });
        assert_eq!(g.library_card_count(p.other()), 8 - expected);
    }
}
#[test]
fn attack_optional_player_decline_mills_nobody() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &CLIMBER)]);
    let p = Player::P1;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    attack(&mut g, source);
    drain(&mut g);
    assert_eq!(g.library_card_count(p), 8);
    assert_eq!(g.library_card_count(p.other()), 8);
}
#[test]
fn graveyard_power_is_captured_before_return_changes_cda() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &LIE), (card("Bear Cub"), &CDA)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let grave = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        put(&mut g, p, "Forest", Zone::Hand);
        put(&mut g, p, "Forest", Zone::Hand);
        let target = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        spell_targets(&mut g, p, &[Target::Card(grave), Target::Card(target)]);
        drain(&mut g);
        assert_eq!(g.object_view(grave).zone, Zone::Hand);
        assert_eq!(g.stats(grave).power, 3);
        assert_eq!(g.state.objects[target as usize].damage, 2);
    }
}
#[test]
fn independent_invalid_lie_targets_preserve_only_their_own_effects() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &LIE)]);
    for invalid_grave in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let grave = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let victim = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        spell_targets(&mut g, p, &[Target::Card(grave), Target::Card(victim)]);
        let invalid = if invalid_grave { grave } else { victim };
        g.move_object(invalid, Zone::Exile);
        drain(&mut g);
        if invalid_grave {
            assert_eq!(g.state.objects[victim as usize].damage, 0);
        } else {
            assert_eq!(g.object_view(grave).zone, Zone::Hand);
        }
    }
}
#[test]
fn lie_needs_both_targets_and_rejects_wrong_grave_owner_transactionally() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &LIE)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Opt", Zone::Hand);
    let wrong = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
    let legal = put(&mut g, p, "Bear Cub", Zone::Graveyard);
    put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
    act(&mut g, Action::AnnounceCast { card: spell });
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(wrong),
        },
    );
    reject(&mut g, Action::FinishTargets);
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(legal),
        },
    );
    reject(&mut g, Action::FinishTargets);
    assert_eq!(g.object_view(spell).zone, Zone::Stack);
}
#[test]
fn token_qualified_return_draws_from_pre_move_token_fact_only() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &BOUNCE)]);
    for token in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let target = if token {
            token_elf(&mut g, p)
        } else {
            put(&mut g, p, "Llanowar Elves", Zone::Battlefield)
        };
        spell_targets(&mut g, p, &[Target::Card(target)]);
        drain(&mut g);
        assert_eq!(
            g.state.players[p.index()].cards_drawn_this_turn,
            u64::from(token)
        );
        assert_eq!(
            g.object_view(target).zone,
            if token { Zone::Outside } else { Zone::Hand }
        );
    }
}
#[test]
fn token_return_optional_none_or_stale_token_does_not_draw() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &BOUNCE)]);
    for selected in [false, true] {
        let p = Player::P1;
        let mut g = scenario(p);
        let target = token_elf(&mut g, p);
        spell_targets(
            &mut g,
            p,
            &if selected {
                vec![Target::Card(target)]
            } else {
                vec![]
            },
        );
        if selected {
            g.move_object(target, Zone::Exile);
        }
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
    }
}
#[test]
fn draw_result_fallback_checks_actual_draw_and_any_counter_kind() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &ENVOY)]);
    for p in [Player::P0, Player::P1] {
        for eligible in [false, true] {
            let mut g = scenario(p);
            let recipient = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            if eligible {
                g.add_counter(recipient, CounterKind::Soul, 1);
            }
            let source = cast_creature(&mut g, p);
            drain(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                u64::from(eligible)
            );
            assert_eq!(
                g.counter_count(source, CounterKind::PlusOne),
                u64::from(!eligible)
            );
        }
    }
}
#[test]
fn failed_draw_uses_fallback_while_opposing_or_noncreature_counters_do_not_qualify() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &ENVOY)]);
    for mode in 0..3 {
        let p = Player::P0;
        let mut g = scenario(p);
        let id = put(
            &mut g,
            if mode == 1 { p.other() } else { p },
            if mode == 2 {
                "Forest"
            } else {
                "Llanowar Elves"
            },
            Zone::Battlefield,
        );
        g.add_counter(id, CounterKind::Stun, 1);
        if mode == 0 {
            clear_library(&mut g, p);
        }
        let source = cast_creature(&mut g, p);
        drain(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        assert_eq!(g.state.players[p.index()].attempted_empty_draw, mode == 0);
    }
}
#[test]
fn remembered_draw_survives_private_miracle_pause_and_owned_restore() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &ENVOY)]);
    let p = Player::P1;
    let mut parent = scenario(p);
    put(&mut parent, p, "Lorehold, the Historian", Zone::Battlefield);
    let recipient = put(&mut parent, p, "Llanowar Elves", Zone::Battlefield);
    parent.add_counter(recipient, CounterKind::Charge, 1);
    let source = cast_creature(&mut parent, p);
    ready(&mut parent, None, |g| {
        matches!(g.state.pending, Pending::MiracleReveal { .. })
    });
    assert!(parent.observe(p).miracle.is_some());
    assert!(parent.observe(p.other()).miracle.is_none());
    assert_eq!(parent.counter_count(source, CounterKind::PlusOne), 0);
    let action = Action::DeclineMiracle;
    let rev = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    raw.apply(p, action.clone()).unwrap();
    indexed.apply_index(p, rev, index).unwrap();
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.counter_count(source, CounterKind::PlusOne), 0);
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(p, rev, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
#[test]
fn actual_player_damage_discards_only_that_player_and_gain_is_independent() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &SHRIEKER)]);
    for p in [Player::P0, Player::P1] {
        for player_target in [false, true] {
            let mut g = scenario(p);
            let discarded = put(&mut g, p.other(), "Forest", Zone::Hand);
            let victim = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
            cast_creature(&mut g, p);
            let target = if player_target {
                Target::Player(p.other())
            } else {
                Target::Card(victim)
            };
            ready(&mut g, Some(target), |g| {
                matches!(g.state.pending, Pending::EffectDiscard { .. })
                    || (g.state.stack.is_empty()
                        && g.state.resolving.is_none()
                        && g.state.pending == Pending::Priority)
            });
            if player_target {
                assert_eq!(g.actor(), Some(p.other()));
                act(&mut g, Action::Discard { card: discarded });
            }
            drain(&mut g);
            assert_eq!(g.state.players[p.index()].life, 22);
            assert_eq!(
                g.state.players[p.other().index()].life,
                if player_target { 18 } else { 20 }
            );
            assert_eq!(
                g.object_view(discarded).zone,
                if player_target {
                    Zone::Graveyard
                } else {
                    Zone::Hand
                }
            );
        }
    }
}
#[test]
fn prevented_player_damage_has_no_discard_but_still_gains_life() {
    let _scope = fixture_rules::activate(&[
        (card("Bear Cub"), &SHRIEKER),
        (card("Short Bow"), &PREVENTER),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let preventer = put(&mut g, p, "Short Bow", Zone::Battlefield);
    let hand = put(&mut g, p.other(), "Forest", Zone::Hand);
    let source = cast_creature(&mut g, p);
    ready(&mut g, Some(Target::Player(p.other())), |g| {
        g.object_view(source).zone == Zone::Battlefield
            && g.state.pending == Pending::Priority
            && !g.state.stack.is_empty()
    });
    give(&mut g, p);
    act(
        &mut g,
        Action::AnnounceActivate {
            card: preventer,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.state.players[0].life, 22);
    assert_eq!(g.state.players[1].life, 20);
    assert_eq!(g.object_view(hand).zone, Zone::Hand);
}
#[test]
fn damage_result_is_owned_across_replacement_order_and_discard_choice() {
    let _scope = fixture_rules::activate(&[
        (card("Bear Cub"), &SHRIEKER),
        (card("Short Bow"), &DOUBLER),
        (card("Swiftfoot Boots"), &DOUBLER),
    ]);
    let p = Player::P1;
    let mut parent = scenario(p);
    put(&mut parent, p, "Short Bow", Zone::Battlefield);
    put(&mut parent, p, "Swiftfoot Boots", Zone::Battlefield);
    let hand = put(&mut parent, p.other(), "Forest", Zone::Hand);
    cast_creature(&mut parent, p);
    ready(&mut parent, Some(Target::Player(p.other())), |g| {
        matches!(g.state.pending, Pending::Replacement { .. })
    });
    let actor = parent.actor().unwrap();
    let action = parent
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::ChooseReplacement { .. }))
        .unwrap();
    let rev = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    assert_eq!(
        raw.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(raw.snapshot(), saved);
    raw.apply(actor, action.clone()).unwrap();
    indexed.apply_index(actor, rev, index).unwrap();
    for child in [&mut raw, &mut indexed] {
        ready(child, None, |g| {
            matches!(g.state.pending, Pending::EffectDiscard { .. })
        });
        reject(child, Action::Discard { card: 0 });
        act(child, Action::Discard { card: hand });
        drain(child);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.state.players[p.other().index()].life, 12);
    assert_eq!(raw.state.players[p.index()].life, 22);
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(actor, rev, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    ready(&mut indexed, None, |g| {
        matches!(g.state.pending, Pending::EffectDiscard { .. })
    });
    act(&mut indexed, Action::Discard { card: hand });
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
#[test]
fn yathan_short_library_mills_available_without_exact_four_reflexive() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &YATHAN)]);
    for count in 0..4 {
        let p = Player::P0;
        let mut g = scenario(p);
        clear_library(&mut g, p);
        for _ in 0..count {
            put(&mut g, p, "Forest", Zone::Library);
        }
        let old = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        cast_creature(&mut g, p);
        drain(&mut g);
        assert_eq!(g.library_card_count(p), 0);
        assert_eq!(g.object_view(old).zone, Zone::Graveyard);
        assert!(!g.state.players[p.index()].attempted_empty_draw);
    }
}
#[test]
fn yathan_full_mill_targets_after_milling_and_public_exile_still_counts() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &YATHAN)]);
    for exile in [false, true] {
        let p = Player::P1;
        let mut g = scenario(p);
        clear_library(&mut g, p);
        let target = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        if exile {
            put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        }
        for _ in 0..4 {
            put(&mut g, p, "Forest", Zone::Library);
        }
        cast_creature(&mut g, p);
        ready(&mut g, None, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(target),
            })
        });
        assert_eq!(g.library_card_count(p), 0);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        drain(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
    }
}
#[test]
fn yathan_shuffle_in_place_is_no_completed_fourth_mill_and_direct_entry_does_nothing() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &YATHAN)]);
    let p = Player::P0;
    let mut g = scenario(p);
    clear_library(&mut g, p);
    let old = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
    put(&mut g, p, "Progenitus", Zone::Library);
    for _ in 0..3 {
        put(&mut g, p, "Forest", Zone::Library);
    }
    cast_creature(&mut g, p);
    drain(&mut g);
    assert_eq!(g.library_card_count(p), 1);
    assert_eq!(g.object_view(old).zone, Zone::Graveyard);
    let direct = put(&mut g, p, "Bear Cub", Zone::Graveyard);
    g.move_object(direct, Zone::Battlefield);
    drain(&mut g);
    assert_eq!(g.library_card_count(p), 1);
    assert_eq!(g.object_view(old).zone, Zone::Graveyard);
}
