//! God death returns, face-specific abilities, and Temple activation gates.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn lci(game: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, owner, Card::from_name(name).unwrap(), zone)
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn reach(game: &mut Game, predicate: impl Fn(&Game) -> bool) {
    for _ in 0..24 {
        if predicate(game) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::FinishBlockers)
        });
    }
    panic!("did not reach the intended decision");
}

fn transform(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 1,
        counter_payment: Counters::default(),
    }
}

fn tap_for(card: CardId, color: usize) -> Action {
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

fn shock(game: &mut Game, player: Player, target: Target) -> CardId {
    let spell = put(game, player, "Shock", Zone::Hand);
    cast(game, spell, Some(target));
    settle(game);
    spell
}

#[test]
fn gods_return_as_their_tapped_temple_under_the_owner_and_each_produces_its_color() {
    let cases = [
        ("Ojer Axonil, Deepest Might", "Temple of Power", 3, 0),
        ("Ojer Kaslem, Deepest Growth", "Temple of Cultivation", 4, 0),
        (
            "Ojer Pakpatiq, Deepest Epoch",
            "Temple of Cyclical Time",
            1,
            3,
        ),
    ];
    for owner in [Player::P0, Player::P1] {
        for (front, back, color, time) in cases {
            let mut game = scenario(owner);
            let god = lci(&mut game, owner, front, Zone::Battlefield);
            let original = game.object_ref(god);
            game.record_control(original, owner.other(), None);
            let murder = put(&mut game, owner, "Murder", Zone::Hand);
            cast(&mut game, murder, Some(Target::Card(god)));
            pass_pair(&mut game);
            assert_eq!(game.state.stack.last().unwrap().controller, owner.other());
            assert_eq!(game.object_view(god).zone, Zone::Graveyard);
            settle(&mut game);
            let temple = game.object_view(god);
            assert_eq!(temple.name, back);
            assert!(temple.face.unwrap().back);
            assert_eq!(temple.controller, owner);
            assert!(temple.tapped);
            assert_eq!(temple.counters.get(CounterKind::Time), time);
            assert_ne!(temple.reference, original);
            reject(&mut game, tap_for(god, color));
            game.state.objects[god as usize].tapped = false;
            let prior = game.state.players[owner.index()].mana[color];
            act(&mut game, tap_for(god, color));
            assert_eq!(game.state.players[owner.index()].mana[color], prior + 1);
            assert_eq!(
                game.counter_count(god, CounterKind::Time),
                time.saturating_sub(1)
            );
        }
    }
}

#[test]
fn axonil_floors_red_damage_to_the_opponent_and_temple_counts_creature_damage() {
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        let god = lci(
            &mut game,
            player,
            "Ojer Axonil, Deepest Might",
            Zone::Battlefield,
        );
        assert_eq!(stats(&game, god), (4, 4));
        assert!(game.keywords(god).contains(Keywords::TRAMPLE));
        shock(&mut game, player, Target::Player(player));
        assert_eq!(game.state.players[player.index()].life, STARTING_LIFE - 2);
        game.state.objects[god as usize].growth.power = 1;
        shock(&mut game, player, Target::Player(other));
        assert_eq!(game.state.players[other.index()].life, STARTING_LIFE - 5);

        let mut game = scenario(player);
        let temple = lci(&mut game, player, "Temple of Power", Zone::Battlefield);
        let creature = put(&mut game, other, "Hill Giant", Zone::Battlefield);
        shock(&mut game, player, Target::Card(creature));
        reject(&mut game, transform(temple));
        shock(&mut game, player, Target::Card(creature));
        assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
        assert_eq!(game.state.players[other.index()].life, STARTING_LIFE);
        let reference = game.object_ref(temple);
        act(&mut game, transform(temple));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_ref(temple), reference);
        assert_eq!(game.object_view(temple).name, "Ojer Axonil, Deepest Might");
        assert!(game.object_view(temple).tapped);
    }
}

