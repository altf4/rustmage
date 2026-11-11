//! Actual token creation retains each pinned Ixalan recipe's characteristics.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ANGEL: Card = Card::from_catalogue_id(5569);
const DINOSAUR: Card = Card::from_catalogue_id(5570);
const EGG: Card = Card::from_catalogue_id(5571);
const FUNGUS_DINOSAUR: Card = Card::from_catalogue_id(5572);
const GNOME_SOLDIER: Card = Card::from_catalogue_id(5573);
const GOLEM: Card = Card::from_catalogue_id(5574);
const VAMPIRE: Card = Card::from_catalogue_id(5575);
const MERFOLK: Card = Card::from_catalogue_id(5576);
const SKELETON: Card = Card::from_catalogue_id(5577);
const DEMON: Card = Card::from_catalogue_id(5578);

fn create(game: &mut Game, controller: Player, definition: Card) -> CardId {
    let id = game.state.objects.len() as CardId;
    game.create_tokens(controller, definition, 1, false);
    assert_eq!(game.object_view(id).card, definition);
    id
}

#[test]
fn fixed_recipes_make_the_exact_typed_creatures() {
    struct Expected {
        card: Card,
        name: &'static str,
        kinds: u16,
        colors: u8,
        subtypes: &'static [&'static str],
        stats: (i128, i128),
        keywords: Keywords,
    }
    let rows = [
        Expected {
            card: ANGEL,
            name: "Angel Token",
            kinds: 1,
            colors: 1,
            subtypes: &["Angel"],
            stats: (4, 4),
            keywords: Keywords::FLYING.union(Keywords::VIGILANCE),
        },
        Expected {
            card: DINOSAUR,
            name: "Dinosaur Token",
            kinds: 1,
            colors: 8,
            subtypes: &["Dinosaur"],
            stats: (3, 1),
            keywords: Keywords::NONE,
        },
        Expected {
            card: EGG,
            name: "Dinosaur Egg Token",
            kinds: 1,
            colors: 16,
            subtypes: &["Dinosaur", "Egg"],
            stats: (0, 1),
            keywords: Keywords::NONE,
        },
        Expected {
            card: GOLEM,
            name: "Golem Token",
            kinds: 17,
            colors: 3,
            subtypes: &["Golem"],
            stats: (4, 4),
            keywords: Keywords::NONE,
        },
        Expected {
            card: VAMPIRE,
            name: "Vampire Token",
            kinds: 1,
            colors: 1,
            subtypes: &["Vampire"],
            stats: (1, 1),
            keywords: Keywords::LIFELINK,
        },
        Expected {
            card: MERFOLK,
            name: "Merfolk Token",
            kinds: 1,
            colors: 2,
            subtypes: &["Merfolk"],
            stats: (1, 1),
            keywords: Keywords::HEXPROOF,
        },
        Expected {
            card: SKELETON,
            name: "Skeleton Pirate Token",
            kinds: 1,
            colors: 4,
            subtypes: &["Skeleton", "Pirate"],
            stats: (2, 2),
            keywords: Keywords::NONE,
        },
        Expected {
            card: DEMON,
            name: "Vampire Demon Token",
            kinds: 1,
            colors: 5,
            subtypes: &["Vampire", "Demon"],
            stats: (4, 3),
            keywords: Keywords::FLYING,
        },
    ];
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        for row in &rows {
            let id = create(&mut game, player, row.card);
            let view = game.object_view(id);
            assert!(view.is_token());
            assert_eq!(view.name, row.name);
            assert_eq!(view.zone, Zone::Battlefield);
            assert_eq!(view.owner, player);
            assert_eq!(view.controller, player);
            assert_eq!(view.kinds, row.kinds);
            assert_eq!(view.colors, row.colors);
            assert_eq!((view.power, view.toughness), row.stats);
            assert_eq!(view.keywords, row.keywords);
            assert_eq!(view.card.fixed_mana_value(), 0);
            assert!(row.subtypes.iter().all(|subtype| view.has_subtype(subtype)));
        }
    }
}

#[test]
fn gnome_soldier_counts_the_live_union_including_itself_and_its_copies() {
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        let id = create(&mut game, player, GNOME_SOLDIER);
        let view = game.object_view(id);
        assert!(view.is_token() && view.has_subtype("Gnome") && view.has_subtype("Soldier"));
        assert_eq!(view.name, "Gnome Soldier Token");
        assert_eq!(view.kinds, 17);
        assert_eq!(view.colors, 1);
        assert_eq!(stats(&game, id), (1, 1));

        let golem = create(&mut game, player, GOLEM);
        put(&mut game, player, "Forest", Zone::Battlefield);
        put(&mut game, player, "Gingerbrute", Zone::Graveyard);
        put(&mut game, player, "Bear Cub", Zone::Battlefield);
        put(&mut game, player, "Short Bow", Zone::Battlefield);
        put(&mut game, other, "Gingerbrute", Zone::Battlefield);
        assert_eq!(stats(&game, id), (4, 4));

        let reference = game.object_ref(id);
        let recipe = game.copiable_recipe(reference);
        let copy = game.state.objects.len() as CardId;
        game.stage_copy_creation(player, recipe, 1);
        game.advance_replacements(true);
        game.advance_entries();
        assert_eq!(stats(&game, id), (5, 5));
        assert_eq!(stats(&game, copy), (5, 5));

        game.record_control(reference, other, None);
        assert_eq!(stats(&game, id), (2, 2));
        assert_eq!(stats(&game, copy), (4, 4));
        game.record_control(game.object_ref(golem), other, None);
        assert_eq!(stats(&game, id), (3, 3));
        assert_eq!(stats(&game, copy), (3, 3));
    }
}

#[test]
fn fungus_dinosaur_copies_the_sampled_base_instead_of_counters_or_live_stats() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let id = game.state.objects.len() as CardId;
        game.create_sized_token(player, FUNGUS_DINOSAUR, 6);
        game.advance_replacements(true);
        game.advance_entries();
        let view = game.object_view(id);
        assert!(view.is_token() && view.has_subtype("Fungus") && view.has_subtype("Dinosaur"));
        assert_eq!(view.name, "Fungus Dinosaur Token");
        assert_eq!(view.kinds, 1);
        assert_eq!(view.colors, 16);
        assert_eq!(view.keywords, Keywords::NONE);
        assert_eq!(stats(&game, id), (6, 6));

        game.add_counter(id, CounterKind::PlusOne, 1);
        let recipe = game.copiable_recipe(game.object_ref(id));
        assert_eq!(recipe.copiable_base(), Some(Stats::from(6i128)));
        let copy = game.state.objects.len() as CardId;
        game.stage_copy_creation(player.other(), recipe, 1);
        game.advance_replacements(true);
        game.advance_entries();
        assert_eq!(stats(&game, id), (7, 7));
        assert_eq!(stats(&game, copy), (6, 6));
        assert_eq!(game.object_view(copy).controller, player.other());
        assert_eq!(game.object_view(copy).counters.plus_one, 0);
    }
}
