//! Shared combat/entry/cast callbacks and exact token continuations; no admissions.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const WARRIOR: Card = Card::test_index(5255);
const FOOD: Card = Card::test_index(5246);
const OWN: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control()));
const fn callback(targets: TargetPlan, program: &'static [Instruction]) -> TriggeredAbility {
    TriggeredAbility {
        condition: TriggerCondition::DelayedTargetEvent,
        targets,
        program,
    }
}
const fn activated(program: &'static [Instruction]) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program,
        sorcery: false,
        once_per_turn: false,
    }
}
const ASSAULT: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                guard: StepGuard::YourMainPhase,
                event: &TriggerCondition::SelfEnters,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::CombatAndMainAfterCurrentMain(1),
                Instruction::ScheduleTurnTrigger {
                    event: TurnCallbackEvent::Attack,
                    ability: 1,
                    once: true,
                },
            ],
        },
        callback(
            TargetPlan::NONE,
            &[Instruction::UntapPlayerCreatures(PlayerSet::Controller)],
        ),
    ],
    ..CardRules::EMPTY
};
const INSTALL: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[
        activated(&[Instruction::ScheduleTurnTrigger {
            event: TurnCallbackEvent::NoncreatureCast,
            ability: 0,
            once: false,
        }]),
        activated(&[Instruction::ScheduleTurnTrigger {
            event: TurnCallbackEvent::CreatureEntry,
            ability: 1,
            once: false,
        }]),
        activated(&[Instruction::ScheduleTurnTrigger {
            event: TurnCallbackEvent::Attack,
            ability: 2,
            once: true,
        }]),
    ],
    triggers: &[
        callback(
            OWN,
            &[Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::DOUBLE_STRIKE,
            }],
        ),
        callback(
            TargetPlan::NONE,
            &[
                Instruction::LoseLife(PlayerSet::Opponent, 1),
                Instruction::GainLife(PlayerSet::Controller, 1),
            ],
        ),
        callback(
            TargetPlan::NONE,
            &[Instruction::UntapPlayerCreatures(PlayerSet::Controller)],
        ),
    ],
    ..CardRules::EMPTY
};
const DALKOVAN: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            colored: [1, 0, 0, 0, 0, 0],
            tap: true,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        ..activated(&[Instruction::ScheduleAttackTriggerThisTurn { ability: 0 }])
    }],
    triggers: &[callback(
        TargetPlan::NONE,
        &[Instruction::Mobilize {
            token: WARRIOR,
            amount: AmountValue::Fixed(2),
        }],
    )],
    ..CardRules::EMPTY
};
const DEFENDER: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::DEFENDER,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            colored: [0, 0, 0, 0, 1, 0],
            mana_component: true,
            ..ActivationCost::ZERO
        },
        ..activated(&[Instruction::AttackWithoutDefenderThisTurn])
    }],
    ..CardRules::EMPTY
};
const LEGACY_PHASE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::AdditionalCombatsAfterCurrentMain(2)],
    ),
    ..CardRules::EMPTY
};
const PAIR_PHASE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CombatAndMainAfterCurrentMain(1)],
    ),
    ..CardRules::EMPTY
};
const COUNTER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::Counter],
    ),
    ..CardRules::EMPTY
};
const SALT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::Destroy,
            Instruction::CreateTemporaryTokens {
                token: WARRIOR,
                amount: AmountValue::Fixed(2),
                keywords: Keywords::HASTE,
            },
        ],
    ),
    ..CardRules::EMPTY
};
const ADD_FOOD_SPEC: ReplacementSpec = ReplacementSpec::AddToken(FOOD);
const FOOD_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, ADD_FOOD_SPEC)],
}];
const ADD_FOOD: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: FOOD_GROUPS,
    extras: &CardExtras {
        static_ability_groups: FOOD_GROUPS,
        replacement_effects: &[ADD_FOOD_SPEC],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const SUPPRESS: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        entry_trigger_suppression: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const MARDU: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::HASTE.union(Keywords::DEATHTOUCH),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: 0,
                    other: true,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: 1,
            }),
            program: &[Instruction::ExileUntilSourceLeaves],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateLinkedExileAttackingCopies],
        },
    ],
    ..CardRules::EMPTY
};
const BLUEPRINT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 9)],
        },
    ],
    ..CardRules::EMPTY
};
fn idle(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
}
fn drain(g: &mut Game) {
    for _ in 0..120 {
        if idle(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishTargets
                    | Action::ChooseEntryTimestamp { .. }
            )
        });
    }
    panic!("drain {:?}", g.state.pending);
}
fn wait(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..100 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("wait {:?}", g.state.pending);
}
fn activate(g: &mut Game, card: CardId, ability: u16) {
    act(
        g,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
    drain(g);
}
fn install(g: &mut Game, p: Player, slot: u16) -> CardId {
    let source = put(g, p, "Ghost Vacuum", Zone::Battlefield);
    activate(g, source, slot);
    source
}
fn attack(g: &mut Game, p: Player, cards: &[CardId]) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.attack_declared = false;
    g.state.attackers.clear();
    g.state.selected_attackers.clear();
    for &card in cards {
        g.state.objects[card as usize].tapped = false;
        act(g, Action::Attack { card });
    }
    act(g, Action::FinishAttackers);
}
fn doubler(g: &mut Game, player: Player) -> CardId {
    let id = put(g, player, "Elspeth, Storm Slayer", Zone::Battlefield);
    g.set_counters(id, Counters::of(CounterKind::Loyalty, 5));
    id
}
fn tokens(g: &Game) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Battlefield && o.is_token())
        .map(|(i, _)| i as CardId)
        .collect()
}
fn next(g: &mut Game) {
    match g.state.pending {
        Pending::Priority => act(g, Action::Pass),
        Pending::Attackers => act(g, Action::FinishAttackers),
        Pending::Blockers => act(g, Action::FinishBlockers),
        Pending::OrderTriggers { .. } => choose(g, |a| matches!(a, Action::PutTrigger { .. })),
        _ => panic!("phase {:?}", g.state.pending),
    }
}
fn phase_sequence(g: &mut Game) -> Vec<Step> {
    let mut seen = Vec::new();
    let mut last = g.state.step;
    for _ in 0..240 {
        next(g);
        if g.state.step != last {
            last = g.state.step;
            if matches!(last, Step::BeginCombat | Step::Main2 | Step::End) {
                seen.push(last);
            }
            if last == Step::End {
                return seen;
            }
        }
    }
    panic!("phase sequence");
}
fn counter_packet(g: &mut Game, kind: StackKind) {
    wait(g, |g| g.state.stack.iter().any(|s| s.kind == kind));
    let id = g.state.stack.iter().find(|s| s.kind == kind).unwrap().id;
    let p = g.actor().unwrap();
    let c = put(g, p, "Opt", Zone::Hand);
    cast(g, c, Some(Target::Stack(id)));
    drain(g);
}
fn end(g: &mut Game, p: Player) {
    g.state.active = p;
    g.begin_step(Step::End);
    g.give_priority();
    drain(g);
}
fn exile(g: &mut Game, p: Player, target: CardId) -> CardId {
    let physical = g.state.objects[target as usize].is_card();
    let source = put(g, p, "Grizzly Bears", Zone::Hand);
    cast(g, source, None);
    wait(g, |g| matches!(g.state.pending, Pending::Targets { .. }));
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    drain(g);
    assert_eq!(
        g.state.objects[target as usize].zone,
        if physical { Zone::Exile } else { Zone::Outside }
    );
    source
}
fn paint(g: &mut Game, id: CardId, color: u8) {
    g.change_object_form(
        g.object_ref(id),
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        characteristic_layers::AnimationTypeMode::PreserveCardTypes,
    );
    g.state
        .roles
        .as_mut()
        .unwrap()
        .animations
        .last_mut()
        .unwrap()
        .colors = Some(color);
}

