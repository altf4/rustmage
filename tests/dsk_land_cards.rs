//! Real land plays, life thresholds, live subtype checks, and searches.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PLAYER: Player = Player::P0;
const OTHER: Player = Player::P1;
const ZERO: Mana = [0; 6];

fn mana(color: usize) -> Mana {
    let mut production = ZERO;
    production[color] = 1;
    production
}

fn activation(card: CardId, ability: u16, color: usize) -> Action {
    Action::ActivateMana {
        card,
        ability,
        payment: ZERO,
        counter_payment: Counters::default(),
        production: mana(color),
    }
}

fn play(game: &mut Game, name: &str) -> CardId {
    let card = put(game, PLAYER, name, Zone::Hand);
    act(game, Action::PlayLand { card });
    card
}

#[test]
fn the_ten_life_lands_check_both_players_at_thirteen_and_tap_for_either_color() {
    for (name, colors) in [
        ("Abandoned Campground", [0, 1]),
        ("Bleeding Woods", [3, 4]),
        ("Etched Cornfield", [4, 0]),
        ("Lakeside Shack", [4, 1]),
        ("Murky Sewer", [1, 2]),
        ("Neglected Manor", [0, 2]),
        ("Peculiar Lighthouse", [1, 3]),
        ("Raucous Carnival", [3, 0]),
        ("Razortrap Gorge", [2, 3]),
        ("Strangled Cemetery", [2, 4]),
    ] {
        for (life, tapped) in [([14, 14], true), ([13, 14], false), ([14, 13], false)] {
            let mut game = scenario(PLAYER);
            for (state, life) in game.state.players.iter_mut().zip(life) {
                state.life = life;
                state.mana = ZERO;
            }
            let land = play(&mut game, name);
            assert_eq!(game.object_view(land).tapped, tapped, "{name}, {life:?}");
            for color in colors {
                let mut branch = game.clone();
                let tap = activation(land, 0, color);
                if tapped {
                    reject(&mut branch, tap);
                } else {
                    act(&mut branch, tap);
                    assert_eq!(branch.state.players[PLAYER.index()].mana, mana(color));
                    assert!(branch.object_view(land).tapped);
                }
            }
        }
    }
}

#[test]
fn the_five_verges_offer_the_second_color_only_with_a_controlled_matching_subtype() {
    for (name, first, second, types, nonbasic) in [
        (
            "Blazemire Verge",
            2,
            3,
            ["Mountain", "Swamp"],
            "Undercity Sewers",
        ),
        (
            "Floodfarm Verge",
            0,
            1,
            ["Plains", "Island"],
            "Meticulous Archive",
        ),
        (
            "Gloomlake Verge",
            1,
            2,
            ["Island", "Swamp"],
            "Undercity Sewers",
        ),
        (
            "Hushwood Verge",
            4,
            0,
            ["Forest", "Plains"],
            "Shadowy Backstreet",
        ),
        (
            "Thornspire Verge",
            3,
            4,
            ["Mountain", "Forest"],
            "Commercial District",
        ),
    ] {
        let mut game = scenario(PLAYER);
        game.state.players[PLAYER.index()].mana = ZERO;
        for subtype in types {
            put(&mut game, OTHER, subtype, Zone::Battlefield);
        }
        let land = play(&mut game, name);
        assert!(!game.object_view(land).tapped);
        let alternate = activation(land, 1, second);
        reject(&mut game, alternate.clone());
        let mut normal = game.clone();
        act(&mut normal, activation(land, 0, first));
        assert_eq!(normal.state.players[PLAYER.index()].mana, mana(first));

        for enabler in types.into_iter().chain([nonbasic]) {
            let mut branch = game.clone();
            let subtype = put(&mut branch, PLAYER, enabler, Zone::Battlefield);
            let mut transferred = branch.clone();
            transferred.change_controller(transferred.object_ref(subtype), OTHER);
            reject(&mut transferred, alternate.clone());
            act(&mut branch, alternate.clone());
            assert_eq!(branch.state.players[PLAYER.index()].mana, mana(second));
            assert!(branch.object_view(land).tapped);
        }
    }
}

