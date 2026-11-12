//! Earlier-layer type and color effects and their actual and projected mana.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const CONTROLLED_COLORS_AND_TYPES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Color,
            fragments: &[StaticAbilityFragment::ControlledNonlandAllColors],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Type,
            fragments: &[StaticAbilityFragment::ControlledLandsAllBasicTypes],
        },
    ],
    ..CardRules::EMPTY
};

const PERMANENT: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 0,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
}));

const ADD_LAND_TYPE: CardRules = CardRules {
    effect: Effect::Spell(
        PERMANENT,
        &[Instruction::AddCharacteristics {
            subject: Subject::Targets,
            addition: CharacteristicAddition {
                card_types: 2,
                creature_types: &[],
                colors: 0,
                duration: AnimationDuration::EndOfTurn,
            },
        }],
    ),
    ..CardRules::EMPTY
};

const SET_BLUE: CardRules = CardRules {
    effect: Effect::Spell(
        PERMANENT,
        &[Instruction::SetColors {
            subject: Subject::Targets,
            colors: ColorValue::Fixed(2),
            duration: AnimationDuration::EndOfTurn,
        }],
    ),
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap_or_else(|| panic!("unknown fixture {name}"))
}

fn has_all_basic_types(game: &Game, land: CardId) -> bool {
    let view = game.object_view(land);
    ["Plains", "Island", "Swamp", "Mountain", "Forest"]
        .into_iter()
        .all(|kind| view.has_subtype(kind))
}

#[test]
fn live_colors_use_timestamp_order_and_basic_types_follow_later_land_types() {
    let _rules = fixture_rules::activate(&[
        (named("Anthem of Champions"), &CONTROLLED_COLORS_AND_TYPES),
        (named("Opt"), &ADD_LAND_TYPE),
        (named("Quick Study"), &SET_BLUE),
    ]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let source = put(&mut game, player, "Anthem of Champions", Zone::Battlefield);
        let cave = put(&mut game, player, "Hidden Courtyard", Zone::Battlefield);
        let artifact = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        let enemy_land = put(&mut game, opponent, "Forest", Zone::Battlefield);
        let out_of_play = put(&mut game, player, "Grizzly Bears", Zone::Hand);
        assert!(has_all_basic_types(&game, cave));
        assert!(game.object_view(cave).has_subtype("Cave"));
        assert_eq!(game.object_view(cave).colors, 0);
        assert_eq!(game.object_view(source).colors, 31);
        assert_eq!(game.object_view(artifact).colors, 31);
        assert_eq!(game.object_view(enemy).colors, 16);
        assert_eq!(game.object_view(out_of_play).colors, 16);
        assert!(!has_all_basic_types(&game, enemy_land));

        let paint = put(&mut game, player, "Quick Study", Zone::Hand);
        cast(&mut game, paint, Some(Target::Card(artifact)));
        settle(&mut game);
        assert_eq!(game.object_view(artifact).colors, 2);
        let newer = put(&mut game, player, "Anthem of Champions", Zone::Battlefield);
        assert_eq!(game.object_view(artifact).colors, 31);

        let change = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, change, Some(Target::Card(creature)));
        settle(&mut game);
        assert_eq!(game.object_view(creature).kinds & 3, 3);
        assert!(game.object_view(creature).has_subtype("Bear"));
        assert!(has_all_basic_types(&game, creature));
        assert_eq!(game.object_view(creature).colors, 16);

        game.move_object(newer, Zone::Graveyard);
        game.record_control(game.object_ref(source), opponent, None);
        assert!(!has_all_basic_types(&game, cave));
        assert!(!has_all_basic_types(&game, creature));
        assert!(has_all_basic_types(&game, enemy_land));
        assert_eq!(game.object_view(enemy).colors, 31);
        assert_eq!(game.object_view(artifact).colors, 2);
        game.move_object(source, Zone::Graveyard);
        assert!(!has_all_basic_types(&game, enemy_land));
        assert_eq!(game.object_view(enemy).colors, 16);
    }
}

