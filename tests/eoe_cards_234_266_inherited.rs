//! Paid gameplay for EOE's inherited late artifacts, shock lands and basics.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn until(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..96 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishAttackers
                    | Action::FinishBlockers
            )
        });
    }
    panic!("scenario did not reach its stop condition");
}

fn activate(game: &mut Game, card: CardId, ability: u16) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
}

fn mana(card: CardId, color: usize) -> Action {
    let mut production = [0; 6];
    production[color] = 1;
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}

fn tokens(game: &Game, factory: Card, player: Player) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield
                && object.is_token()
                && object.card == factory
                && object.controller == player)
                .then_some(id as CardId)
        })
        .collect()
}

#[test]
fn all_fates_scroll_taps_for_each_color_or_sacrifices_for_distinct_controlled_land_names() {
    let player = Player::P0;
    let mut game = scenario(player);
    let scroll = put(&mut game, player, "All-Fates Scroll", Zone::Battlefield);
    for name in ["Forest", "Forest", "Swamp"] {
        put(&mut game, player, name, Zone::Battlefield);
    }
    put(&mut game, player.other(), "Mountain", Zone::Battlefield);

    reject(&mut game, mana(scroll, 5));
    for color in 0..5 {
        let mut tap = game.clone();
        let mut expected = tap.state.players[player.index()].mana;
        expected[color] += 1;
        act(&mut tap, mana(scroll, color));
        assert_eq!(tap.state.players[player.index()].mana, expected);
        assert!(tap.object_view(scroll).tapped);
    }
    activate(&mut game, scroll, 1);
    pay(&mut game);
    assert_eq!(game.object_view(scroll).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.hand_count(player), 2);
}

#[test]
fn bygone_colossus_warps_as_a_nine_nine_then_uses_its_normal_cost_on_later_turns() {
    let player = Player::P1;
    let mut game = scenario(player);
    let card = put(&mut game, player, "Bygone Colossus", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
    reject(&mut game, Action::AnnounceCast { card });
    act(&mut game, Action::AnnounceWarp { card });
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(card).cast_for_warp);
    assert_eq!(stats(&game, card), (9, 9));
    let turn = game.state.turn;
    until(&mut game, |game| game.object_view(card).zone == Zone::Exile);
    assert_eq!(game.state.turn, turn);
    assert!(!game.can_cast_exiled_card(player, card));

    until(&mut game, |game| {
        game.state.turn == turn + 2 && game.state.step == Step::Main1
    });
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 8];
    reject(&mut game, Action::AnnounceExile { card });
    game.state.players[player.index()].mana[5] = 9;
    act(&mut game, Action::AnnounceExile { card });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert!(!game.object_view(card).cast_for_warp);
    assert_eq!(stats(&game, card), (9, 9));
}

#[test]
fn chrome_companion_gains_life_when_tapped_and_bottoms_a_card_in_its_owners_library() {
    let player = Player::P0;
    let mut game = scenario(player);
    let companion = put(&mut game, player, "Chrome Companion", Zone::Battlefield);
    let rival = put(&mut game, player.other(), "Grizzly Bears", Zone::Graveyard);
    let own = put(&mut game, player, "Shock", Zone::Graveyard);

    activate(&mut game, companion, 0);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(rival),
        },
    );
    pay(&mut game);
    assert!(game.object_view(companion).tapped);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 21);
    assert_eq!(game.object_view(own).zone, Zone::Graveyard);
    assert_eq!(game.object_view(rival).zone, Zone::Library);
    assert_eq!(
        game.state.players[player.other().index()].library.first(),
        Some(&rival)
    );
}

#[test]
fn dauntless_scrapbot_exiles_only_the_opponents_graveyard_and_makes_a_lander() {
    let player = Player::P1;
    let mut game = scenario(player);
    let own = put(&mut game, player, "Island", Zone::Graveyard);
    let enemy = [
        put(&mut game, player.other(), "Forest", Zone::Graveyard),
        put(&mut game, player.other(), "Shock", Zone::Graveyard),
    ];
    let robot = put(&mut game, player, "Dauntless Scrapbot", Zone::Hand);
    cast(&mut game, robot, None);
    settle(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Graveyard);
    assert!(
        enemy
            .iter()
            .all(|&id| game.object_view(id).zone == Zone::Exile)
    );
    let [lander] = tokens(&game, Card::from_catalogue_id(5238), player)
        .try_into()
        .unwrap();
    let view = game.object_view(lander);
    assert_eq!((view.kinds, view.colors), (16, 0));
    assert!(view.has_subtype("Lander"));
}

#[test]
fn nutrient_block_survives_destroy_and_sacrifices_for_life_and_a_death_draw() {
    let player = Player::P0;
    let mut game = scenario(player);
    let food = put(&mut game, player, "Nutrient Block", Zone::Battlefield);
    let destroy = put(&mut game, player, "Disenchant", Zone::Hand);
    cast(&mut game, destroy, Some(Target::Card(food)));
    settle(&mut game);
    assert!(
        game.object_view(food)
            .keywords
            .contains(Keywords::INDESTRUCTIBLE)
    );
    assert_eq!(game.object_view(food).zone, Zone::Battlefield);
    assert_eq!(game.hand_count(player), 0);

    activate(&mut game, food, 0);
    pay(&mut game);
    assert_eq!(game.object_view(food).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 23);
    assert_eq!(game.hand_count(player), 1);
}

