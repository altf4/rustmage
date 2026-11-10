//! Captured cast copies and continuous rules with live subtype membership.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const COPY_FROM_OUTSIDE: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayFromOutsideOwnHand(&TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Any,
            ordinal: None,
        }),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayOncePerTurn { skip: 2 },
            Instruction::RecordOncePerTurn,
            Instruction::CopyCapturedSpellModified {
                retarget: true,
                modification: CapturedSpellModification::PermanentHaste,
            },
        ],
    }],
    ..CardRules::EMPTY
};

const COPY_MATCHING_POWER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::CreatureManaValueEqualsSourcePower,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyCapturedSpellModified {
            retarget: false,
            modification: CapturedSpellModification::Nonlegendary,
        }],
    }],
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn declining_copy_preserves_the_use_and_only_the_permanent_copy_gets_haste() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &COPY_FROM_OUTSIDE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);

    for turn_use in 0..3 {
        let bear = put(&mut game, player, "Grizzly Bears", Zone::Exile);
        game.grant_exile_play_permission(player, game.object_ref(bear), false);
        act(&mut game, Action::AnnounceExile { card: bear });
        pay(&mut game);
        assert_eq!(game.state.stack.len(), if turn_use < 2 { 2 } else { 1 });
        if turn_use < 2 {
            pass_pair(&mut game);
            act(
                &mut game,
                Action::ChooseOptional {
                    accept: turn_use == 1,
                },
            );
        }
        settle(&mut game);
        assert_eq!(game.object_view(bear).zone, Zone::Battlefield);
        assert!(!game.object_view(bear).keywords.contains(Keywords::HASTE));
    }
    let copies: Vec<_> = game
        .state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, object)| object.zone == Zone::Battlefield && object.is_copy_token())
        .map(|(index, _)| game.object_view(index as CardId))
        .collect();
    assert_eq!(copies.len(), 1);
    assert!(copies[0].keywords.contains(Keywords::HASTE));
    assert!(
        !copies[0]
            .copy_recipe
            .unwrap()
            .intrinsic_keywords()
            .contains(Keywords::HASTE)
    );
}

#[test]
fn cast_time_power_qualifies_and_the_copy_keeps_its_nonlegendary_exception_after_entry() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &COPY_MATCHING_POWER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let observer = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let ordinary = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, ordinary, None);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    let legend = put(&mut game, player, "Alesha, Who Laughs at Fate", Zone::Hand);
    cast(&mut game, legend, None);
    assert_eq!(game.state.stack.len(), 2);
    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    cast(&mut game, bounce, Some(Target::Card(observer)));
    settle(&mut game);
    assert_eq!(game.object_view(observer).zone, Zone::Hand);
    assert_eq!(game.object_view(legend).zone, Zone::Battlefield);
    let copy = game
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, object)| object.zone == Zone::Battlefield && object.is_copy_token())
        .map(|(index, _)| game.object_view(index as CardId))
        .unwrap();
    assert_eq!(copy.name, "Alesha, Who Laughs at Fate");
    assert_eq!(
        copy.copy_recipe.unwrap().overrides.unwrap().legendary,
        Some(false)
    );
}

const SCOPED_RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::ControlledSubtypeIgnoresLegendRule("Spider"),
            StaticAbilityFragment::ControlledSubtypeCannotBeBlockedByKeyword {
                subtype: "Bear",
                blocker_keyword: Keywords::DEFENDER,
            },
        ],
    }],
    ..CardRules::EMPTY
};

#[test]
fn legend_choices_include_only_nonexempt_permanents_until_the_rule_source_leaves() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Nutrient Block").unwrap(), &SCOPED_RULES)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let provider = put(&mut game, player, "Nutrient Block", Zone::Battlefield);
    let spiders = [0, 1].map(|_| {
        put(
            &mut game,
            player,
            "Spider-Woman, Stunning Savior",
            Zone::Battlefield,
        )
    });
    let ordinary = [0, 1].map(|_| {
        put(
            &mut game,
            player,
            "Alesha, Who Laughs at Fate",
            Zone::Battlefield,
        )
    });
    game.state_based_actions();
    let keep = |game: &Game, card| Action::KeepLegend {
        object: game.object_ref(card),
    };
    for card in spiders {
        let choice = keep(&game, card);
        reject(&mut game, choice);
    }
    let ordinary_choice = keep(&game, ordinary[0]);
    act(&mut game, ordinary_choice);
    assert_eq!(game.object_view(ordinary[1]).zone, Zone::Graveyard);
    for card in spiders {
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
    }

    let removal = put(&mut game, player, "Disenchant", Zone::Hand);
    cast(&mut game, removal, Some(Target::Card(provider)));
    pass_pair(&mut game);
    let spider_choice = keep(&game, spiders[0]);
    act(&mut game, spider_choice);
    assert_eq!(game.object_view(spiders[1]).zone, Zone::Graveyard);
}

const DEFENDER: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    keywords: Keywords::DEFENDER,
    ..CardRules::EMPTY
};

#[test]
fn scoped_block_restriction_rejects_only_the_matching_blocker_and_attacker_pair() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Nutrient Block").unwrap(), &SCOPED_RULES),
        (Card::HillGiant, &DEFENDER),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Nutrient Block", Zone::Battlefield);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let other = put(&mut game, player, "Savannah Lions", Zone::Battlefield);
    let defender = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let ordinary = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    game.begin_step(Step::BeginCombat);
    pass_pair(&mut game);
    act(&mut game, Action::Attack { card: bear });
    act(&mut game, Action::Attack { card: other });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::Block {
            blocker: defender,
            attacker: bear,
        },
    );
    act(
        &mut game,
        Action::Block {
            blocker: defender,
            attacker: other,
        },
    );
    act(
        &mut game,
        Action::Block {
            blocker: ordinary,
            attacker: bear,
        },
    );
    act(&mut game, Action::FinishBlockers);
}