#[test]
fn kaslem_uses_actual_combat_damage_to_reveal_and_select_distinct_types() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let god = lci(
            &mut game,
            player,
            "Ojer Kaslem, Deepest Growth",
            Zone::Battlefield,
        );
        let creature = put(&mut game, player, "Bear Cub", Zone::Library);
        let other_creature = put(&mut game, player, "Grizzly Bears", Zone::Library);
        let land = put(&mut game, player, "Forest", Zone::Library);
        let other_land = put(&mut game, player, "Island", Zone::Library);
        let spell = put(&mut game, player, "Shock", Zone::Library);
        let artifact = put(&mut game, player, "Swiftfoot Boots", Zone::Library);
        assert_eq!(stats(&game, god), (6, 5));
        assert!(game.keywords(god).contains(Keywords::TRAMPLE));

        pass_pair(&mut game);
        pass_pair(&mut game);
        act(&mut game, Action::Attack { card: god });
        act(&mut game, Action::FinishAttackers);
        reach(&mut game, |game| {
            game.observe(player).library_selection.is_some()
        });
        let revealed = game.observe(player.other()).library_selection.unwrap();
        assert!(revealed.public);
        assert_eq!(revealed.remaining.len(), 6);
        assert_eq!(
            game.state.players[player.other().index()].life,
            STARTING_LIFE - 6
        );
        act(&mut game, Action::SelectLibrary { card: creature });
        reject(
            &mut game,
            Action::SelectLibrary {
                card: other_creature,
            },
        );
        reject(&mut game, Action::SelectLibrary { card: artifact });
        act(&mut game, Action::SelectLibrary { card: land });
        act(&mut game, Action::FinishLibrarySelection);
        assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
        assert_eq!(game.object_view(land).zone, Zone::Battlefield);
        for card in [other_creature, other_land, spell, artifact] {
            assert_eq!(game.object_view(card).zone, Zone::Library);
        }
    }
}

#[test]
fn cultivation_counts_all_controlled_permanents_and_requires_sorcery_timing() {
    let player = Player::P0;
    let mut game = scenario(player);
    let temple = lci(
        &mut game,
        player,
        "Temple of Cultivation",
        Zone::Battlefield,
    );
    for _ in 0..8 {
        put(&mut game, player, "Forest", Zone::Battlefield);
    }
    put(
        &mut game,
        player.other(),
        "Swiftfoot Boots",
        Zone::Battlefield,
    );
    reject(&mut game, transform(temple));
    game.create_tokens(player, Card::from_catalogue_id(5323), 1, false);
    game.state.step = Step::Upkeep;
    reject(&mut game, transform(temple));
    game.state.step = Step::Main1;
    act(&mut game, transform(temple));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(temple).name, "Ojer Kaslem, Deepest Growth");
}

#[test]
fn pakpatiq_rebounds_hand_instants_and_time_mana_works_at_zero() {
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        let god = lci(
            &mut game,
            player,
            "Ojer Pakpatiq, Deepest Epoch",
            Zone::Battlefield,
        );
        assert_eq!(stats(&game, god), (4, 3));
        assert!(game.keywords(god).contains(Keywords::FLYING));
        let creature = put(&mut game, other, "Hill Giant", Zone::Battlefield);
        let sorcery = put(&mut game, player, "Rumbling Rockslide", Zone::Hand);
        cast(&mut game, sorcery, Some(Target::Card(creature)));
        settle(&mut game);
        assert_eq!(game.object_view(sorcery).zone, Zone::Graveyard);
        let instant = shock(&mut game, player, Target::Player(other));
        assert_eq!(game.object_view(instant).zone, Zone::Exile);
        let exiled = game.object_ref(instant);
        game.begin_step(Step::Upkeep);
        game.give_priority();
        game.prepare_triggers();
        let recast = Action::CastExiledDuringResolution {
            object: exiled,
            face: CastFace::Normal,
        };
        reach(&mut game, |game| game.legal_actions().contains(&recast));
        act(&mut game, Action::FinishResolutionCasts);
        assert_eq!(game.object_ref(instant), exiled);

        let mut game = scenario(player);
        let temple = lci(
            &mut game,
            player,
            "Temple of Cyclical Time",
            Zone::Battlefield,
        );
        game.add_counter(temple, CounterKind::Time, 2);
        for remaining in [1, 0, 0] {
            if game.counter_count(temple, CounterKind::Time) != 0 {
                reject(&mut game, transform(temple));
            }
            let prior = game.state.players[player.index()].mana[1];
            act(&mut game, tap_for(temple, 1));
            assert_eq!(game.state.players[player.index()].mana[1], prior + 1);
            assert_eq!(game.counter_count(temple, CounterKind::Time), remaining);
            assert!(game.state.stack.is_empty());
            game.state.objects[temple as usize].tapped = false;
        }
        act(&mut game, transform(temple));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(
            game.object_view(temple).name,
            "Ojer Pakpatiq, Deepest Epoch"
        );
    }
}
