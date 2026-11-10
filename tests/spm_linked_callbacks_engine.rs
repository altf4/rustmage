//! Exact exile links and source-independent combat callbacks.
mod support;

use rustmage as _;
use rustmage_core::CardName;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

const FOES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::ManaValueMinimum {
                filter: &CastFilter::Any,
                minimum: 4,
            },
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayExileTop { skip: 1 },
            Instruction::ExileTopPlayUntilSourceExilesAnother,
        ],
    }],
    ..CardRules::EMPTY
};

#[test]
fn new_actual_exile_replaces_only_that_players_exact_source_permission() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &FOES)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let old = put(&mut game, player, "Forest", Zone::Library);
    let exile = |game: &mut Game, accept| {
        let trigger = put(game, player, "Hill Giant", Zone::Hand);
        cast(game, trigger, None);
        pass_pair(game);
        act(game, Action::ChooseOptional { accept });
        settle(game);
    };
    let permitted = |game: &Game, id| {
        game.observe(player)
            .exile_permissions
            .iter()
            .any(|row| row.object == game.object_ref(id))
    };
    exile(&mut game, true);
    assert!(permitted(&game, old));

    let new = put(&mut game, player, "Island", Zone::Library);
    exile(&mut game, false);
    assert!(permitted(&game, old) && !permitted(&game, new));
    let saved = game.snapshot();
    let mut replaced = game.clone();
    exile(&mut replaced, true);
    assert!(!permitted(&replaced, old) && permitted(&replaced, new));
    assert_eq!(replaced.object_view(old).zone, Zone::Exile);
    assert_eq!(game.snapshot(), saved);

    game.move_object(source, Zone::Exile);
    game.move_object(source, Zone::Battlefield);
    exile(&mut game, true);
    assert!(permitted(&game, old) && permitted(&game, new));
    act(&mut game, Action::PlayLand { card: old });
    assert_eq!(game.object_view(old).zone, Zone::Battlefield);
}

const SPOT: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Creature),
                TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 1,
                    relation: -1,
                    other: false,
                    nonland: true,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                }),
            ]),
            program: &[Instruction::ExileAllTargetsLinkedToSource],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: &[Instruction::BottomDepartedSourceAndReturnLinkedExile],
        },
    ],
    ..CardRules::EMPTY
};

#[test]
fn death_cost_uses_the_exact_graveyard_card_and_returns_both_linked_groups() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &SPOT)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let top = game.top_library_card(player).unwrap();
    let creature = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let grave = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let unrelated = put(&mut game, player, "Savannah Lions", Zone::Exile);
    let source = put(&mut game, player, "Hill Giant", Zone::Hand);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(creature),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(grave),
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Exile);
    assert_eq!(game.object_view(grave).zone, Zone::Exile);

    game.move_object(source, Zone::Graveyard);
    game.prepare_triggers();
    let mut stale = game.clone();
    stale.move_object(source, Zone::Hand);
    stale.move_object(source, Zone::Graveyard);
    settle(&mut stale);
    assert_eq!(stale.object_view(creature).zone, Zone::Exile);
    assert_eq!(stale.object_view(grave).zone, Zone::Exile);

    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Library);
    assert_eq!(game.state.players[player.index()].library[0], source);
    assert_eq!(game.top_library_card(player), Some(top));
    assert_eq!(game.object_view(creature).zone, Zone::Hand);
    assert_eq!(game.object_view(creature).owner, opponent);
    assert_eq!(game.object_view(grave).zone, Zone::Hand);
    assert_eq!(game.object_view(unrelated).zone, Zone::Exile);
}

const HIDE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: 3,
        }]),
        &[Instruction::BlinkExiledTargetsUntilEndStep],
    ),
    ..CardRules::EMPTY
};

#[test]
fn only_actual_exiled_incarnations_return_under_their_owners_control() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &HIDE)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let stolen = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let stale = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let prevented = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.record_control(game.object_ref(stolen), player, None);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|row| row.set(Some(game.object_ref(prevented))));
    announce(
        &mut game,
        spell,
        &[stolen, stale, prevented].map(Target::Card),
    );
    settle(&mut game);
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|row| row.set(None));
    assert_eq!(game.object_view(prevented).zone, Zone::Battlefield);
    let prevented_reference = game.object_ref(prevented);
    assert_eq!(game.object_view(stolen).zone, Zone::Exile);
    assert_eq!(game.object_view(stale).zone, Zone::Exile);

    game.move_object(stale, Zone::Graveyard);
    game.move_object(stale, Zone::Exile);
    game.begin_step(Step::End);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.object_ref(prevented), prevented_reference);
    assert_eq!(game.object_view(stale).zone, Zone::Exile);
    assert_eq!(game.object_view(stolen).zone, Zone::Battlefield);
    assert_eq!(game.object_view(stolen).controller, opponent);
}

