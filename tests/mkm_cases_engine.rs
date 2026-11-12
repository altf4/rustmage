//! Case designation, trigger timing, and live rewards on exact permanents.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules, object, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const CASE: Card = Card::from_catalogue_id(667); // Case of the Pilfered Proof
const GORGON: Card = Card::from_catalogue_id(665); // Case of the Gorgon's Kiss
const HOTHOUSE: Card = Card::from_catalogue_id(666); // Case of the Locked Hothouse
const AUDITOR: Card = Card::from_catalogue_id(660); // Case File Auditor
const CLUE: Card = Card::from_catalogue_id(5296);
const EXTRA_CLUE: ReplacementSpec = ReplacementSpec::AddToken(CLUE);
const CASE_RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                guard: StepGuard::SourceUnsolved(&StepGuard::Threshold {
                    value: StepValue::Creatures,
                    minimum: Some(2),
                    maximum: None,
                }),
                event: &TriggerCondition::AtStep {
                    step: TriggerStep::End,
                    relation: CastRelation::You,
                    guard: StepGuard::Always,
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::SolveSource],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::SourceSolved,
                event: &TriggerCondition::SpellCast {
                    relation: CastRelation::You,
                    filter: CastFilter::InstantSorcery,
                    ordinal: None,
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::SourceSolved,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateCopyToken {
            subject: Subject::Source,
            amount: 1,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    static_groups: &[
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::ControlledPermanentKeywords {
                kinds: 1,
                other: false,
                keywords: Keywords::FLYING,
            }],
        },
        StaticAbilityGroup {
            line: 3,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Replacement(0, EXTRA_CLUE)],
        },
    ],
    extras: &CardExtras {
        solved_static_groups: &[2, 3],
        replacement_effects: &[EXTRA_CLUE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const GORGON_BASE: BaseStatSetter = BaseStatSetter {
    power: Some(4),
    toughness: Some(4),
    own_turn: false,
    condition: None,
};
const GORGON_KEYWORDS: Keywords = Keywords::DEATHTOUCH.union(Keywords::LIFELINK);
const GORGON_RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                guard: StepGuard::SourceUnsolved(&StepGuard::Threshold {
                    value: StepValue::Creatures,
                    minimum: Some(1),
                    maximum: None,
                }),
                event: &TriggerCondition::AtStep {
                    step: TriggerStep::End,
                    relation: CastRelation::You,
                    guard: StepGuard::Always,
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::SolveSource],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Type,
        fragments: &[
            StaticAbilityFragment::SourceAdditionalCreature {
                subtypes: &["Gorgon"],
                keywords: GORGON_KEYWORDS,
            },
            StaticAbilityFragment::BaseStatSetter(0, GORGON_BASE),
        ],
    }],
    extras: &CardExtras {
        base_stat_setter: Some(GORGON_BASE),
        solved_static_groups: &[2],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const HOTHOUSE_RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::SourceUnsolved(&StepGuard::Threshold {
                value: StepValue::Lands,
                minimum: Some(7),
                maximum: None,
            }),
            event: &TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::SolveSource],
    }],
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AdditionalLandPlays(1)],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::LookAtLibraryTop],
        },
        StaticAbilityGroup {
            line: 3,
            first_layer: AbilityLayer::Rule,
            fragments: &[
                StaticAbilityFragment::PlayLandFromLibraryTop,
                StaticAbilityFragment::CastCreatureFromLibraryTop,
                StaticAbilityFragment::CastFromLibraryTop(CastFilter::Enchantment),
            ],
        },
    ],
    extras: &CardExtras {
        solved_static_groups: &[2, 3],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const AUDITOR_RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CaseSolved(CastRelation::You),
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
    }],
    ..CardRules::EMPTY
};
const BLINK_RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(object(0)),
        &[Instruction::BlinkImmediately {
            counter: None,
            tapped: false,
        }],
    ),
    ..CardRules::EMPTY
};
const COPY_TARGET_RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(object(0)),
        &[Instruction::CreateCopyToken {
            subject: Subject::Targets,
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};
const CONTROL_RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(object(0)),
        &[Instruction::GainControl {
            subject: Subject::Targets,
            controller: PlayerSet::Controller,
        }],
    ),
    ..CardRules::EMPTY
};
const BLANK_RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(object(0)),
        &[Instruction::RemoveAbilities {
            subject: Subject::Targets,
            duration: AnimationDuration::EndOfTurn,
        }],
    ),
    ..CardRules::EMPTY
};