#[test]
fn thrumming_hivepool_counts_controlled_slivers_grants_both_keywords_and_creates_two_on_upkeep() {
    let player = Player::P0;
    let mut game = scenario(player);
    let sliver = Card::from_catalogue_id(5295);
    for controller in [player, player.other()] {
        game.create_tokens(controller, sliver, 1, false);
        assert!(!game.advance_entries());
    }
    let pool = put(&mut game, player, "Thrumming Hivepool", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 4];
    reject(&mut game, Action::AnnounceCast { card: pool });
    game.create_tokens(player, sliver, 1, false);
    assert!(!game.advance_entries());
    cast(&mut game, pool, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);

    let keywords = Keywords::HASTE.union(Keywords::DOUBLE_STRIKE);
    for id in tokens(&game, sliver, player) {
        assert!(game.object_view(id).keywords.contains(keywords));
    }
    for id in tokens(&game, sliver, player.other()) {
        assert!(!game.object_view(id).keywords.contains(Keywords::HASTE));
        assert!(
            !game
                .object_view(id)
                .keywords
                .contains(Keywords::DOUBLE_STRIKE)
        );
    }

    let turn = game.state.turn;
    until(&mut game, |game| {
        game.state.turn == turn + 2 && game.state.step == Step::Upkeep
    });
    settle(&mut game);
    let owned = tokens(&game, sliver, player);
    assert_eq!(owned.len(), 4);
    for id in owned {
        let view = game.object_view(id);
        assert_eq!((view.colors, view.power, view.toughness), (0, 1, 1));
        assert!(view.has_subtype("Sliver"));
        assert!(view.keywords.contains(keywords));
    }
    assert_eq!(tokens(&game, sliver, player.other()).len(), 1);
}

#[test]
fn virulent_silencer_counts_each_nontoken_artifact_creatures_combat_damage() {
    let player = Player::P1;
    let mut game = scenario(player);
    let silencer = put(&mut game, player, "Virulent Silencer", Zone::Battlefield);
    let other_artifact = put(&mut game, player, "Chrome Companion", Zone::Battlefield);
    let nonartifact = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let robot = Card::from_catalogue_id(5309);
    game.create_tokens(player, robot, 1, false);
    assert!(!game.advance_entries());
    let [token] = tokens(&game, robot, player).try_into().unwrap();
    game.state.objects[token as usize].summoning_sick = false;

    until(&mut game, |game| game.state.pending == Pending::Attackers);
    for card in [silencer, other_artifact, nonartifact, token] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    until(&mut game, |game| game.state.step == Step::Main2);
    assert_eq!(game.state.players[player.other().index()].life, 12);
    assert_eq!(
        game.state.players[player.other().index()].poison_counters,
        4
    );
    assert_eq!(game.state.players[player.index()].poison_counters, 0);
}

#[test]
fn eoe_shock_lands_enter_tapped_or_pay_two_and_produce_only_their_two_colors() {
    let player = Player::P0;
    for (name, subtypes, colors) in [
        ("Breeding Pool", ["Forest", "Island"], [4, 1]),
        ("Godless Shrine", ["Plains", "Swamp"], [0, 2]),
        ("Sacred Foundry", ["Mountain", "Plains"], [3, 0]),
        ("Stomping Ground", ["Mountain", "Forest"], [3, 4]),
        ("Watery Grave", ["Island", "Swamp"], [1, 2]),
    ] {
        let mut game = scenario(player);
        let land = put(&mut game, player, name, Zone::Hand);
        act(&mut game, Action::PlayLand { card: land });
        let mut tapped = game.clone();
        act(&mut tapped, Action::ChooseEntryLife { pay: false });
        assert!(tapped.object_view(land).tapped, "{name}");
        assert_eq!(tapped.state.players[player.index()].life, 20, "{name}");
        reject(&mut tapped, Action::TapMana { card: land });

        act(&mut game, Action::ChooseEntryLife { pay: true });
        let view = game.object_view(land);
        assert_eq!(view.zone, Zone::Battlefield, "{name}");
        assert!(!view.tapped && !view.card.is_basic_land(), "{name}");
        assert!(
            subtypes.iter().all(|subtype| view.has_subtype(subtype)),
            "{name}"
        );
        assert_eq!(game.state.players[player.index()].life, 18, "{name}");
        let intrinsic = colors[0].min(colors[1]);
        let grantor = game.object_ref(land);
        for color in 0..6 {
            let action = if color == intrinsic {
                Action::TapMana { card: land }
            } else {
                let mut production = [0; 6];
                production[color] = 1;
                Action::TapGrantedMana {
                    card: land,
                    grantor,
                    production,
                }
            };
            if colors.contains(&color) {
                let mut paid = game.clone();
                let mut expected = paid.state.players[player.index()].mana;
                expected[color] += 1;
                act(&mut paid, action);
                assert_eq!(paid.state.players[player.index()].mana, expected, "{name}");
            } else {
                reject(&mut game, action);
            }
        }
    }
}

#[test]
fn eoe_basic_lands_enter_untapped_and_tap_for_their_own_color() {
    let player = Player::P1;
    for (color, name) in ["Plains", "Island", "Swamp", "Mountain", "Forest"]
        .into_iter()
        .enumerate()
    {
        let mut game = scenario(player);
        let land = put(&mut game, player, name, Zone::Hand);
        let second = put(&mut game, player, name, Zone::Hand);
        act(&mut game, Action::PlayLand { card: land });
        let view = game.object_view(land);
        assert_eq!(view.zone, Zone::Battlefield, "{name}");
        assert!(
            !view.tapped && view.card.is_basic_land() && view.has_subtype(name),
            "{name}"
        );
        reject(&mut game, Action::PlayLand { card: second });
        let mut expected = game.state.players[player.index()].mana;
        expected[color] += 1;
        act(&mut game, Action::TapMana { card: land });
        assert_eq!(game.state.players[player.index()].mana, expected, "{name}");
        assert!(game.object_view(land).tapped, "{name}");
    }
}