const NAMED_CALLBACK: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ChooseCardName { nonland: false },
                Instruction::ScheduleNamedCombatDamageTrigger { ability: 1 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::DelayedTargetEvent,
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
    ],
    ..CardRules::EMPTY
};

fn hit(game: &Game, id: CardId, target: Target, combat: bool) -> DamageEvent {
    DamageEvent {
        preventable: true,
        source: game.damage_source(id),
        target,
        amount: 1,
        combat,
    }
}

#[test]
fn named_combat_callback_survives_saga_sacrifice_and_matches_either_seat() {
    let saga = Card::from_name("Rediscover the Way").unwrap();
    let _rules = fixture_rules::activate(&[(saga, &NAMED_CALLBACK)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let different = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let saga = spawn(&mut game, player, saga, Zone::Hand);
    cast(&mut game, saga, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::CardName(CardName::from_name("Grizzly Bears").unwrap()),
        },
    );
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);
    let packets = [
        hit(&game, first, Target::Player(opponent), true),
        hit(&game, first, Target::Player(opponent), true),
        hit(&game, second, Target::Player(player), true),
        hit(&game, second, Target::Player(player), false),
        hit(&game, different, Target::Player(opponent), true),
    ];
    game.apply_damage_batch(&packets);
    game.advance_replacements(false);
    game.prepare_triggers();
    choose(&mut game, |a| matches!(a, Action::PutTrigger { .. }));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);

    game.begin_step(Step::End);
    game.give_priority();
    pass_pair(&mut game);
    assert!(
        game.state
            .combat_rules
            .as_ref()
            .is_none_or(|rules| rules.turn_callbacks.is_empty())
    );
    game.apply_damage_batch(&[hit(&game, first, Target::Player(opponent), true)]);
    game.advance_replacements(false);
    assert!(game.state.pending_triggers.is_empty());
}

const IMPOSTOR: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Creatures {
                filter: CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: Some(false),
                    artifact: false,
                },
                qualification: DamageQualification::Any,
            },
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateNonlegendaryCopyTokens {
            subject: Subject::Entering,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};

#[test]
fn damage_copy_uses_latest_copiable_values_of_the_dealer_even_after_departure() {
    let card = Card::from_name("Nutrient Block").unwrap();
    let _rules = fixture_rules::activate(&[(card, &IMPOSTOR)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let observer = spawn(&mut game, player, card, Zone::Battlefield);
    let dealer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.add_counter(dealer, CounterKind::PlusOne, 1);
    let exact = game.object_ref(dealer);
    let mut recipe = CopyRecipe::new(Card::HillGiant);
    recipe.overrides = Some(CopyOverrides {
        legendary: Some(true),
        ..CopyOverrides::NONE
    });
    game.set_copy_recipe(exact, recipe);
    let packets = [
        hit(&game, dealer, Target::Player(opponent), true),
        hit(&game, dealer, Target::Player(opponent), false),
    ];
    game.apply_damage_batch(&packets);
    game.advance_replacements(false);
    game.prepare_triggers();
    assert_eq!(game.state.stack.len(), 1);

    recipe.definition = Card::from_name("Savannah Lions").unwrap();
    game.set_copy_recipe(exact, recipe);
    game.move_object(dealer, Zone::Graveyard);
    game.move_object(dealer, Zone::Battlefield);
    game.move_object(observer, Zone::Graveyard);
    settle(&mut game);
    let copy = game
        .observe(player)
        .objects
        .into_iter()
        .find(|row| row.zone == Zone::Battlefield && row.is_token())
        .unwrap();
    assert_eq!(copy.name, "Savannah Lions");
    assert!(!copy.is_legendary());
    assert_eq!(copy.counters.plus_one, 0);
    assert_eq!(game.object_view(dealer).name, "Grizzly Bears");
}
