//! Suspect designations follow exact battlefield objects through shared rules.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const CREATURE_TARGET: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Creature));
const ATTACHED_PROHIBITION: CardRules = CardRules {
    effect: Effect::Aura(CREATURE_TARGET),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AttachedSuspectProhibited],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Unsuspect {
            subject: Subject::Attached,
        }],
    }],
    ..CardRules::EMPTY
};
const SUSPECT: CardRules = CardRules {
    effect: Effect::Spell(
        CREATURE_TARGET,
        &[Instruction::Suspect {
            subject: Subject::Targets,
        }],
    ),
    ..CardRules::EMPTY
};
const SUSPECTED_TARGET: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Creature,
            predicate: &TargetPredicate::Suspected,
        })),
        &[
            Instruction::IfSubjectSuspected {
                subject: Subject::Targets,
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 1),
            Instruction::Unsuspect {
                subject: Subject::Targets,
            },
        ],
    ),
    ..CardRules::EMPTY
};

const OTHER_CREATURE: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 1,
    nonland: false,
    other: true,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const SACRIFICE: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    cycling: false,
    origin: ActivationOrigin::Battlefield,
    cost: ActivationCost {
        generic: 1,
        mana_component: true,
        additional: Some(AdditionalCost {
            kind: EffectChoiceKind::Sacrifice,
            filter: OTHER_CREATURE,
        }),
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[
        Instruction::GainLife(PlayerSet::Controller, 1),
        Instruction::IfSacrificedSuspected { skip: 1 },
        Instruction::GainLife(PlayerSet::Controller, 1),
    ],
    sorcery: false,
    once_per_turn: false,
};
const SACRIFICE_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[
        SACRIFICE,
        ActivatedAbility {
            cost: ActivationCost {
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::FilteredCost {
                        action: CostAction::Sacrifice,
                        any: &[CostMatch::Suspected],
                    },
                    filter: OTHER_CREATURE,
                }),
                ..SACRIFICE.cost
            },
            ..SACRIFICE
        },
    ],
    ..CardRules::EMPTY
};

const ATTACK_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledSuspectedCreaturesAttack,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const ENTRY_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::Expression {
                value: StatValue::SourceSuspected,
                minimum: Some(1),
                maximum: None,
            },
            event: &TriggerCondition::FilteredEntersBatchYou {
                predicate: EntryPredicate::CreatureWithKeyword(Keywords::NONE),
                other: true,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ChooseObjects {
            kind: EffectChoiceKind::TransferSuspectToEventCreature,
            players: PlayerSet::Controller,
            filter: OTHER_CREATURE,
            amount: 1,
            optional_skip: Some(0),
        }],
    }],
    ..CardRules::EMPTY
};
const DETECTIVE: Card = Card::from_catalogue_id(5329);
const CREATE_PAIR: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokens {
            token: DETECTIVE,
            amount: 2,
            tapped: false,
        }],
    ),
    ..CardRules::EMPTY
};
const CREATE_THEN_SUSPECT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokensThenSuspect {
            token: DETECTIVE,
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};
const TOKEN_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::Expression {
                value: StatValue::SourceSuspected,
                minimum: Some(1),
                maximum: None,
            },
            event: &TriggerCondition::SelfEnters,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn suspect(game: &mut Game, player: Player, id: CardId) {
    let spell = put(game, player, "Opt", Zone::Hand);
    cast(game, spell, Some(Target::Card(id)));
    settle(game);
}

#[test]
fn an_attachment_can_clear_and_prohibit_suspect_on_the_exact_object() {
    let _rules = fixture_rules::activate(&[
        (named("Pacifism"), &ATTACHED_PROHIBITION),
        (named("Opt"), &SUSPECT),
        (named("Shock"), &SUSPECTED_TARGET),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let other = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let aura = put(&mut game, player, "Pacifism", Zone::Hand);
    let bear = game.object_ref(creature);
    suspect(&mut game, player, creature);
    assert_eq!(game.observe(player).suspected, [bear]);

    cast(&mut game, aura, Some(Target::Card(creature)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(aura).attached_to, Some(bear));
    assert_eq!(game.observe(player).suspected, [bear]);
    settle(&mut game);
    suspect(&mut game, player, creature);
    assert!(game.observe(player.other()).suspected.is_empty());

    suspect(&mut game, player, other);
    let payoff = put(&mut game, player, "Shock", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: payoff });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(creature),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(other),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 21);
    assert!(game.observe(player).suspected.is_empty());

    let disenchant = put(&mut game, player, "Disenchant", Zone::Hand);
    cast(&mut game, disenchant, Some(Target::Card(aura)));
    settle(&mut game);
    suspect(&mut game, player, creature);
    game.record_control(bear, player.other(), None);
    for controller in [player, player.other()] {
        assert_eq!(
            game.stat_value(
                StatValue::ControlledSuspectedSubtype("Bear"),
                bear,
                controller,
            ),
            i128::from(controller == player.other()),
        );
    }
    game.move_object(creature, Zone::Hand);
    game.move_object(creature, Zone::Battlefield);
    assert_ne!(game.object_ref(creature), bear);
    assert!(game.observe(player).suspected.is_empty());
}

fn start_sacrifice(game: &mut Game, card: CardId, ability: u16) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
}