#[test]
fn the_five_reprinted_basic_lands_are_unlimited_and_produce_their_printed_colors() {
    for (name, color) in [
        ("Plains", 0),
        ("Island", 1),
        ("Swamp", 2),
        ("Mountain", 3),
        ("Forest", 4),
    ] {
        let card = Card::from_name(name).unwrap();
        assert!(
            Game::new(Config {
                decks: [vec![card; 60], vec![card; 60]],
                seed: 71,
                starting_player: PLAYER,
                deck_rules: DeckRules::Standard,
            })
            .is_ok()
        );
        let mut game = scenario(PLAYER);
        game.state.players[PLAYER.index()].mana = ZERO;
        let land = play(&mut game, name);
        assert!(!game.object_view(land).tapped);
        let expected = mana(color);
        choose(&mut game, |action| {
            matches!(action, Action::ActivateMana { card, production, .. }
                if *card == land && *production == expected)
        });
        assert_eq!(game.state.players[PLAYER.index()].mana, expected);
        assert!(game.object_view(land).tapped);
    }
}

#[test]
fn terramorphic_expanse_sacrifices_before_a_private_optional_basic_search() {
    let mut game = scenario(PLAYER);
    let basic = put(&mut game, PLAYER, "Island", Zone::Library);
    let nonbasic = put(&mut game, PLAYER, "Abandoned Campground", Zone::Library);
    let expanse = play(&mut game, "Terramorphic Expanse");
    assert!(!game.object_view(expanse).tapped);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: expanse,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    assert_eq!(game.object_view(expanse).zone, Zone::Graveyard);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);

    let select = Action::ChooseSearch { card: basic };
    assert!(!game.observe(OTHER).actions.contains(&select));
    reject(&mut game, Action::ChooseSearch { card: nonbasic });
    let mut decline = game.clone();
    act(&mut decline, Action::FinishSearch);
    assert_eq!(decline.object_view(basic).zone, Zone::Library);
    act(&mut game, select);
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    assert_eq!(game.object_view(basic).zone, Zone::Battlefield);
    assert!(game.object_view(basic).tapped);
    assert_eq!(game.object_view(nonbasic).zone, Zone::Library);
}

#[test]
fn valgavoths_lair_chooses_a_color_before_tapped_entry_and_has_hexproof() {
    let mut entry = scenario(PLAYER);
    let lair = play(&mut entry, "Valgavoth's Lair");
    assert!(matches!(entry.state.pending, Pending::EntryValue { .. }));
    for (index, color) in ChoiceColor::ALL.into_iter().enumerate() {
        let mut game = entry.clone();
        act(
            &mut game,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::Color(color),
            },
        );
        let view = game.object_view(lair);
        assert_eq!(view.zone, Zone::Battlefield);
        assert!(view.tapped);
        assert_eq!(view.kinds & (2 | 32), 2 | 32);
        assert!(view.keywords.contains(Keywords::HEXPROOF));
        game.state.players[PLAYER.index()].mana = ZERO;
        let tap = activation(lair, 0, index);
        reject(&mut game, tap.clone());
        game.state.objects[lair as usize].tapped = false;
        reject(&mut game, activation(lair, 0, (index + 1) % 5));
        act(&mut game, tap);
        assert_eq!(game.state.players[PLAYER.index()].mana, mana(index));

        let alternative = put(&mut game, PLAYER, "Bear Trap", Zone::Battlefield);
        let removal = put(&mut game, OTHER, "Disenchant", Zone::Hand);
        act(&mut game, Action::Pass);
        act(&mut game, Action::AnnounceCast { card: removal });
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(lair),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(alternative),
            },
        );
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(lair).zone, Zone::Battlefield);
        assert_eq!(game.object_view(alternative).zone, Zone::Graveyard);
    }
}
