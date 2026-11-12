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
