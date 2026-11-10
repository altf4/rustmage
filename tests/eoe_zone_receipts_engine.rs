//! Untargeted linked exile and actual destruction receipts in owned resolutions.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::characteristic_layers::AnimationTypeMode;
use rustmage_core::engine::*;
use support::*;

const CREATURE: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 1,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};

const LINKED_EXILE: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::CreatureDies {
                relation: CastRelation::You,
                other: false,
                nontoken: false,
                subtype: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::ChooseObjects {
                kind: EffectChoiceKind::ExileControlledLinked,
                players: PlayerSet::Opponent,
                filter: CREATURE,
                amount: 1,
                optional_skip: None,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::AnyPlayerControlsNoCreatures,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::SacrificeSource,
                Instruction::ChooseLinkedCreatureReturnWithCounters(2),
            ],
        },
    ],
    ..CardRules::EMPTY
};

const SWEEP: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::DestroyCreaturesByToughness {
                maximum: AmountValue::ChosenX,
                remember: true,
            },
            Instruction::LoseLifeScaled(PlayerSet::Controller, AmountValue::ChosenX),
            Instruction::IfAmountAtLeast {
                value: AmountValue::ChosenX,
                minimum: 6,
                skip: 1,
            },
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ReturnRememberedCreature,
                players: PlayerSet::Controller,
                filter: CREATURE,
                amount: 1,
                optional_skip: None,
            },
        ],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn animate_artifact(game: &mut Game, artifact: CardId) {
    game.change_object_form(
        game.object_ref(artifact),
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((2, 2)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        AnimationTypeMode::AddCreature,
    );
}

fn activate_exile(game: &mut Game, owner: Player, victim: CardId) {
    let murder = put(game, owner, "Murder", Zone::Hand);
    cast(game, murder, Some(Target::Card(victim)));
    resolve_top(game);
    resolve_top(game);
    assert_eq!(game.actor(), Some(owner.other()));
    assert!(
        game.state
            .resolving
            .as_ref()
            .unwrap()
            .item
            .targets
            .is_empty()
    );
}

#[test]
fn opponent_exiles_by_control_and_end_step_returns_only_an_exact_linked_creature() {
    let _fixture =
        fixture_rules::activate(&[(Card::from_name("Impact Tremors").unwrap(), &LINKED_EXILE)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    game.state.players[player.index()].mana = [12; 6];
    let source = put(&mut game, player, "Impact Tremors", Zone::Battlefield);
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    let island = put(&mut game, player, "Island", Zone::Battlefield);
    let stolen = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    game.state.objects[stolen as usize].controller = opponent;
    game.state.objects[stolen as usize].gained_keywords = Keywords::HEXPROOF;
    let animation = put(&mut game, opponent, "Soul-Guide Lantern", Zone::Battlefield);
    animate_artifact(&mut game, animation);
    let retained = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let unrelated = put(&mut game, opponent, "Hill Giant", Zone::Exile);
    let first = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let second = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let third = put(&mut game, player, "Hill Giant", Zone::Battlefield);

    let mut both_have_creatures = game.clone();
    both_have_creatures.state.step = Step::Main2;
    resolve_top(&mut both_have_creatures);
    assert_eq!(both_have_creatures.state.step, Step::End);
    assert!(both_have_creatures.state.stack.is_empty());

    for (victim, exile) in [(first, stolen), (second, animation), (third, retained)] {
        activate_exile(&mut game, player, victim);
        let object = game.object_ref(exile);
        let wrong = game.object_ref(source);
        reject(&mut game, Action::SelectResolutionExile { object: wrong });
        reject(&mut game, Action::DeclineEffectPayment);
        act(&mut game, Action::SelectResolutionExile { object });
        assert_eq!(game.object_view(exile).zone, Zone::Exile);
    }

    game.move_object(stolen, Zone::Hand);
    game.move_object(stolen, Zone::Exile);
    game.state.step = Step::Main2;
    resolve_top(&mut game);
    assert_eq!(game.state.step, Step::End);
    assert_eq!(game.state.stack.len(), 1);

    let mut invalidated = game.clone();
    let flash = put(&mut invalidated, player, "Spectral Sailor", Zone::Hand);
    act(&mut invalidated, Action::TapMana { card: island });
    cast(&mut invalidated, flash, None);
    settle(&mut invalidated);
    assert_eq!(invalidated.object_view(source).zone, Zone::Battlefield);
    assert_eq!(invalidated.object_view(retained).zone, Zone::Exile);

    let mut removed = game.clone();
    removed.move_object(source, Zone::Hand);
    resolve_top(&mut removed);
    let object = removed.object_ref(retained);
    act(&mut removed, Action::ChooseLinkedExile { object });
    settle(&mut removed);
    assert_eq!(removed.object_view(source).zone, Zone::Hand);
    assert_eq!(removed.object_view(retained).controller, player);

    resolve_top(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    assert_eq!(game.actor(), Some(player));
    for wrong in [stolen, unrelated, animation] {
        let object = game.object_ref(wrong);
        reject(&mut game, Action::ChooseLinkedExile { object });
    }
    reject(&mut game, Action::DeclineEffectPayment);
    let object = game.object_ref(retained);
    act(&mut game, Action::ChooseLinkedExile { object });
    settle(&mut game);
    let entrant = game.object_view(retained);
    assert_eq!(entrant.zone, Zone::Battlefield);
    assert_eq!(entrant.owner, opponent);
    assert_eq!(entrant.controller, player);
    assert_eq!(game.counter_count(retained, CounterKind::PlusOne), 4);
}

fn cast_with_x(game: &mut Game, spell: CardId, value: u32) {
    act(game, Action::AnnounceCast { card: spell });
    act(game, Action::ChooseX { value });
    pay(game);
    resolve_top(game);
}

#[test]
fn toughness_sweep_loses_life_then_requires_an_actual_creature_card_receipt() {
    let _fixture = fixture_rules::activate(&[(Card::from_name("Exsanguinate").unwrap(), &SWEEP)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Exsanguinate", Zone::Hand);
    let boundary = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    game.add_counter(boundary, CounterKind::PlusOne, 3);
    let smaller = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let shielded = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    game.add_counter(shielded, CounterKind::Shield, 1);
    let large = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
    let animated = put(&mut game, opponent, "Soul-Guide Lantern", Zone::Battlefield);
    animate_artifact(&mut game, animated);
    let preexisting = put(&mut game, opponent, "Hill Giant", Zone::Graveyard);
    let token = game.state.objects.len() as CardId;
    game.create_tokens(opponent, Card::from_catalogue_id(5241), 1, false);
    let old_boundary = game.object_ref(boundary);

    let mut small_x = game.clone();
    cast_with_x(&mut small_x, spell, 5);
    settle(&mut small_x);
    assert_eq!(small_x.object_view(boundary).zone, Zone::Battlefield);
    assert_eq!(small_x.object_view(smaller).zone, Zone::Graveyard);
    assert_eq!(small_x.observe(player).players[player.index()].life, 15);

    let mut replaced = game.clone();
    put(&mut replaced, opponent, "Rest in Peace", Zone::Battlefield);
    cast_with_x(&mut replaced, spell, 6);
    settle(&mut replaced);
    assert_eq!(replaced.object_view(boundary).zone, Zone::Exile);
    assert_eq!(replaced.observe(player).players[player.index()].life, 14);

    cast_with_x(&mut game, spell, 6);
    assert_eq!(game.observe(player).players[player.index()].life, 14);
    assert_eq!(game.object_view(boundary).zone, Zone::Graveyard);
    assert_eq!(game.object_view(smaller).zone, Zone::Graveyard);
    assert_eq!(game.object_view(animated).zone, Zone::Graveyard);
    assert_eq!(game.object_view(large).zone, Zone::Battlefield);
    assert_eq!(game.object_view(shielded).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(shielded, CounterKind::Shield), 0);
    assert_eq!(game.actor(), Some(player));
    for id in [animated, preexisting, token, shielded, large] {
        let object = game.object_ref(id);
        reject(&mut game, Action::SelectGraveyardReturn { object });
    }
    reject(
        &mut game,
        Action::SelectGraveyardReturn {
            object: old_boundary,
        },
    );
    reject(&mut game, Action::FinishGraveyardReturn);
    let object = game.object_ref(boundary);
    let action = Action::SelectGraveyardReturn { object };
    let saved = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut game, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
    let returned = game.object_view(boundary);
    assert_eq!(returned.zone, Zone::Battlefield);
    assert_eq!(returned.controller, player);
    assert_eq!(returned.owner, opponent);

    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    let object = indexed.object_ref(smaller);
    act(&mut indexed, Action::SelectGraveyardReturn { object });
    assert_eq!(indexed.object_view(boundary).zone, Zone::Graveyard);
    assert_eq!(indexed.object_view(smaller).zone, Zone::Battlefield);
}