fn end_step(game: &mut Game) {
    game.begin_step(Step::End);
    game.prepare_triggers();
}

fn life(game: &Game, player: Player) -> i128 {
    game.observe(player).players[player.index()].life
}

fn solved(game: &Game, viewer: Player) -> Vec<ObjectRef> {
    game.observe(viewer).solved_cases
}

fn activation(game: &Game, card: CardId) -> Option<Action> {
    game.legal_actions().into_iter().find(|action| {
        matches!(action,
            Action::Activate { card: source, ability: 0, .. }
            | Action::AnnounceActivate { card: source, ability: 0, .. }
            if *source == card)
    })
}

fn clues(game: &Game, player: Player) -> usize {
    game.observe(player)
        .objects
        .iter()
        .filter(|object| {
            object.zone == Zone::Battlefield && object.controller == player && object.card == CLUE
        })
        .count()
}

#[test]
fn solve_rechecks_its_condition_and_incarnation_across_owned_children() {
    let blink = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[
        (CASE, &CASE_RULES),
        (AUDITOR, &AUDITOR_RULES),
        (blink, &BLINK_RULES),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    let case = spawn(&mut game, player, CASE, Zone::Battlefield);
    let original = game.object_ref(case);
    spawn(&mut game, player, AUDITOR, Zone::Battlefield);
    spawn(&mut game, player.other(), AUDITOR, Zone::Battlefield);
    end_step(&mut game);
    assert!(game.state.stack.is_empty());

    let bear = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    let murder = put(&mut game, player, "Murder", Zone::Hand);
    let blink = spawn(&mut game, player, blink, Zone::Hand);
    end_step(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    assert!(activation(&game, case).is_none());
    let checkpoint = game.snapshot();
    let revision = game.revision();
    let pass = game
        .legal_actions()
        .iter()
        .position(|action| *action == Action::Pass)
        .unwrap();
    let mut raw = game.clone();
    let mut indexed = game.clone();
    let mut fewer_creatures = game.clone();
    let mut new_case = game;
    act(&mut raw, Action::Pass);
    indexed.apply_index(player, revision, pass).unwrap();
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(solved(&raw, player), [original]);
    assert_eq!(solved(&raw, player.other()), [original]);
    assert_eq!(life(&raw, player), 23);
    assert_eq!(life(&raw, player.other()), 20);
    assert!(activation(&raw, case).is_some());

    cast(&mut fewer_creatures, murder, Some(Target::Card(bear)));
    settle(&mut fewer_creatures);
    assert_eq!(fewer_creatures.object_view(bear).zone, Zone::Graveyard);
    assert!(solved(&fewer_creatures, player).is_empty());
    assert_eq!(life(&fewer_creatures, player), 20);

    cast(&mut new_case, blink, Some(Target::Card(case)));
    settle(&mut new_case);
    assert_eq!(new_case.object_view(case).zone, Zone::Battlefield);
    assert_ne!(new_case.object_ref(case), original);
    assert!(solved(&new_case, player).is_empty());
    assert!(activation(&new_case, case).is_none());

    indexed.restore(&checkpoint).unwrap();
    assert!(indexed.apply_index(player, revision, pass).is_err());
    assert_eq!(indexed.snapshot(), checkpoint);
    settle(&mut indexed);
    assert_eq!(indexed.snapshot(), raw.snapshot());
    end_step(&mut raw);
    assert!(raw.state.stack.is_empty());
    assert_eq!(life(&raw, player), 23);
}

#[test]
fn solve_event_belongs_to_the_ability_controller_and_rewards_follow_live_control() {
    let take = Card::from_name("Quick Study").unwrap();
    let _rules = fixture_rules::activate(&[
        (CASE, &CASE_RULES),
        (AUDITOR, &AUDITOR_RULES),
        (take, &CONTROL_RULES),
    ]);
    let owner = Player::P0;
    let opponent = owner.other();
    let mut game = scenario(owner);
    let case = spawn(&mut game, owner, CASE, Zone::Battlefield);
    let own_auditor = spawn(&mut game, owner, AUDITOR, Zone::Battlefield);
    let enemy_auditor = spawn(&mut game, opponent, AUDITOR, Zone::Battlefield);
    spawn(&mut game, owner, Card::GrizzlyBears, Zone::Battlefield);
    let take = spawn(&mut game, opponent, take, Zone::Hand);

    end_step(&mut game);
    act(&mut game, Action::Pass);
    cast(&mut game, take, Some(Target::Card(case)));
    settle(&mut game);
    assert_eq!(game.object_view(case).controller, opponent);
    assert_eq!(solved(&game, opponent), [game.object_ref(case)]);
    assert_eq!(life(&game, owner), 23);
    assert_eq!(life(&game, opponent), 20);
    assert!(
        !game
            .object_view(own_auditor)
            .keywords
            .contains(Keywords::FLYING)
    );
    assert!(
        game.object_view(enemy_auditor)
            .keywords
            .contains(Keywords::FLYING)
    );
}

#[test]
fn solved_rewards_gate_triggers_replacements_and_activations_without_becoming_copiable() {
    let blank = Card::from_name("Get Lost").unwrap();
    let _rules = fixture_rules::activate(&[(CASE, &CASE_RULES), (blank, &BLANK_RULES)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let case = spawn(&mut game, player, CASE, Zone::Battlefield);
    let first = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    let first_deduce = put(&mut game, player, "Deduce", Zone::Hand);
    let second_deduce = put(&mut game, player, "Deduce", Zone::Hand);
    let third_deduce = put(&mut game, player, "Deduce", Zone::Hand);
    let blank = spawn(&mut game, player.other(), blank, Zone::Hand);
    assert!(!game.object_view(first).keywords.contains(Keywords::FLYING));
    reject(
        &mut game,
        Action::AnnounceActivate {
            card: case,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    cast(&mut game, first_deduce, None);
    settle(&mut game);
    assert_eq!(life(&game, player), 20);
    assert_eq!(clues(&game, player), 1);

    end_step(&mut game);
    settle(&mut game);
    assert!(game.object_view(first).keywords.contains(Keywords::FLYING));
    cast(&mut game, second_deduce, None);
    let mut suppressed = game.clone();
    act(&mut suppressed, Action::Pass);
    assert_eq!(suppressed.actor(), Some(player.other()));
    cast(&mut suppressed, blank, Some(Target::Card(case)));
    settle(&mut suppressed);
    assert_eq!(life(&suppressed, player), 22, "already queued reward");
    assert_eq!(clues(&suppressed, player), 2, "replacement was suppressed");

    settle(&mut game);
    assert_eq!(life(&game, player), 22);
    assert_eq!(clues(&game, player), 3);

    let activate = activation(&game, case).expect("solved activation");
    let needs_payment = matches!(activate, Action::AnnounceActivate { .. });
    act(&mut game, activate);
    if needs_payment {
        pay(&mut game);
    }
    settle(&mut game);
    let copy = game
        .observe(player)
        .objects
        .iter()
        .find(|object| object.zone == Zone::Battlefield && object.card == CASE && object.id != case)
        .expect("the solved ability created a Case copy")
        .id;
    assert_eq!(solved(&game, player), [game.object_ref(case)]);
    assert!(activation(&game, copy).is_none());
    assert_eq!(clues(&game, player), 4);

    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(player.other()));
    cast(&mut game, blank, Some(Target::Card(case)));
    settle(&mut game);
    assert_eq!(solved(&game, player), [game.object_ref(case)]);
    assert!(!game.object_view(first).keywords.contains(Keywords::FLYING));
    assert!(activation(&game, case).is_none());
    assert_eq!(life(&game, player), 22);
    cast(&mut game, third_deduce, None);
    settle(&mut game);
    assert_eq!(life(&game, player), 22);
    assert_eq!(clues(&game, player), 5);
}

#[test]
fn solved_library_permissions_keep_the_top_private_and_use_normal_play_timing() {
    let _rules = fixture_rules::activate(&[(HOTHOUSE, &HOTHOUSE_RULES)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let case = spawn(&mut game, player, HOTHOUSE, Zone::Battlefield);
    let forests = [(); 5].map(|_| put(&mut game, player, "Forest", Zone::Battlefield));
    let island = put(&mut game, player, "Island", Zone::Hand);
    let mountain = put(&mut game, player, "Mountain", Zone::Hand);
    let spell = put(&mut game, player, "Grizzly Bears", Zone::Library);
    reject(&mut game, Action::AnnounceCast { card: spell });
    let land = put(&mut game, player, "Forest", Zone::Library);
    let play_top = Action::PlayLand { card: land };
    let top_for = |game: &Game, viewer| {
        game.observe(viewer).known_library_top[player.index()]
            .as_ref()
            .map(|top| top.object)
    };

    assert_eq!(top_for(&game, player), None);
    assert_eq!(top_for(&game, opponent), None);
    reject(&mut game, play_top.clone());
    act(&mut game, Action::PlayLand { card: island });
    act(&mut game, Action::PlayLand { card: mountain });
    assert_eq!(game.object_view(mountain).zone, Zone::Battlefield);
    assert_eq!(top_for(&game, player), None);

    end_step(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(top_for(&game, player), None);
    settle(&mut game);
    assert_eq!(solved(&game, player), [game.object_ref(case)]);
    assert_eq!(top_for(&game, player), Some(game.object_ref(land)));
    assert_eq!(top_for(&game, opponent), None);
    reject(&mut game, play_top.clone());

    // This draw leaves the land on top for the next main phase.
    let draw = put(&mut game, player, "Forest", Zone::Library);
    let turn = game.state.turn;
    for _ in 0..32 {
        if game.state.turn > turn && game.state.active == player && game.state.step == Step::Main1 {
            break;
        }
        let action = if game.state.pending == Pending::Attackers {
            Action::FinishAttackers
        } else {
            Action::Pass
        };
        act(&mut game, action);
    }
    assert_eq!(game.state.step, Step::Main1);
    assert_eq!(game.actor(), Some(player));
    assert_eq!(game.object_view(draw).zone, Zone::Hand);
    assert_eq!(top_for(&game, player), Some(game.object_ref(land)));

    act(&mut game, play_top);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert_eq!(top_for(&game, player), Some(game.object_ref(spell)));
    assert_eq!(top_for(&game, opponent), None);
    for card in &forests[..2] {
        act(&mut game, Action::TapMana { card: *card });
    }
    cast(&mut game, spell, None);
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().origin,
        Zone::Library
    );
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
    assert_eq!(top_for(&game, opponent), None);
}

#[test]
fn solved_type_group_uses_one_incarnation_and_earlier_layers_survive_ability_removal() {
    let blank = Card::from_name("Get Lost").unwrap();
    let copy_spell = Card::from_name("Quick Study").unwrap();
    let blink = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[
        (GORGON, &GORGON_RULES),
        (blank, &BLANK_RULES),
        (copy_spell, &COPY_TARGET_RULES),
        (blink, &BLINK_RULES),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let gorgon = spawn(&mut game, player, GORGON, Zone::Battlefield);
    let original = game.object_ref(gorgon);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let blank = spawn(&mut game, player.other(), blank, Zone::Hand);
    let copy_spell = spawn(&mut game, player, copy_spell, Zone::Hand);
    let blink = spawn(&mut game, player, blink, Zone::Hand);

    let unsolved = game.object_view(gorgon);
    assert_eq!(unsolved.kinds, 32);
    assert!(unsolved.has_subtype("Case"));
    assert!(!unsolved.has_subtype("Gorgon"));
    assert!(!unsolved.keywords.intersects(GORGON_KEYWORDS));
    end_step(&mut game);
    assert_eq!(game.state.stack.len(), 1);

    let mut blanked = game.clone();
    act(&mut blanked, Action::Pass);
    cast(&mut blanked, blank, Some(Target::Card(gorgon)));
    settle(&mut blanked);
    let still_creature = blanked.object_view(gorgon);
    assert_eq!(solved(&blanked, player), [original]);
    assert_eq!(still_creature.kinds, 1 | 32);
    assert!(still_creature.has_subtype("Case") && still_creature.has_subtype("Gorgon"));
    assert_eq!(stats(&blanked, gorgon), (4, 4));
    assert!(!still_creature.keywords.intersects(GORGON_KEYWORDS));
    assert_eq!(life(&blanked, player), 20);

    settle(&mut game);
    let solved_view = game.object_view(gorgon);
    assert_eq!(solved_view.reference, original);
    assert_eq!(solved_view.card, GORGON);
    assert_eq!(solved_view.kinds, 1 | 32);
    assert!(solved_view.has_subtype("Case") && solved_view.has_subtype("Gorgon"));
    assert!(solved_view.keywords.contains(GORGON_KEYWORDS));
    assert_eq!(stats(&game, gorgon), (4, 4));
    assert_eq!(life(&game, player), 20);

    game.add_counter(gorgon, CounterKind::PlusOne, 1);
    assert_eq!(stats(&game, gorgon), (5, 5));
    cast(&mut game, copy_spell, Some(Target::Card(gorgon)));
    settle(&mut game);
    let copy = game
        .observe(player)
        .objects
        .into_iter()
        .find(|object| {
            object.zone == Zone::Battlefield && object.card == GORGON && object.id != gorgon
        })
        .expect("copy of the Case");
    assert!(copy.is_token());
    assert_eq!(copy.kinds, 32);
    assert!(copy.has_subtype("Case"));
    assert!(!copy.has_subtype("Gorgon"));
    assert!(!copy.keywords.intersects(GORGON_KEYWORDS));
    assert_eq!(life(&game, player), 21);
    assert_eq!(solved(&game, player), [original]);

    game.apply_temporary_copy(
        original,
        CopyRecipe::new(Card::GrizzlyBears),
        CopyDeadline::EndOfTurn,
    );
    let bear = game.object_view(gorgon);
    assert_eq!(bear.reference, original);
    assert_eq!(bear.card, Card::GrizzlyBears);
    assert!(!bear.has_subtype("Gorgon"));
    assert_eq!(stats(&game, gorgon), (3, 3));
    assert_eq!(solved(&game, player), [original]);
    game.expire_temporary_copies(true);
    let restored = game.object_view(gorgon);
    assert_eq!(restored.reference, original);
    assert!(restored.has_subtype("Case") && restored.has_subtype("Gorgon"));
    assert!(restored.keywords.contains(GORGON_KEYWORDS));
    assert_eq!(stats(&game, gorgon), (5, 5));

    cast(&mut game, blink, Some(Target::Card(gorgon)));
    settle(&mut game);
    let returned = game.object_view(gorgon);
    assert_ne!(returned.reference, original);
    assert_eq!(returned.kinds, 32);
    assert!(returned.has_subtype("Case"));
    assert!(!returned.has_subtype("Gorgon"));
    assert!(!returned.keywords.intersects(GORGON_KEYWORDS));
    assert!(solved(&game, player).is_empty());
    assert_eq!(life(&game, player), 22);
}