#[test]
fn combat_and_main_pairs_preserve_natural_main1_and_main2_successors() {
    let _s = fixture_rules::activate(&[(Card::Divination, &PAIR_PHASE)]);
    for p in [Player::P0, Player::P1] {
        for main in [Step::Main1, Step::Main2] {
            let mut g = scenario(p);
            g.state.step = main;
            let card = put(&mut g, p, "Divination", Zone::Hand);
            cast(&mut g, card, None);
            drain(&mut g);
            let mut expected = vec![Step::BeginCombat, Step::Main2];
            if main == Step::Main1 {
                expected.extend([Step::BeginCombat, Step::Main2]);
            }
            expected.push(Step::End);
            assert_eq!(phase_sequence(&mut g), expected);
        }
    }
}
#[test]
fn mixed_old_combats_and_new_pairs_insert_last_created_first_and_preserve_old_contract() {
    let _s = fixture_rules::activate(&[
        (Card::Divination, &PAIR_PHASE),
        (Card::Shock, &LEGACY_PHASE),
    ]);
    for pair_last in [false, true] {
        let mut g = scenario(Player::P0);
        g.state.step = Step::Main2;
        for name in if pair_last {
            ["Shock", "Divination"]
        } else {
            ["Divination", "Shock"]
        } {
            let c = put(&mut g, Player::P0, name, Zone::Hand);
            cast(&mut g, c, None);
            drain(&mut g);
        }
        let expected = if pair_last {
            vec![
                Step::BeginCombat,
                Step::Main2,
                Step::BeginCombat,
                Step::BeginCombat,
                Step::End,
            ]
        } else {
            vec![
                Step::BeginCombat,
                Step::BeginCombat,
                Step::BeginCombat,
                Step::Main2,
                Step::End,
            ]
        };
        assert_eq!(phase_sequence(&mut g), expected);
    }
}
#[test]
fn additional_pair_created_in_inserted_main_retains_original_normal_combat() {
    let _s = fixture_rules::activate(&[(Card::Divination, &PAIR_PHASE)]);
    let mut g = scenario(Player::P0);
    let c = put(&mut g, Player::P0, "Divination", Zone::Hand);
    cast(&mut g, c, None);
    drain(&mut g);
    for _ in 0..30 {
        if g.state.step == Step::Main2 {
            break;
        }
        next(&mut g);
    }
    assert_eq!(g.state.step, Step::Main2);
    g.state.players[0].mana = [8; 6];
    let c = put(&mut g, Player::P0, "Divination", Zone::Hand);
    cast(&mut g, c, None);
    drain(&mut g);
    assert_eq!(
        phase_sequence(&mut g),
        vec![
            Step::BeginCombat,
            Step::Main2,
            Step::BeginCombat,
            Step::Main2,
            Step::End
        ]
    );
}
#[test]
fn assault_intervening_main_guard_and_source_independent_next_nonempty_attack() {
    let _s = fixture_rules::activate(&[(Card::from_name("Phyrexian Arena").unwrap(), &ASSAULT)]);
    for main in [true, false] {
        let mut g = scenario(Player::P0);
        if !main {
            g.state.step = Step::BeginCombat;
        }
        let c = put(&mut g, Player::P0, "Phyrexian Arena", Zone::Hand);
        if !main {
            g.state.objects[c as usize].card = Card::from_name("Phyrexian Arena").unwrap();
            g.move_object(c, Zone::Battlefield);
            g.give_priority();
        } else {
            cast(&mut g, c, None);
        }
        drain(&mut g);
        assert_eq!(
            g.observe(Player::P0)
                .turn_callbacks
                .as_ref()
                .map_or(0, Vec::len),
            usize::from(main)
        );
        if main {
            g.move_object(c, Zone::Graveyard);
            attack(&mut g, Player::P0, &[]);
            assert_eq!(g.observe(Player::P0).turn_callbacks.unwrap().len(), 1);
            let giant = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
            let spare = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
            g.state.objects[spare as usize].tapped = true;
            attack(&mut g, Player::P0, &[giant]);
            let late = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
            g.state.objects[late as usize].tapped = true;
            drain(&mut g);
            assert!(!g.state.objects[late as usize].tapped);
            assert!(!g.state.objects[giant as usize].tapped);
            assert!(!g.state.objects[spare as usize].tapped);
            assert!(g.observe(Player::P0).turn_callbacks.is_none());
        }
    }
}
#[test]
fn countering_next_attack_callback_consumes_it_while_another_instance_remains_independent() {
    let _s = fixture_rules::activate(&[
        (Card::from_name("Ghost Vacuum").unwrap(), &INSTALL),
        (Card::from_name("Opt").unwrap(), &COUNTER),
    ]);
    for instances in [1, 2] {
        let mut g = scenario(Player::P0);
        for _ in 0..instances {
            install(&mut g, Player::P0, 2);
        }
        let giant = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
        attack(&mut g, Player::P0, &[giant]);
        counter_packet(&mut g, StackKind::TriggeredAbility { ability: 2 });
        assert_eq!(g.state.objects[giant as usize].tapped, instances == 1);
        assert!(g.observe(Player::P0).turn_callbacks.is_none());
        attack(&mut g, Player::P0, &[giant]);
        drain(&mut g);
        assert!(g.state.objects[giant as usize].tapped);
    }
}