#[test]
fn actual_and_projected_land_mana_keep_the_original_ability_and_each_added_color() {
    let _rules =
        fixture_rules::activate(&[(named("Anthem of Champions"), &CONTROLLED_COLORS_AND_TYPES)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        game.state.players[player.index()].mana = [0; 6];
        let source = put(&mut game, player, "Anthem of Champions", Zone::Battlefield);
        let first = put(&mut game, player, "Forest", Zone::Battlefield);
        let second = put(&mut game, player, "Hidden Courtyard", Zone::Battlefield);
        let spell = put(&mut game, player, "Lightning Helix", Zone::Hand);
        let announce = Action::AnnounceCast { card: spell };
        let mut short = game.clone();
        short.state.objects[second as usize].tapped = true;
        reject(&mut short, announce.clone());
        let mut removed = game.clone();
        removed.move_object(source, Zone::Graveyard);
        reject(&mut removed, announce.clone());

        let mut original = game.clone();
        let wastes = put(&mut original, player, "Wastes", Zone::Battlefield);
        choose(
            &mut original,
            |action| matches!(action, Action::ActivateMana { card, production: [0, 0, 0, 0, 0, 1], .. } if *card == wastes),
        );
        assert_eq!(
            original.state.players[player.index()].mana,
            [0, 0, 0, 0, 0, 1]
        );

        act(&mut game, announce);
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(player.other()),
            },
        );
        let red = Action::TapGrantedMana {
            card: first,
            grantor: game.object_ref(first),
            production: [0, 0, 0, 1, 0, 0],
        };
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == red)
            .unwrap();
        let revision = game.revision();
        let saved = game.snapshot();
        let mut indexed = game.clone();
        indexed.apply_index(player, revision, index).unwrap();
        act(&mut game, red.clone());
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        act(&mut indexed, red);
        assert_eq!(game.snapshot(), indexed.snapshot());
        act(&mut game, Action::TapMana { card: second });
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.state.players[player.other().index()].life, 17);
        assert_eq!(game.state.players[player.index()].life, 23);
    }
}

const CREATURE_TOKEN_MANA: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledTokenTapMana {
            kinds: 1,
            production: ManaProduction::Choice {
                mask: 31,
                amount: 1,
            },
        }],
    }],
    ..CardRules::EMPTY
};

const BLANK_PERMANENT: CardRules = CardRules {
    effect: Effect::Spell(
        PERMANENT,
        &[Instruction::RemoveAbilities {
            subject: Subject::Targets,
            duration: AnimationDuration::EndOfTurn,
        }],
    ),
    ..CardRules::EMPTY
};

const ANIMATE_ARTIFACT: CardRules = CardRules {
    effect: Effect::Spell(
        PERMANENT,
        &[Instruction::AnimateTargets(AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((2, 2)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        })],
    ),
    ..CardRules::EMPTY
};

fn token_mana(game: &Game, provider: CardId, card: CardId, color: usize) -> Action {
    let mut production = [0; 6];
    production[color] = 1;
    Action::TapGrantedMana {
        card,
        grantor: game.object_ref(provider),
        production,
    }
}

