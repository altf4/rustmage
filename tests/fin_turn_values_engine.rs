//! Exact turn facts and resolution values that read real objects and events.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const fn end_step(guard: StepGuard, program: &'static [Instruction]) -> TriggeredAbility {
    TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard,
        },
        targets: TargetPlan::NONE,
        program,
    }
}

const RETURN_OLDER_SOURCE: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[end_step(
        StepGuard::SourceEnteredThisTurn(false),
        &[Instruction::ReturnSourceToHand(TargetZone::Battlefield)],
    )],
    ..CardRules::EMPTY
};

const OPPONENT_DEATH: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[end_step(
        StepGuard::OpponentControlledCreatureDied,
        &[Instruction::GainLife(PlayerSet::Controller, 1)],
    )],
    ..CardRules::EMPTY
};

const LIVE_COUNTS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::DrawScaled(
                PlayerSet::Controller,
                AmountValue::Devotion(ChoiceColor::Red),
            ),
            Instruction::GainLifeScaled(
                PlayerSet::Controller,
                AmountValue::NonbasicLandsYouControl,
            ),
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

const RETURN_AND_LOSE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[
            Instruction::ReturnToHand,
            Instruction::LoseLifeScaled(PlayerSet::Opponent, AmountValue::TargetCreaturePower),
        ],
    ),
    ..CardRules::EMPTY
};

const DISCARD_TYPES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::DiscardThenDrawForTypes {
            players: PlayerSet::Each,
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};

const MAY_DRAW: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::MayDraw { skip: 2 },
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::Discard(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn reach_end(game: &mut Game) {
    game.state.step = Step::Main2;
    pass_pair(game);
    assert_eq!(game.state.step, Step::End);
}

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

#[test]
fn entry_turn_is_exact_and_uses_actual_turn_history() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &RETURN_OLDER_SOURCE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, Card::GrizzlyBears, Zone::Hand);
    game.move_object(source, Zone::Battlefield);
    reach_end(&mut game);
    assert!(game.state.stack.is_empty());
    pass_pair(&mut game);
    assert_eq!(game.state.active, player.other());
    reach_end(&mut game);
    assert!(game.state.stack.is_empty());
    pass_pair(&mut game);
    assert_eq!(game.state.active, player);
    reach_end(&mut game);
    assert_eq!(game.state.stack.len(), 1);

    let mut blinked = game.clone();
    blinked.move_object(source, Zone::Exile);
    blinked.move_object(source, Zone::Battlefield);
    settle(&mut blinked);
    assert_eq!(blinked.object_view(source).zone, Zone::Battlefield);

    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Hand);
}

#[test]
fn the_death_condition_uses_predeparture_control_and_actual_destination() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &OPPONENT_DEATH)]);
    let player = Player::P0;
    let mut game = scenario(player);
    spawn(&mut game, player, Card::HillGiant, Zone::Battlefield);
    let creature = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    let mut own_death = game.clone();
    own_death.move_object(creature, Zone::Graveyard);
    reach_end(&mut own_death);
    assert!(own_death.state.stack.is_empty());

    game.state.objects[creature as usize].controller = player.other();
    let mut exile = game.clone();
    exile.move_object(creature, Zone::Exile);
    reach_end(&mut exile);
    assert!(exile.state.stack.is_empty());

    game.move_object(creature, Zone::Graveyard);
    assert_eq!(game.object_view(creature).owner, player);
    reach_end(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 21);
}