#[test]
fn repeated_attack_registration_survives_land_departure_ignores_empty_and_stacks_instances() {
    let _s = fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &DALKOVAN)]);
    let mut g = scenario(Player::P0);
    let src = put(&mut g, Player::P0, "Ghost Vacuum", Zone::Battlefield);
    activate(&mut g, src, 0);
    g.state.objects[src as usize].tapped = false;
    activate(&mut g, src, 0);
    g.move_object(src, Zone::Graveyard);
    attack(&mut g, Player::P0, &[]);
    assert!(g.state.pending_triggers.is_empty());
    let giant = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    attack(&mut g, Player::P0, &[giant]);
    drain(&mut g);
    assert_eq!(tokens(&g).len(), 4);
    assert_eq!(g.state.attackers.len(), 5);
    assert!(
        tokens(&g)
            .iter()
            .all(|&id| g.state.objects[id as usize].tapped)
    );
    attack(&mut g, Player::P0, &[giant]);
    drain(&mut g);
    assert_eq!(tokens(&g).len(), 8);
    assert_eq!(g.observe(Player::P0).delayed_attack_this_turn.len(), 2);
    end(&mut g, Player::P1);
    assert!(tokens(&g).is_empty());
}
#[test]
fn exact_defender_permission_is_paid_keeps_keyword_and_survives_control_or_text_change() {
    let _s = fixture_rules::activate(&[(Card::from_name("Aegis Turtle").unwrap(), &DEFENDER)]);
    let mut g = scenario(Player::P0);
    let wall = put(&mut g, Player::P0, "Aegis Turtle", Zone::Battlefield);
    assert!(!g.can_declare_attacker(Player::P0, wall));
    g.state.players[0].mana = [0; 6];
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: wall,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    g.state.players[0].mana = [0, 0, 0, 0, 1, 2];
    activate(&mut g, wall, 0);
    assert!(g.keywords(wall).contains(Keywords::DEFENDER));
    assert!(g.can_declare_attacker(Player::P0, wall));
    let r = g.object_ref(wall);
    g.change_controller(r, Player::P1);
    g.state.objects[wall as usize].summoning_sick = false;
    assert!(g.can_declare_attacker(Player::P1, wall));
    g.apply_temporary_copy(r, CopyRecipe::new(Card::HillGiant), CopyDeadline::EndOfTurn);
    assert!(g.can_declare_attacker(Player::P1, wall));
    g.expire_temporary_copies(true);
    assert!(g.keywords(wall).contains(Keywords::DEFENDER));
    assert!(g.can_declare_attacker(Player::P1, wall));
    g.move_object(wall, Zone::Hand);
    g.move_object(wall, Zone::Battlefield);
    g.state.objects[wall as usize].summoning_sick = false;
    assert!(!g.can_declare_attacker(Player::P0, wall));
}
#[test]
fn cleanup_expires_callback_permissions_and_phase_insertions() {
    let _s = fixture_rules::activate(&[
        (Card::from_name("Ghost Vacuum").unwrap(), &INSTALL),
        (Card::from_name("Aegis Turtle").unwrap(), &DEFENDER),
    ]);
    let mut g = scenario(Player::P0);
    install(&mut g, Player::P0, 1);
    let wall = put(&mut g, Player::P0, "Aegis Turtle", Zone::Battlefield);
    activate(&mut g, wall, 0);
    g.add_combat_main_pairs(2);
    g.state.step = Step::Cleanup;
    g.finish_cleanup();
    assert!(g.observe(Player::P0).turn_callbacks.is_none());
    assert!(g.observe(Player::P0).attack_without_defender.is_none());
    assert!(g.observe(Player::P0).inserted_phases.is_none());
    assert!(!g.can_declare_attacker(Player::P0, wall));
}
#[test]
fn noncreature_cast_callbacks_survive_source_departure_and_choose_targets_at_trigger_announcement()
{
    let _s = fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &INSTALL)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let src = install(&mut g, p, 0);
        g.move_object(src, Zone::Graveyard);
        let creature = put(&mut g, p, "Hill Giant", Zone::Hand);
        cast(&mut g, creature, None);
        drain(&mut g);
        assert!(!g.keywords(creature).contains(Keywords::DOUBLE_STRIKE));
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        for name in ["Divination", "Short Bow"] {
            let card = put(&mut g, p, name, Zone::Hand);
            cast(&mut g, card, None);
            wait(&mut g, |g| {
                matches!(g.state.pending, Pending::Targets { .. })
            });
            assert!(g.observe(p.other()).actions.is_empty());
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(creature),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            drain(&mut g);
            assert!(g.keywords(creature).contains(Keywords::DOUBLE_STRIKE));
        }
        assert_eq!(g.observe(p).turn_callbacks.unwrap().len(), 1);
    }
}
#[test]
fn creature_entry_callbacks_are_per_actual_entrant_and_each_independent_chapter_observer() {
    let _s = fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &INSTALL)]);
    let mut g = scenario(Player::P0);
    let src = install(&mut g, Player::P0, 1);
    activate(&mut g, src, 1);
    g.move_object(src, Zone::Graveyard);
    g.create_tokens(Player::P0, WARRIOR, 3, false);
    g.give_priority();
    drain(&mut g);
    assert_eq!((g.state.players[0].life, g.state.players[1].life), (26, 14));
    g.create_tokens(Player::P1, WARRIOR, 2, false);
    g.create_tokens(Player::P0, FOOD, 2, false);
    g.give_priority();
    drain(&mut g);
    assert_eq!((g.state.players[0].life, g.state.players[1].life), (26, 14));
}
#[test]
fn delayed_creature_entry_observer_obeys_suppression_even_when_no_permanent_observer_remains() {
    let _s = fixture_rules::activate(&[
        (Card::from_name("Ghost Vacuum").unwrap(), &INSTALL),
        (Card::from_name("Short Bow").unwrap(), &SUPPRESS),
    ]);
    let mut g = scenario(Player::P0);
    let src = install(&mut g, Player::P0, 1);
    g.move_object(src, Zone::Graveyard);
    let blocker = put(&mut g, Player::P1, "Short Bow", Zone::Battlefield);
    g.create_tokens(Player::P0, WARRIOR, 2, false);
    g.give_priority();
    drain(&mut g);
    assert_eq!(g.state.players[0].life, 20);
    g.move_object(blocker, Zone::Graveyard);
    g.create_tokens(Player::P0, WARRIOR, 1, false);
    g.give_priority();
    drain(&mut g);
    assert_eq!((g.state.players[0].life, g.state.players[1].life), (21, 19));
}