#[test]
fn only_live_controlled_creature_tokens_gain_the_tap_mana_ability() {
    let _rules = fixture_rules::activate(&[
        (named("Anthem of Champions"), &CREATURE_TOKEN_MANA),
        (named("Quick Study"), &BLANK_PERMANENT),
        (named("Opt"), &ANIMATE_ARTIFACT),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Anthem of Champions", Zone::Battlefield);
        let bearer = spawn(&mut game, player, Card::DrakeToken, Zone::Battlefield);
        let sick = spawn(&mut game, player, Card::DrakeToken, Zone::Battlefield);
        game.state.objects[sick as usize].summoning_sick = true;
        let enemy = spawn(
            &mut game,
            player.other(),
            Card::DrakeToken,
            Zone::Battlefield,
        );
        let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let clue = spawn(
            &mut game,
            player,
            named("@token/a5a792437637731d892720f8"),
            Zone::Battlefield,
        );
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        let helix = put(&mut game, player, "Lightning Helix", Zone::Hand);
        game.state.players[player.index()].mana = [0; 6];

        for color in 0..5 {
            assert!(
                game.legal_actions()
                    .contains(&token_mana(&game, source, bearer, color))
            );
        }
        for card in [sick, enemy, creature, clue, source] {
            let invalid = token_mana(&game, source, card, 3);
            reject(&mut game, invalid);
        }
        let colorless = token_mana(&game, source, bearer, 5);
        reject(&mut game, colorless);
        let announce = Action::AnnounceCast { card: shock };
        let two_colors = Action::AnnounceCast { card: helix };
        reject(&mut game, two_colors.clone());

        let mut animated = game.clone();
        animated.state.players[player.index()].mana = [8; 6];
        let animation = put(&mut animated, player, "Opt", Zone::Hand);
        cast(&mut animated, animation, Some(Target::Card(clue)));
        settle(&mut animated);
        assert!(animated.object_view(clue).is_creature());
        assert!(animated.object_view(clue).is_token());
        animated.state.players[player.index()].mana = [0; 6];
        assert!(
            animated
                .legal_actions()
                .contains(&token_mana(&animated, source, clue, 0))
        );
        act(&mut animated, two_colors);
        act(
            &mut animated,
            Action::ChooseTarget {
                target: Target::Player(player.other()),
            },
        );
        let white = token_mana(&animated, source, clue, 0);
        let red = token_mana(&animated, source, bearer, 3);
        act(&mut animated, white);
        act(&mut animated, red);
        pay(&mut animated);
        settle(&mut animated);
        assert_eq!(animated.state.players[player.other().index()].life, 17);

        for blank in [source, bearer] {
            let mut suppressed = game.clone();
            suppressed.state.players[player.index()].mana = [8; 6];
            let spell = put(&mut suppressed, player, "Quick Study", Zone::Hand);
            cast(&mut suppressed, spell, Some(Target::Card(blank)));
            settle(&mut suppressed);
            suppressed.state.players[player.index()].mana = [0; 6];
            reject(&mut suppressed, announce.clone());
            let lost_grant = token_mana(&suppressed, source, bearer, 3);
            reject(&mut suppressed, lost_grant);
        }
        let mut departed = game.clone();
        departed.move_object(source, Zone::Graveyard);
        reject(&mut departed, announce.clone());

        act(&mut game, announce);
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(player.other()),
            },
        );
        let action = token_mana(&game, source, bearer, 3);
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let revision = game.revision();
        let saved = game.snapshot();
        let mut indexed = game.clone();
        indexed.apply_index(player, revision, index).unwrap();
        act(&mut game, action.clone());
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        act(&mut indexed, action);
        assert_eq!(game.snapshot(), indexed.snapshot());
        assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 1, 0, 0]);
        assert!(game.object_view(bearer).tapped);
        assert!(!game.object_view(sick).tapped);
        assert!(!game.object_view(source).tapped);
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.state.players[player.other().index()].life, 18);
    }
}

const GRAVEYARD_TOKEN_MANA: CardRules = CardRules {
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CardsLeaveGraveyardKinds(1),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::CreateTokens {
                token: Card::DrakeToken,
                amount: 1,
                tapped: false,
            },
            Instruction::ChangeControlledPermanentGroupCounters {
                filters: &[ControlledPermanentFilter {
                    subtype: Some("Drake"),
                    ..ControlledPermanentFilter::ANY
                }],
                effect: PermanentGroupCounters::Add {
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
            },
        ],
    }],
    ..CREATURE_TOKEN_MANA
};

const PAY_FOUR_EVIDENCE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CollectEvidence {
            threshold: 4,
            skip: 0,
        }],
    ),
    ..CardRules::EMPTY
};

#[test]
fn one_creature_card_departure_batch_creates_one_token_before_group_counters() {
    let _rules = fixture_rules::activate(&[
        (named("Anthem of Champions"), &GRAVEYARD_TOKEN_MANA),
        (named("Opt"), &PAY_FOUR_EVIDENCE),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Anthem of Champions", Zone::Battlefield);
    let previous = spawn(&mut game, player, Card::DrakeToken, Zone::Battlefield);
    let opposing = spawn(
        &mut game,
        player.other(),
        Card::DrakeToken,
        Zone::Battlefield,
    );
    let first = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let second = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let payment = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, payment, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    for card in [first, second] {
        let object = game.object_ref(card);
        act(&mut game, Action::ChooseEvidence { object });
    }
    act(&mut game, Action::FinishEvidence);
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(
        game.state.stack[0].source.unwrap().object,
        game.object_ref(source)
    );
    let created = game.state.objects.len() as CardId;
    settle(&mut game);
    assert_eq!(game.state.objects.len(), created as usize + 1);
    assert_eq!(game.object_view(created).card, Card::DrakeToken);
    assert_eq!(game.counter_count(previous, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(created, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(opposing, CounterKind::PlusOne), 0);
    let new_tap = token_mana(&game, source, created, 3);
    reject(&mut game, new_tap);
}
