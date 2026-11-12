//! Land type changes in the battlefield, prospective entry, and cleanup.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const ARTIFACT_LANDS: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Type,
        fragments: &[StaticAbilityFragment::ControlledNontokenArtifactsAreLands],
    }],
    ..CardRules::EMPTY
};
const MOUNTAIN_SETTER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::NonbasicLandsEnterTapped],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Type,
            fragments: &[StaticAbilityFragment::NonbasicLandTypeWhileCounter {
                kind: BasicLandType::Mountain,
                counter: CounterKind::Conqueror,
            }],
        },
    ],
    ..CardRules::EMPTY
};
const ALL_BASIC_TYPES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::GrantAllBasicLandTypesUntilCleanup],
    ),
    ..CardRules::EMPTY
};
const ANIMATE: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 2,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        &[
            Instruction::AnimateEarthbendTargets,
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 2,
            },
            Instruction::RegisterEarthbendReturn,
        ],
    ),
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn granted_mana(game: &Game, card: CardId, color: usize) -> Action {
    let mut production = [0; 6];
    production[color] = 1;
    Action::TapGrantedMana {
        card,
        grantor: game.object_ref(card),
        production,
    }
}

#[test]
fn nontoken_artifacts_are_lands_at_entry_and_can_keep_an_independent_animation() {
    let _rules = fixture_rules::activate(&[
        (named("Hill Giant"), &ARTIFACT_LANDS),
        (named("Goblin Piker"), &MOUNTAIN_SETTER),
        (named("Opt"), &ANIMATE),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let toph = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    put(&mut game, player.other(), "Goblin Piker", Zone::Battlefield);
    let equipment = put(&mut game, player, "Short Bow", Zone::Hand);
    let opposing = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let token = spawn(
        &mut game,
        player,
        Card::from_catalogue_id(5296),
        Zone::Battlefield,
    );
    let animate = put(&mut game, player, "Opt", Zone::Hand);
    assert!(
        !game
            .legal_actions()
            .contains(&Action::PlayLand { card: equipment })
    );
    cast(&mut game, equipment, None);
    settle(&mut game);
    let object = game.object_view(equipment);
    assert_eq!(object.kinds, 2 | 16);
    assert!(object.tapped);
    assert_eq!(game.object_view(opposing).kinds, 16);
    assert_eq!(game.object_view(token).kinds, 16);
    assert!(game.printed_abilities_available(equipment));
    game.state.objects[equipment as usize].tapped = false;
    let red = granted_mana(&game, equipment, 3);
    reject(&mut game, red);
    cast(&mut game, animate, Some(Target::Card(equipment)));
    settle(&mut game);
    game.move_object(toph, Zone::Exile);
    let object = game.object_view(equipment);
    assert_eq!(object.kinds, 1 | 2 | 16);
    assert!(object.has_subtype("Equipment"));
    assert!(object.keywords.contains(Keywords::HASTE));
    assert_eq!((object.power, object.toughness), (2, 2));
}

#[test]
fn global_tapped_entry_and_mountain_replacement_apply_to_both_seats() {
    let _rules = fixture_rules::activate(&[(named("Goblin Piker"), &MOUNTAIN_SETTER)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let zhao = put(&mut game, Player::P0, "Goblin Piker", Zone::Battlefield);
        let passage = put(&mut game, player, "Fabled Passage", Zone::Hand);
        act(&mut game, Action::PlayLand { card: passage });
        assert!(game.object_view(passage).tapped);
        assert!(game.printed_abilities_available(passage));

        game.add_counter(zhao, CounterKind::Conqueror, 1);
        game.state.land_played = false;
        let shock_land = put(&mut game, player, "Breeding Pool", Zone::Hand);
        let life = game.state.players[player.index()].life;
        act(&mut game, Action::PlayLand { card: shock_land });
        let land = game.object_view(shock_land);
        assert_eq!(land.zone, Zone::Battlefield);
        assert!(land.tapped && land.has_subtype("Mountain"));
        assert!(!land.has_subtype("Forest") && !land.has_subtype("Island"));
        assert_eq!(game.state.pending, Pending::Priority);
        assert_eq!(game.state.players[player.index()].life, life);
        assert!(!game.printed_abilities_available(shock_land));
        let island = put(&mut game, player, "Island", Zone::Battlefield);
        assert!(game.object_view(island).has_subtype("Island"));
        game.state.objects[shock_land as usize].tapped = false;
        let red = granted_mana(&game, shock_land, 3);
        act(&mut game, red);
        game.move_object(zhao, Zone::Exile);
        let restored = game.object_view(shock_land);
        assert!(restored.has_subtype("Island") && restored.has_subtype("Forest"));
        assert!(!restored.has_subtype("Mountain"));
        assert!(game.printed_abilities_available(shock_land));
    }
}

#[test]
fn all_basic_land_types_are_additive_and_frozen_through_cleanup() {
    let _rules = fixture_rules::activate(&[(named("Opt"), &ALL_BASIC_TYPES)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let cave = put(&mut game, player, "Hidden Courtyard", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Hidden Courtyard",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);
    let late = put(&mut game, player, "Hidden Courtyard", Zone::Battlefield);
    let view = game.object_view(cave);
    assert!(view.has_subtype("Cave"));
    assert!(
        BasicLandType::ALL
            .into_iter()
            .all(|kind| view.has_subtype(kind.name()))
    );
    assert!(!game.object_view(enemy).has_subtype("Mountain"));
    assert!(!game.object_view(late).has_subtype("Mountain"));
    let mut white = game.clone();
    let before = white.state.players[player.index()].mana[0];
    act(&mut white, Action::TapMana { card: cave });
    assert_eq!(white.state.players[player.index()].mana[0], before + 1);
    for color in 1..5 {
        assert!(
            game.legal_actions()
                .contains(&granted_mana(&game, cave, color))
        );
    }
    game.state.step = Step::Cleanup;
    game.finish_cleanup();
    let view = game.object_view(cave);
    assert!(view.has_subtype("Cave"));
    assert!(
        !BasicLandType::ALL
            .into_iter()
            .any(|kind| view.has_subtype(kind.name()))
    );
}