#[test]
fn paying_a_sacrifice_reads_the_designation_through_ability_removal_and_exile() {
    let _rules = fixture_rules::activate(&[(named("Bear Cub"), &SACRIFICE_OBSERVER)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let marked = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let menacing = put(
        &mut game,
        player,
        "Prowler, Clawed Thief",
        Zone::Battlefield,
    );
    let mark = game.object_ref(marked);
    let menace = game.object_ref(menacing);
    game.set_suspected(mark, true);
    let blank = put(&mut game, player, "Witness Protection", Zone::Hand);
    cast(&mut game, blank, Some(Target::Card(marked)));
    settle(&mut game);
    assert!(!game.keywords(marked).contains(Keywords::MENACE));
    assert!(game.keywords(menacing).contains(Keywords::MENACE));
    assert_eq!(game.observe(player.other()).suspected, [mark]);
    put(
        &mut game,
        player.other(),
        "Rest in Peace",
        Zone::Battlefield,
    );

    let mut ordinary = game.clone();
    start_sacrifice(&mut ordinary, source, 0);
    act(
        &mut ordinary,
        Action::ChooseAdditionalCost { object: menace },
    );
    pay(&mut ordinary);
    let paid = ordinary
        .state
        .stack
        .last()
        .unwrap()
        .paid_additional_cost
        .as_ref()
        .unwrap();
    assert!(!paid.sacrificed_suspected);
    settle(&mut ordinary);
    assert_eq!(ordinary.state.players[player.index()].life, 21);
    assert_eq!(ordinary.object_view(menacing).zone, Zone::Exile);

    start_sacrifice(&mut game, source, 1);
    reject(&mut game, Action::ChooseAdditionalCost { object: menace });
    act(&mut game, Action::ChooseAdditionalCost { object: mark });
    pay(&mut game);
    let paid = game
        .state
        .stack
        .last()
        .unwrap()
        .paid_additional_cost
        .as_ref()
        .unwrap();
    assert_eq!(paid.count, 1);
    assert!(paid.sacrificed_suspected);
    assert_eq!(game.object_view(marked).zone, Zone::Exile);
    assert!(game.observe(player).suspected.is_empty());
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 22);
}

#[test]
fn one_attack_trigger_records_only_the_suspected_members_of_a_declaration() {
    let _rules = fixture_rules::activate(&[(named("Bear Cub"), &ATTACK_OBSERVER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let menacing = put(
        &mut game,
        player,
        "Prowler, Clawed Thief",
        Zone::Battlefield,
    );
    let first_ref = game.object_ref(first);
    let second_ref = game.object_ref(second);
    game.set_suspected(first_ref, true);
    game.set_suspected(second_ref, true);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;

    let mut only_menace = game.clone();
    act(&mut only_menace, Action::Attack { card: menacing });
    act(&mut only_menace, Action::FinishAttackers);
    assert!(only_menace.state.stack.is_empty());

    for card in [first, menacing, second] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    assert_eq!(game.state.stack.len(), 1);
    let event = game.state.stack[0].event_object.as_deref().unwrap();
    assert_eq!(event.object, first_ref);
    assert_eq!(
        event.semantic_objects.as_deref().unwrap().as_slice(),
        &[DamageSourceRef::Object(second_ref)],
    );
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 21);
}

#[test]
fn suspect_transfer_owns_the_optional_exact_entry_cohort() {
    let _rules = fixture_rules::activate(&[
        (named("Bear Cub"), &ENTRY_OBSERVER),
        (named("Opt"), &CREATE_PAIR),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let outsider = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    let mark = game.object_ref(source);
    game.set_suspected(mark, true);
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    assert!(game.state.stack[0].targets.is_empty());
    let entrants = game
        .controlled_token_ids(player)
        .map(|id| game.object_ref(id))
        .collect::<Vec<_>>();
    let [first, second] = entrants.as_slice() else {
        panic!("expected a simultaneous pair");
    };

    let mut changed = game.clone();
    changed.set_suspected(*first, true);
    changed.apply_temporary_copy(
        *second,
        CopyRecipe::new(Card::Forest),
        CopyDeadline::EndOfTurn,
    );
    assert!(!changed.is_creature(second.card));
    pass_pair(&mut changed);
    assert_eq!(
        changed.observe(player).effect_choice.unwrap().candidates,
        [*second],
    );
    act(&mut changed, Action::ChooseEffectObject { object: *second });
    assert_eq!(changed.observe(player).suspected, [*first, *second]);
    let mut no_source = game.clone();
    no_source.set_suspected(mark, false);
    settle(&mut no_source);
    assert!(no_source.observe(player).suspected.is_empty());

    pass_pair(&mut game);
    for viewer in [player, player.other()] {
        let choice = game.observe(viewer).effect_choice.unwrap();
        assert!(choice.optional);
        assert_eq!(choice.candidates, entrants);
    }
    let outsider = game.object_ref(outsider);
    reject(&mut game, Action::ChooseEffectObject { object: outsider });
    let checkpoint = game.snapshot();
    let action = Action::ChooseEffectObject { object: *first };
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, action);
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(indexed.snapshot(), game.snapshot());
    assert_eq!(game.observe(player).suspected, [*first]);

    indexed.restore(&checkpoint).unwrap();
    act(&mut indexed, Action::ChooseOptional { accept: false });
    assert_eq!(indexed.observe(player).suspected, [mark]);
}

#[test]
fn only_new_tokens_are_suspected_after_replacement_and_entry() {
    let _rules = fixture_rules::activate(&[
        (named("Opt"), &CREATE_THEN_SUSPECT),
        (DETECTIVE, &TOKEN_OBSERVER),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    game.create_tokens(player, DETECTIVE, 1, false);
    let existing = game.controlled_token_ids(player).next().unwrap();
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);
    let all_tokens: Vec<_> = game.controlled_token_ids(player).collect();
    assert_eq!(all_tokens.len(), 3);
    assert_eq!(
        game.observe(player).suspected,
        all_tokens
            .into_iter()
            .filter(|&id| id != existing)
            .map(|id| game.object_ref(id))
            .collect::<Vec<_>>(),
    );
    assert_eq!(game.state.players[player.index()].life, 20);
}