#[test]
fn delayed_targeting_uses_live_or_exact_departure_source_color_after_source_reentry() {
    const RED: CardRules = CardRules {
        effect: Effect::Creature(3, 3),
        keywords: Keywords::PROTECTION_RED,
        ..CardRules::EMPTY
    };
    const BLUE: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        keywords: Keywords::PROTECTION_BLUE,
        ..CardRules::EMPTY
    };
    let _s = fixture_rules::activate(&[
        (Card::from_name("Ghost Vacuum").unwrap(), &INSTALL),
        (Card::HillGiant, &RED),
        (Card::from_name("Bear Cub").unwrap(), &BLUE),
    ]);
    for departed in [false, true] {
        let mut g = scenario(Player::P0);
        let src = install(&mut g, Player::P0, 0);
        paint(&mut g, src, 8);
        if departed {
            g.move_object(src, Zone::Hand);
            g.move_object(src, Zone::Battlefield);
            paint(&mut g, src, 2);
        }
        let red = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
        let blue = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        let c = put(&mut g, Player::P0, "Divination", Zone::Hand);
        cast(&mut g, c, None);
        wait(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(red),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(blue),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        drain(&mut g);
        assert!(g.keywords(blue).contains(Keywords::DOUBLE_STRIKE));
    }
}
#[test]
fn delayed_cast_template_does_not_observe_uncast_copies_or_retroactively_observe_prior_casts() {
    let _s = fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &INSTALL)]);
    let mut g = scenario(Player::P0);
    let c = put(&mut g, Player::P0, "Divination", Zone::Hand);
    cast(&mut g, c, None);
    let spell = g.state.stack.last().unwrap().clone();
    drain(&mut g);
    install(&mut g, Player::P0, 0);
    let copy = g.allocate_spell_copy(&spell, Player::P0);
    g.state.stack.push(copy);
    assert!(g.state.pending_triggers.is_empty());
    drain(&mut g);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
    assert_eq!(g.observe(Player::P0).turn_callbacks.unwrap().len(), 1);
}
#[test]
fn temporary_tokens_are_untapped_hasty_and_created_when_legal_target_survives_indestructible() {
    const INDESTRUCTIBLE: CardRules = CardRules {
        effect: Effect::Creature(3, 3),
        keywords: Keywords::INDESTRUCTIBLE,
        ..CardRules::EMPTY
    };
    let _s = fixture_rules::activate(&[
        (Card::from_name("Murder").unwrap(), &SALT),
        (Card::HillGiant, &INDESTRUCTIBLE),
    ]);
    for illegal in [false, true] {
        let mut g = scenario(Player::P0);
        let victim = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
        let c = put(&mut g, Player::P0, "Murder", Zone::Hand);
        cast(&mut g, c, Some(Target::Card(victim)));
        if illegal {
            g.move_object(victim, Zone::Hand);
        }
        drain(&mut g);
        assert_eq!(tokens(&g).len(), if illegal { 0 } else { 2 });
        for id in tokens(&g) {
            assert!(!g.state.objects[id as usize].tapped);
            assert!(g.keywords(id).contains(Keywords::HASTE));
        }
        if !illegal {
            assert_eq!(g.state.objects[victim as usize].zone, Zone::Battlefield);
            end(&mut g, Player::P1);
            assert!(tokens(&g).is_empty());
        }
    }
}
#[test]
fn temporary_token_results_include_added_and_doubled_groups_and_delays_are_one_shot() {
    let _s = fixture_rules::activate(&[
        (Card::from_name("Murder").unwrap(), &SALT),
        (Card::from_name("Ghost Vacuum").unwrap(), &ADD_FOOD),
    ]);
    let mut g = scenario(Player::P0);
    doubler(&mut g, Player::P0);
    put(&mut g, Player::P0, "Ghost Vacuum", Zone::Battlefield);
    let victim = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    let c = put(&mut g, Player::P0, "Murder", Zone::Hand);
    cast(&mut g, c, Some(Target::Card(victim)));
    wait(&mut g, |g| {
        matches!(g.state.pending, Pending::Replacement { .. })
    });
    let add = g.legal_actions().into_iter().find(|a| matches!(a,Action::ChooseReplacement{source,..} if g.state.objects[source.card as usize].card==Card::from_name("Ghost Vacuum").unwrap())).unwrap();
    act(&mut g, add);
    drain(&mut g);
    let made = tokens(&g);
    assert_eq!(made.len(), 6);
    assert_eq!(g.observe(Player::P0).delayed_sacrifices[0].tokens.len(), 6);
    assert!(
        made.iter()
            .all(|&id| g.keywords(id).contains(Keywords::HASTE))
    );
    let stolen = made[0];
    g.change_controller(g.object_ref(stolen), Player::P1);
    end(&mut g, Player::P0);
    assert_eq!(tokens(&g), vec![stolen]);
    g.change_controller(g.object_ref(stolen), Player::P0);
    end(&mut g, Player::P0);
    assert_eq!(tokens(&g), vec![stolen]);
}
#[test]
fn temporary_token_keywords_and_delay_wait_for_as_entry_copy_choices_and_actual_recipients() {
    const COPY: CardRules = CardRules {
        effect: Effect::Creature(1, 1),
        extras: &CardExtras {
            entry_copy: Some(EntryCopySpec {
                filters: &[BattlefieldFilter {
                    kinds: 1,
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
    let _s = fixture_rules::activate(&[
        (Card::from_name("Murder").unwrap(), &SALT),
        (WARRIOR, &COPY),
    ]);
    let mut g = scenario(Player::P0);
    let own = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let victim = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    let c = put(&mut g, Player::P0, "Murder", Zone::Hand);
    cast(&mut g, c, Some(Target::Card(victim)));
    wait(&mut g, |g| {
        matches!(g.state.pending, Pending::EntryCopy { .. })
    });
    assert!(tokens(&g).is_empty());
    assert!(g.observe(Player::P0).delayed_sacrifices.is_empty());
    for _ in 0..2 {
        let object = g.object_ref(own);
        act(
            &mut g,
            Action::ChooseEntryCopy {
                object: Some(object),
            },
        );
    }
    drain(&mut g);
    let made = tokens(&g);
    assert_eq!(made.len(), 2);
    for id in made {
        assert_eq!(stats(&g, id), (2, 2));
        assert!(g.keywords(id).contains(Keywords::HASTE));
        assert!(!g.state.objects[id as usize].tapped);
    }
    assert_eq!(g.observe(Player::P0).delayed_sacrifices[0].tokens.len(), 2);
    end(&mut g, Player::P0);
    assert!(tokens(&g).is_empty());
}
#[test]
fn linked_card_copy_uses_printed_blueprint_enters_tapped_attacking_without_attack_trigger_and_waits_for_own_end()
 {
    let _s = fixture_rules::activate(&[
        (Card::GrizzlyBears, &MARDU),
        (Card::from_name("Bear Cub").unwrap(), &BLUEPRINT),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let blueprint = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.add_counter(blueprint, CounterKind::PlusOne, 5);
        g.state.objects[blueprint as usize].growth.power = 8;
        let src = exile(&mut g, p, blueprint);
        attack(&mut g, p, &[src]);
        drain(&mut g);
        let made = tokens(&g);
        assert_eq!(made.len(), 1);
        assert_eq!(stats(&g, made[0]), (2, 2));
        assert!(g.state.objects[made[0] as usize].tapped);
        assert!(g.state.attackers.iter().any(|a|a.card==made[0]&&a.defender==Some(CombatDefender::Player(p.other()))));
        assert_eq!(g.state.players[p.index()].life, 21);
        end(&mut g, p.other());
        assert_eq!(tokens(&g), made);
        end(&mut g, p);
        assert!(tokens(&g).is_empty());
        g.move_object(src, Zone::Graveyard);
        assert_eq!(g.state.objects[blueprint as usize].zone, Zone::Battlefield);
        assert_eq!(g.counter_count(blueprint, CounterKind::PlusOne), 0);
    }
}
#[test]
fn linked_etb_never_exiles_after_source_departure_and_exiled_tokens_never_supply_copies() {
    let _s = fixture_rules::activate(&[(Card::GrizzlyBears, &MARDU)]);
    let mut g = scenario(Player::P0);
    let target = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let src = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
    cast(&mut g, src, None);
    wait(&mut g, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    g.move_object(src, Zone::Graveyard);
    drain(&mut g);
    assert_eq!(g.state.objects[target as usize].zone, Zone::Battlefield);
    assert!(g.state.exile_links.is_empty());
    let mut g = scenario(Player::P0);
    g.create_tokens(Player::P0, WARRIOR, 1, false);
    g.give_priority();
    drain(&mut g);
    let token = tokens(&g)[0];
    let src = exile(&mut g, Player::P0, token);
    assert_eq!(g.state.objects[token as usize].zone, Zone::Outside);
    attack(&mut g, Player::P0, &[src]);
    drain(&mut g);
    assert!(tokens(&g).is_empty());
}
#[test]
fn copied_exile_triggers_link_multiple_cards_and_copy_each_in_one_replacement_event() {
    let _s = fixture_rules::activate(&[(Card::GrizzlyBears, &MARDU)]);
    let mut g = scenario(Player::P0);
    let first = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let src = exile(&mut g, Player::P0, first);
    let second = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    let mut packet = g.activation_item(Player::P0, src, 0);
    packet.kind = StackKind::TriggeredAbility { ability: 0 };
    packet.targets = vec![g.bind_target(Target::Card(second))];
    g.exile_until_source_leaves(&packet);
    assert_eq!(g.state.exile_links.len(), 2);
    doubler(&mut g, Player::P0);
    attack(&mut g, Player::P0, &[src]);
    drain(&mut g);
    let made = tokens(&g);
    assert_eq!(made.len(), 4);
    assert_eq!(
        made.iter()
            .filter(|&&id| g.object_view(id).card == Card::from_name("Bear Cub").unwrap())
            .count(),
        2
    );
    assert_eq!(
        made.iter()
            .filter(|&&id| g.object_view(id).card == Card::HillGiant)
            .count(),
        2
    );
    assert_eq!(g.observe(Player::P0).delayed_sacrifices[0].tokens.len(), 4);
}
#[test]
fn exiled_noncreature_physical_card_makes_tapped_nonattacking_copy_even_if_previously_animated() {
    let _s = fixture_rules::activate(&[
        (Card::GrizzlyBears, &MARDU),
        (Card::from_name("Ghost Vacuum").unwrap(), &ADD_FOOD),
    ]);
    let mut g = scenario(Player::P0);
    let artifact = put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
    g.change_object_form(
        g.object_ref(artifact),
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((4, 4)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        characteristic_layers::AnimationTypeMode::AddCreature,
    );
    let src = exile(&mut g, Player::P0, artifact);
    put(&mut g, Player::P0, "Ghost Vacuum", Zone::Battlefield);
    attack(&mut g, Player::P0, &[src]);
    drain(&mut g);
    let made = tokens(&g);
    assert_eq!(made.len(), 2, "the artifact event still receives AddToken");
    assert_eq!(
        made.iter()
            .filter(|&&id| g.object_view(id).card == Card::from_name("Short Bow").unwrap())
            .count(),
        1
    );
    assert_eq!(
        made.iter()
            .filter(|&&id| g.object_view(id).card == FOOD)
            .count(),
        1
    );
    for &id in &made {
        assert!(!g.is_creature(id));
        assert!(g.state.objects[id as usize].tapped);
        assert!(g.state.attackers.iter().all(|a| a.card != id));
    }
    assert_eq!(g.observe(Player::P0).delayed_sacrifices[0].tokens.len(), 2);
    end(&mut g, Player::P0);
    assert!(tokens(&g).is_empty());
}
#[test]
fn linked_nonpermanent_manifested_cards_create_no_tokens_or_replacement_event() {
    let _s = fixture_rules::activate(&[
        (Card::GrizzlyBears, &MARDU),
        (Card::from_name("Ghost Vacuum").unwrap(), &ADD_FOOD),
    ]);
    for p in [Player::P0, Player::P1] {
        for name in ["Opt", "Divination"] {
            let mut g = scenario(p);
            let physical = put(&mut g, p, name, Zone::Library);
            let entry = g.manifest_entry(g.object_ref(physical), p).unwrap();
            g.stage_entries(vec![entry], None);
            assert!(!g.advance_entries());
            assert!(g.is_face_down(g.object_ref(physical)));
            assert!(g.is_creature(physical));
            let source = exile(&mut g, p, physical);
            assert_eq!(g.object_view(physical).card, Card::from_name(name).unwrap());
            assert!(!g.is_creature(physical));
            assert_eq!(g.state.exile_links.len(), 1);
            let link = &g.state.exile_links[0];
            assert_eq!(link.source, g.object_ref(source));
            assert_eq!(link.exiled, g.object_ref(physical));
            assert_eq!(link.ability_origin, None);
            put(&mut g, p, "Ghost Vacuum", Zone::Battlefield);
            let objects = g.state.objects.len();
            attack(&mut g, p, &[source]);
            drain(&mut g);
            assert!(
                tokens(&g).is_empty(),
                "CR111.5 rejects {name} before AddToken"
            );
            assert!(g.state.replacements.is_none());
            assert!(g.observe(p).delayed_sacrifices.is_empty());
            assert_eq!(
                g.state.objects.len(),
                objects,
                "no token slot was allocated"
            );
            assert_eq!(g.state.exile_links.len(), 1);
            assert_eq!(g.state.objects[physical as usize].zone, Zone::Exile);
        }
    }
}
#[test]
fn linked_nonpermanent_double_face_front_cannot_use_its_creature_back_to_create_a_token() {
    let _s = fixture_rules::activate(&[
        (Card::GrizzlyBears, &MARDU),
        (Card::from_name("Ghost Vacuum").unwrap(), &ADD_FOOD),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        put(&mut g, p, "Ghost Vacuum", Zone::Battlefield);
        let front = Card::from_name("Opt").unwrap();
        let back = Card::from_name("Bear Cub").unwrap();
        // The pinned catalogue has no instant/sorcery-front modal DFC. Use a
        // complete typed two-face recipe to test the same pre-event boundary.
        let recipe = CopyRecipe::new(front).with_faces(copies::CopyFaces::Double {
            physical_front: front,
            front,
            back,
            back_up: false,
        });
        assert!(!recipe.definition.is_creature());
        assert!(back.is_creature());
        let mut item = g.activation_item(p, source, 0);
        item.kind = StackKind::TriggeredAbility { ability: 1 };
        let before = g.snapshot();
        g.stage_linked_attacking_recipes(&item, vec![recipe]);
        assert!(
            g.state.replacements.is_none(),
            "no creation replacement event"
        );
        assert_eq!(g.snapshot(), before, "no object, entry or delayed packet");
    }
}
#[test]
fn linked_nonpermanent_filter_preserves_valid_recipe_alignment_and_ability_origin() {
    let _s = fixture_rules::activate(&[
        (Card::GrizzlyBears, &MARDU),
        (Card::HillGiant, &MARDU),
        (Card::from_name("Ghost Vacuum").unwrap(), &ADD_FOOD),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let physical = put(&mut g, p, "Opt", Zone::Library);
    let entry = g.manifest_entry(g.object_ref(physical), p).unwrap();
    g.stage_entries(vec![entry], None);
    assert!(!g.advance_entries());
    let source = exile(&mut g, p, physical);
    let native = g.object_ref(source);
    g.apply_temporary_copy(
        native,
        CopyRecipe::new(Card::HillGiant),
        CopyDeadline::EndOfTurn,
    );
    let origin = g.copy_ability_origin(native);
    assert!(origin.is_some());
    let foreign = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let mut item = g.activation_item(p, source, 0);
    item.kind = StackKind::TriggeredAbility { ability: 0 };
    item.source_ability_origin = origin;
    item.targets = vec![g.bind_target(Target::Card(foreign))];
    g.exile_until_source_leaves(&item);
    g.expire_temporary_copies(true);
    assert_eq!(g.copy_ability_origin(native), None);
    let valid = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let mut item = g.activation_item(p, source, 0);
    item.kind = StackKind::TriggeredAbility { ability: 0 };
    item.targets = vec![g.bind_target(Target::Card(valid))];
    g.exile_until_source_leaves(&item);
    assert_eq!(g.state.exile_links.len(), 3);
    put(&mut g, p, "Ghost Vacuum", Zone::Battlefield);
    attack(&mut g, p, &[source]);
    drain(&mut g);
    let made = tokens(&g);
    assert_eq!(
        made.len(),
        2,
        "only matching permanent recipe plus added Food"
    );
    assert_eq!(
        made.iter()
            .filter(|&&id| g.object_view(id).card == Card::from_name("Bear Cub").unwrap())
            .count(),
        1
    );
    assert_eq!(
        made.iter()
            .filter(|&&id| g.object_view(id).card == FOOD)
            .count(),
        1
    );
    assert!(
        made.iter()
            .all(|&id| g.object_view(id).card != Card::HillGiant)
    );
    assert_eq!(g.observe(p).delayed_sacrifices[0].tokens.len(), 2);
}

#[test]
fn linked_original_and_fresh_copy_template_origins_remain_separate_and_departure_returns_all() {
    let _s = fixture_rules::activate(&[(Card::GrizzlyBears, &MARDU), (Card::HillGiant, &MARDU)]);
    let mut g = scenario(Player::P0);
    let first = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let src = exile(&mut g, Player::P0, first);
    let native = g.object_ref(src);
    g.apply_temporary_copy(
        native,
        CopyRecipe::new(Card::HillGiant),
        CopyDeadline::EndOfTurn,
    );
    let origin = g.copy_ability_origin(native);
    assert!(origin.is_some());
    let mut attack_packet = g.activation_item(Player::P0, src, 0);
    attack_packet.kind = StackKind::TriggeredAbility { ability: 1 };
    attack_packet.source_ability_origin = origin;
    g.create_linked_exile_attacking_copies(&attack_packet);
    assert!(tokens(&g).is_empty());
    let second = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    let mut copied = g.activation_item(Player::P0, src, 0);
    copied.kind = StackKind::TriggeredAbility { ability: 0 };
    copied.source_ability_origin = origin;
    copied.targets = vec![g.bind_target(Target::Card(second))];
    g.exile_until_source_leaves(&copied);
    assert_eq!(g.state.exile_links.len(), 2);
    g.expire_temporary_copies(true);
    assert_eq!(g.copy_ability_origin(native), None);
    let mut native_packet = g.activation_item(Player::P0, src, 0);
    native_packet.kind = StackKind::TriggeredAbility { ability: 1 };
    g.create_linked_exile_attacking_copies(&native_packet);
    g.advance_replacements(false);
    drain(&mut g);
    assert_eq!(tokens(&g).len(), 1);
    assert_eq!(
        g.object_view(tokens(&g)[0]).card,
        Card::from_name("Bear Cub").unwrap()
    );
    g.apply_temporary_copy(
        native,
        CopyRecipe::new(Card::HillGiant),
        CopyDeadline::EndOfTurn,
    );
    assert_ne!(g.copy_ability_origin(native), origin);
    let mut fresh = g.activation_item(Player::P0, src, 0);
    fresh.kind = StackKind::TriggeredAbility { ability: 1 };
    fresh.source_ability_origin = g.copy_ability_origin(native);
    g.create_linked_exile_attacking_copies(&fresh);
    g.advance_replacements(false);
    drain(&mut g);
    assert_eq!(
        tokens(&g).len(),
        1,
        "a new copied template cannot use original or older copied links"
    );
    g.create_linked_exile_attacking_copies(&attack_packet);
    g.advance_replacements(false);
    drain(&mut g);
    assert_eq!(tokens(&g).len(), 2);
    g.move_object(src, Zone::Graveyard);
    assert_eq!(g.state.objects[first as usize].zone, Zone::Battlefield);
    assert_eq!(g.state.objects[second as usize].zone, Zone::Battlefield);
}

#[test]
fn countered_delayed_sacrifice_is_consumed_and_old_token_identity_cannot_hit_reused_slot() {
    let _s = fixture_rules::activate(&[
        (Card::from_name("Murder").unwrap(), &SALT),
        (Card::from_name("Opt").unwrap(), &COUNTER),
    ]);
    let mut g = scenario(Player::P0);
    let victim = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    let c = put(&mut g, Player::P0, "Murder", Zone::Hand);
    cast(&mut g, c, Some(Target::Card(victim)));
    drain(&mut g);
    let made = tokens(&g);
    let departed = g.object_ref(made[0]);
    g.move_object(made[0], Zone::Exile);
    g.state_based_actions();
    g.create_tokens(Player::P0, WARRIOR, 1, false);
    g.give_priority();
    drain(&mut g);
    let replacement = tokens(&g)
        .into_iter()
        .find(|&id| id == departed.card)
        .unwrap();
    assert_ne!(g.object_ref(replacement), departed);
    g.state.active = Player::P0;
    g.begin_step(Step::End);
    g.give_priority();
    counter_packet(&mut g, StackKind::DelayedSacrifice);
    assert!(g.observe(Player::P0).delayed_sacrifices.is_empty());
    end(&mut g, Player::P0);
    assert_eq!(tokens(&g).len(), 2);
    assert!(tokens(&g).contains(&replacement));
}
#[test]
fn callback_target_choice_is_owned_transactional_and_restorable_after_parent_disposal() {
    let _s = fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &INSTALL)]);
    let mut parent = scenario(Player::P0);
    let src = install(&mut parent, Player::P0, 0);
    parent.move_object(src, Zone::Graveyard);
    let first = put(&mut parent, Player::P0, "Hill Giant", Zone::Battlefield);
    let second = put(&mut parent, Player::P0, "Bear Cub", Zone::Battlefield);
    let c = put(&mut parent, Player::P0, "Divination", Zone::Hand);
    cast(&mut parent, c, None);
    wait(&mut parent, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    let action = Action::ChooseTarget {
        target: Target::Card(first),
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
    assert_eq!(
        raw.apply(Player::P1, action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(raw.snapshot(), saved);
    reject(
        &mut raw,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    act(&mut raw, action.clone());
    indexed.apply_index(Player::P0, revision, index).unwrap();
    for g in [&mut raw, &mut indexed] {
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(g, Action::FinishTargets);
        }
        drain(g);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    act(
        &mut sibling,
        Action::ChooseTarget {
            target: Target::Card(second),
        },
    );
    if sibling.legal_actions().contains(&Action::FinishTargets) {
        act(&mut sibling, Action::FinishTargets);
    }
    drain(&mut sibling);
    assert!(sibling.keywords(second).contains(Keywords::DOUBLE_STRIKE));
    assert!(!sibling.keywords(first).contains(Keywords::DOUBLE_STRIKE));
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(Player::P0, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    if indexed.legal_actions().contains(&Action::FinishTargets) {
        act(&mut indexed, Action::FinishTargets);
    }
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
#[test]
fn temporary_token_entry_choice_preserves_owned_completion_and_real_recipients_after_parent_disposal()
 {
    const COPY: CardRules = CardRules {
        effect: Effect::Creature(1, 1),
        extras: &CardExtras {
            entry_copy: Some(EntryCopySpec {
                filters: &[BattlefieldFilter {
                    kinds: 1,
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
    let _s = fixture_rules::activate(&[
        (Card::from_name("Murder").unwrap(), &SALT),
        (WARRIOR, &COPY),
    ]);
    let mut parent = scenario(Player::P0);
    let own = put(&mut parent, Player::P0, "Bear Cub", Zone::Battlefield);
    let victim = put(&mut parent, Player::P1, "Hill Giant", Zone::Battlefield);
    let c = put(&mut parent, Player::P0, "Murder", Zone::Hand);
    cast(&mut parent, c, Some(Target::Card(victim)));
    wait(&mut parent, |g| {
        matches!(g.state.pending, Pending::EntryCopy { .. })
    });
    let action = Action::ChooseEntryCopy {
        object: Some(parent.object_ref(own)),
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
    assert_eq!(
        raw.apply(Player::P1, action.clone()),
        Err(Error::WrongActor)
    );
    reject(
        &mut raw,
        Action::ChooseEntryCopy {
            object: Some(ObjectRef {
                card: own,
                generation: 9999,
            }),
        },
    );
    assert_eq!(raw.snapshot(), saved);
    act(&mut raw, action.clone());
    indexed.apply_index(Player::P0, revision, index).unwrap();
    for g in [&mut raw, &mut indexed] {
        act(g, Action::ChooseEntryCopy { object: None });
        drain(g);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert!(
        tokens(&raw)
            .iter()
            .all(|&id| raw.keywords(id).contains(Keywords::HASTE))
    );
    for _ in 0..2 {
        act(&mut sibling, Action::ChooseEntryCopy { object: None });
    }
    drain(&mut sibling);
    assert_ne!(raw.snapshot(), sibling.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    act(&mut indexed, Action::ChooseEntryCopy { object: None });
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    end(&mut indexed, Player::P1);
    assert!(tokens(&indexed).is_empty());
}
#[test]
fn combat_callback_layout_and_absent_wire_remain_cold() {
    let g = scenario(Player::P0);
    let json = serde_json::to_value(g.observe(Player::P0)).unwrap();
    for key in [
        "inserted_phases",
        "turn_callbacks",
        "attack_without_defender",
    ] {
        assert!(json.get(key).is_none());
    }
    let sizes = format!(
        "Object={} ObjectView={} State={} DamageSource={}\n",
        std::mem::size_of::<Object>(),
        std::mem::size_of::<ObjectView>(),
        std::mem::size_of::<State>(),
        std::mem::size_of::<DamageSource>()
    );
    eprintln!("tdm-combat-callback layout: {sizes}");
    if let Ok(path) = std::env::var("RUSTMAGE_TDM_COMBAT_CALLBACK_LAYOUT") {
        std::fs::write(path, sizes).unwrap();
    }
}

#[test]
fn phase_continuations_are_owned_raw_index_and_restorable_after_parent_disposal() {
    let _s = fixture_rules::activate(&[(Card::Divination, &PAIR_PHASE)]);
    let mut parent = scenario(Player::P0);
    let c = put(&mut parent, Player::P0, "Divination", Zone::Hand);
    cast(&mut parent, c, None);
    drain(&mut parent);
    let action = Action::Pass;
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
    assert_eq!(phase_sequence(&mut raw), phase_sequence(&mut indexed));
    assert_eq!(raw.snapshot(), indexed.snapshot());
    sibling.add_combat_main_pairs(1);
    assert_eq!(
        phase_sequence(&mut sibling)
            .iter()
            .filter(|&&step| step == Step::BeginCombat)
            .count(),
        3
    );
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(Player::P0, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    phase_sequence(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}

#[test]
fn delayed_noncreature_cast_uses_chosen_adventure_face_and_survives_countered_original() {
    const ADVENTURE: CardRules = CardRules {
        effect: Effect::Creature(3, 3),
        extras: &CardExtras {
            adventure_spell: Some(Card::Shock),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _s = fixture_rules::activate(&[
        (Card::from_name("Ghost Vacuum").unwrap(), &INSTALL),
        (Card::HillGiant, &ADVENTURE),
    ]);
    let mut g = scenario(Player::P0);
    install(&mut g, Player::P0, 0);
    let own = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let original = put(&mut g, Player::P0, "Hill Giant", Zone::Hand);
    act(&mut g, Action::AnnounceAdventure { card: original });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    pay(&mut g);
    wait(&mut g, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(own),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    if g.actor() != Some(Player::P1) {
        act(&mut g, Action::Pass);
    }
    let cancel = put(&mut g, Player::P1, "Cancel", Zone::Hand);
    cast(&mut g, cancel, Some(Target::Spell(original)));
    drain(&mut g);
    assert!(g.keywords(own).contains(Keywords::DOUBLE_STRIKE));
    assert_eq!(g.state.players[1].life, 20);
    assert_eq!(g.state.objects[original as usize].zone, Zone::Graveyard);
}

#[test]
fn mobilize_battle_without_planeswalker_opens_existing_defender_choice_and_keeps_each_token_assignment()
 {
    let _s = fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &DALKOVAN)]);
    let mut g = scenario(Player::P0);
    let src = put(&mut g, Player::P0, "Ghost Vacuum", Zone::Battlefield);
    activate(&mut g, src, 0);
    let siege = put(&mut g, Player::P1, "Goblin Piker", Zone::Battlefield);
    let mut recipe = CopyRecipe::new(Card::GoblinPiker);
    recipe.overrides = Some(CopyOverrides {
        kinds: Some(BATTLE_KIND),
        ..CopyOverrides::NONE
    });
    g.apply_temporary_copy(g.object_ref(siege), recipe, CopyDeadline::EndOfTurn);
    g.add_counter(siege, CounterKind::Defense, 9);
    g.set_battle_protector(g.object_ref(siege), Player::P1);
    assert!(g.battle_can_be_attacked_by(g.object_ref(siege), Player::P0));
    assert!(
        g.state
            .objects
            .iter()
            .enumerate()
            .all(|(i, o)| o.zone != Zone::Battlefield || g.current_kinds(i as CardId) & 64 == 0)
    );
    let giant = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    attack(&mut g, Player::P0, &[giant]);
    wait(&mut g, |g| {
        matches!(g.state.pending, Pending::Mobilize { .. }) || tokens(g).len() == 2
    });
    assert!(
        matches!(g.state.pending, Pending::Mobilize { player: Player::P0 }),
        "legal battle must expose the real defender chooser"
    );
    let action = Action::ChooseMobilizeDefender {
        defender: CombatDefender::Battle(g.object_ref(siege)),
    };
    assert!(g.legal_actions().contains(&action));
    reject(
        &mut g,
        Action::ChooseMobilizeDefender {
            defender: CombatDefender::Battle(ObjectRef {
                card: siege,
                generation: 9999,
            }),
        },
    );
    act(&mut g, action);
    act(
        &mut g,
        Action::ChooseMobilizeDefender {
            defender: CombatDefender::Player(Player::P1),
        },
    );
    drain(&mut g);
    let made = tokens(&g);
    assert_eq!(made.len(), 2);
    assert_eq!(
        g.state
            .attackers
            .iter()
            .filter(|a| made.contains(&a.card)
                && a.defender == Some(CombatDefender::Battle(g.object_ref(siege))))
            .count(),
        1
    );
    assert_eq!(
        g.state
            .attackers
            .iter()
            .filter(|a| made.contains(&a.card)
                && a.defender == Some(CombatDefender::Player(Player::P1)))
            .count(),
        1
    );
}

#[test]
fn printed_storm_with_a_live_token_doubler_still_creates_one_spell_copy_and_positive_creation_control_doubles()
 {
    const STORM: CardRules = CardRules {
        effect: Effect::Creature(4, 4),
        keywords: Keywords::FLYING,
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::SelfCast,
            targets: TargetPlan::NONE,
            program: &[Instruction::CopyCapturedSpell {
                count: SpellCopyCount::EarlierSpellsThisTurn,
                retarget: true,
            }],
        }],
        ..CardRules::EMPTY
    };
    let dragon = Card::from_name("Shivan Dragon").unwrap();
    let _s = fixture_rules::activate(&[(dragon, &STORM)]);
    let mut g = scenario(Player::P0);
    let source = doubler(&mut g, Player::P0);
    let prior = put(&mut g, Player::P0, "Divination", Zone::Hand);
    cast(&mut g, prior, None);
    drain(&mut g);
    let original = put(&mut g, Player::P0, "Shivan Dragon", Zone::Hand);
    cast(&mut g, original, None);
    drain(&mut g);
    assert_eq!(g.state.objects[source as usize].zone, Zone::Battlefield);
    assert_eq!(g.counter_count(source, CounterKind::Loyalty), 5);
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.card == dragon)
            .count(),
        2
    );
    g.create_tokens(Player::P0, WARRIOR, 1, false);
    g.advance_replacements(false);
    g.give_priority();
    drain(&mut g);
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.card == WARRIOR)
            .count(),
        2,
        "the same live doubler must actually double a creation event"
    );
}