#[test]
fn devotion_and_nonbasic_lands_sample_current_copiable_values_and_control() {
    let opt = card("Opt");
    let titan = card("Abomination, Terrifying Titan");
    let bifur = card("Bifur, Melodic Rider");
    const BODY: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[(opt, &LIVE_COUNTS), (titan, &BODY), (bifur, &BODY)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = spawn(&mut game, player, opt, Zone::Hand);
    let piker = card("Goblin Piker");
    spawn(&mut game, player, piker, Zone::Battlefield);
    spawn(&mut game, player.other(), piker, Zone::Battlefield);
    spawn(&mut game, player, titan, Zone::Battlefield);
    spawn(&mut game, player, bifur, Zone::Battlefield);
    let embalmed = spawn(&mut game, player, piker, Zone::Battlefield);
    let object = game.object_ref(embalmed);
    game.set_copy_recipe(
        object,
        CopyRecipe::new(piker).with_exceptions(CopyExceptions {
            embalm: true,
            ..CopyExceptions::NONE
        }),
    );
    let ground = card("Stomping Ground");
    spawn(&mut game, player, ground, Zone::Battlefield);
    spawn(&mut game, player.other(), ground, Zone::Battlefield);
    spawn(&mut game, player, Card::Forest, Zone::Battlefield);

    cast(&mut game, spell, None);
    let mut original = game.clone();
    settle(&mut original);
    assert_eq!(original.hand_count(player), 4);
    assert_eq!(original.state.players[player.index()].life, 21);
    spawn(&mut game, player, ground, Zone::Battlefield);
    settle(&mut game);
    assert_eq!(game.hand_count(player), 4);
    assert_eq!(game.state.players[player.index()].life, 22);
}

#[test]
fn life_loss_reads_a_graveyard_cards_power_before_it_moves_to_hand() {
    let opt = card("Opt");
    let _rules = fixture_rules::activate(&[(opt, &RETURN_AND_LOSE), (Card::GrizzlyBears, &CDA)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = spawn(&mut game, player, opt, Zone::Hand);
    let target = spawn(&mut game, player, Card::GrizzlyBears, Zone::Graveyard);
    spawn(&mut game, player, Card::Forest, Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(target)));
    let mut illegal = game.clone();
    illegal.move_object(target, Zone::Exile);
    settle(&mut illegal);
    assert_eq!(illegal.state.players[player.other().index()].life, 20);
    settle(&mut game);
    assert_eq!(game.object_view(target).zone, Zone::Hand);
    assert_eq!(game.stats(target).power, 2);
    assert_eq!(game.state.players[player.other().index()].life, 19);
}

#[test]
fn simultaneous_discards_count_the_union_of_actual_card_types() {
    let opt = card("Opt");
    let _rules = fixture_rules::activate(&[(opt, &DISCARD_TYPES)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = spawn(&mut game, player, opt, Zone::Hand);
    let first = spawn(
        &mut game,
        player,
        card("Clockwork Percussionist"),
        Zone::Hand,
    );
    let second = spawn(&mut game, opponent, card("Ashling's Command"), Zone::Hand);
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(player));
    act(&mut game, Action::Discard { card: first });
    assert_eq!(game.actor(), Some(opponent));
    assert_eq!(game.object_view(first).zone, Zone::Hand);
    assert!(
        !game
            .observe(opponent)
            .actions
            .contains(&Action::Discard { card: first })
    );
    let saved = game.snapshot();
    let action = Action::Discard { card: second };
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(opponent, game.revision(), index)
        .unwrap();
    act(&mut game, action.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.hand_count(player), 4);
    assert_eq!(game.object_view(first).zone, Zone::Graveyard);
    assert_eq!(game.object_view(second).zone, Zone::Graveyard);

    struct ClearPrevention;
    impl Drop for ClearPrevention {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
        }
    }
    let _clear = ClearPrevention;
    game.restore(&saved).unwrap();
    let exact = game.object_ref(first);
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(Some(exact)));
    act(&mut game, action);
    assert_eq!(game.object_view(first).zone, Zone::Hand);
    assert_eq!(game.hand_count(player), 3);
}

#[test]
fn prohibited_optional_draw_skips_its_discard_followup() {
    let opt = card("Opt");
    let _rules = fixture_rules::activate(&[(opt, &MAY_DRAW)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = spawn(&mut game, player, opt, Zone::Hand);
    let hand = spawn(&mut game, player, Card::Forest, Zone::Hand);
    cast(&mut game, spell, None);

    let mut prohibited = game.clone();
    spawn(
        &mut prohibited,
        player.other(),
        card("Mornsong Aria"),
        Zone::Battlefield,
    );
    pass_pair(&mut prohibited);
    assert_eq!(prohibited.state.pending, Pending::Priority);
    assert_eq!(prohibited.hand_count(player), 1);
    assert_eq!(prohibited.object_view(hand).zone, Zone::Hand);

    pass_pair(&mut game);
    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.hand_count(player), 2);
    act(&mut game, Action::Discard { card: hand });
    assert_eq!(game.hand_count(player), 1);
    assert_eq!(game.object_view(hand).zone, Zone::Graveyard);
}

const REMEMBER_LANDS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::RememberCount(AmountValue::NonbasicLandsYouControl),
            Instruction::MayPayCosts {
                costs: &[ResolutionCost {
                    objects: Some(ResolutionObjectCost {
                        cost: AdditionalCost {
                            kind: EffectChoiceKind::Sacrifice,
                            filter: SacrificeFilter {
                                keywords: Keywords::NONE,
                                source_only: false,
                                kinds: 2,
                                nonland: false,
                                other: false,
                                tokens: None,
                                artifact_enchantment_or_token: false,
                                subtype: None,
                            },
                        },
                        count: 1,
                    }),
                    ..ResolutionCost::FREE
                }],
                skip: 0,
            },
            Instruction::GainLifeScaled(PlayerSet::Controller, AmountValue::RememberedCount),
        ],
    ),
    ..CardRules::EMPTY
};

#[test]
fn a_saved_count_survives_a_payment_that_changes_the_live_quantity() {
    let opt = card("Opt");
    let _rules = fixture_rules::activate(&[(opt, &REMEMBER_LANDS)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = spawn(&mut game, player, opt, Zone::Hand);
    let ground = card("Stomping Ground");
    let first = spawn(&mut game, player, ground, Zone::Battlefield);
    let second = spawn(&mut game, player, ground, Zone::Battlefield);
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    let object = game.object_ref(second);
    act(&mut game, Action::ChooseAdditionalCost { object });
    let saved = game.snapshot();
    let action = Action::PayResolutionMana { payment: [0; 6] };
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, game.revision(), index).unwrap();
    act(&mut game, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
    assert_eq!(game.object_view(second).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 22);
    game.restore(&saved).unwrap();
    act(&mut game, Action::DeclineResolutionMana);
    assert_eq!(game.object_view(second).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].life, 22);
}
