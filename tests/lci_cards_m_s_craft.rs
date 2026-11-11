//! Actual Ixalan fronts and faces through ordered Craft and transformation.
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

fn until_choice(game: &mut Game) {
    for _ in 0..10 {
        if game.state.pending != Pending::Priority {
            return;
        }
        act(game, Action::Pass);
    }
    panic!("expected a pending choice");
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn begin_craft(game: &mut Game, card: CardId) {
    act(game, activate(card, 0));
    act(game, Action::BeginOrderedCosts);
    let clause = game
        .observe(game.actor().unwrap())
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == CastCostClause::Additional)
        .unwrap() as u16;
    act(game, Action::ChooseCastCost { clause });
}

fn material(game: &mut Game, card: CardId) {
    let object = game.object_ref(card);
    act(game, Action::ChooseAdditionalCost { object });
}

fn finish_craft_costs(game: &mut Game) {
    for _ in 0..8 {
        if game.state.announcing.is_none() {
            return;
        }
        match game.state.pending {
            Pending::CastCosts { .. } => choose(game, |action| {
                matches!(action, Action::ChooseCastCost { .. })
            }),
            Pending::PayCosts { .. } => pay(game),
            _ => panic!("unexpected Craft cost {:?}", game.state.pending),
        }
    }
    panic!("Craft costs did not finish");
}

#[test]
fn landmark_scries_then_crafts_with_market_gnome_and_gives_an_attacker_flying() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let landmark = lci(&mut game, player, "Oteclan Landmark", Zone::Hand);
        let gnome = lci(&mut game, player, "Market Gnome", Zone::Battlefield);
        let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let flier = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
        let bottom = put(&mut game, player, "Forest", Zone::Library);
        let draw = put(&mut game, player, "Plains", Zone::Library);
        cast(&mut game, landmark, None);
        until_choice(&mut game);
        act(
            &mut game,
            Action::ArrangeLibrary {
                card: draw,
                destination: LibraryDestination::Top,
            },
        );
        act(
            &mut game,
            Action::ArrangeLibrary {
                card: bottom,
                destination: LibraryDestination::Bottom,
            },
        );
        assert_eq!(
            game.state.players[player.index()].library.last(),
            Some(&draw)
        );
        assert_eq!(
            game.state.players[player.index()].library.first(),
            Some(&bottom)
        );

        let old = game.object_ref(landmark);
        let life = game.state.players[player.index()].life;
        begin_craft(&mut game, landmark);
        material(&mut game, gnome);
        finish_craft_costs(&mut game);
        assert_eq!(game.object_view(gnome).zone, Zone::Exile);
        assert_eq!(game.object_view(landmark).zone, Zone::Exile);
        assert_eq!(
            game.state
                .stack
                .last()
                .unwrap()
                .source_identity()
                .unwrap()
                .0
                .card,
            gnome
        );
        pass_pair(&mut game);
        assert_eq!(game.state.players[player.index()].life, life + 1);
        assert_eq!(game.object_view(draw).zone, Zone::Hand);
        settle(&mut game);
        assert_ne!(game.object_ref(landmark), old);
        assert_eq!(game.object_view(landmark).name, "Oteclan Levitator");
        assert_eq!(stats(&game, landmark), (1, 4));
        assert!(game.keywords(landmark).contains(Keywords::FLYING));
        assert!(game.observe(player).linked_exile.iter().any(|link| {
            link.source == game.object_ref(landmark) && link.exiled == game.object_ref(gnome)
        }));

        game.state.objects[landmark as usize].summoning_sick = false;
        pass_pair(&mut game);
        pass_pair(&mut game);
        for card in [landmark, attacker, flier] {
            act(&mut game, Action::Attack { card });
        }
        act(&mut game, Action::FinishAttackers);
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(flier),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(attacker),
            },
        );
        settle(&mut game);
        assert!(game.keywords(attacker).contains(Keywords::FLYING));
        game.state.step = Step::End;
        pass_pair(&mut game);
        assert!(!game.keywords(attacker).contains(Keywords::FLYING));
    }
}

#[test]
fn market_gnome_dies_for_one_life_and_one_draw() {
    let player = Player::P1;
    let mut game = scenario(player);
    let gnome = lci(&mut game, player, "Market Gnome", Zone::Battlefield);
    let draw = put(&mut game, player, "Island", Zone::Library);
    assert_eq!(stats(&game, gnome), (0, 3));
    let spell = put(&mut game, player, "Murder", Zone::Hand);
    let life = game.state.players[player.index()].life;
    cast(&mut game, spell, Some(Target::Card(gnome)));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, life + 1);
    assert_eq!(game.object_view(gnome).zone, Zone::Graveyard);
    assert_eq!(game.object_view(draw).zone, Zone::Hand);
}

#[test]
fn lattice_may_rummage_then_raptor_uses_live_craft_links_and_exile_power() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let lattice = lci(&mut game, player, "Saheeli's Lattice", Zone::Hand);
        let discard = put(&mut game, player, "Opt", Zone::Hand);
        let first = put(&mut game, player, "Island", Zone::Library);
        let second = put(&mut game, player, "Forest", Zone::Library);
        let living = put(&mut game, player, "Colossadactyl", Zone::Battlefield);
        let dead = put(&mut game, player, "Pathfinding Axejaw", Zone::Graveyard);
        let ineligible = put(&mut game, player, "Bear Cub", Zone::Graveyard);
        game.state.objects[living as usize].growth.power = 9;
        cast(&mut game, lattice, None);
        until_choice(&mut game);
        let mut decline = game.clone();
        act(&mut decline, Action::DeclineEffectPayment);
        assert_eq!(decline.object_view(first).zone, Zone::Library);
        assert_eq!(decline.object_view(discard).zone, Zone::Hand);

        act(&mut game, Action::Discard { card: discard });
        assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
        for card in [first, second] {
            assert_eq!(game.object_view(card).zone, Zone::Hand);
        }
        begin_craft(&mut game, lattice);
        let object = game.object_ref(ineligible);
        reject(&mut game, Action::ChooseAdditionalCost { object });
        reject(&mut game, Action::FinishCraftMaterials);
        material(&mut game, living);
        material(&mut game, dead);
        act(&mut game, Action::FinishCraftMaterials);
        finish_craft_costs(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(lattice).name, "Mastercraft Raptor");
        assert_eq!(game.object_view(living).zone, Zone::Exile);
        assert_eq!(game.object_view(dead).zone, Zone::Exile);
        assert_eq!(stats(&game, lattice), (8, 4));
        game.move_object(dead, Zone::Graveyard);
        game.move_object(dead, Zone::Exile);
        assert_eq!(stats(&game, lattice), (4, 4));
    }
}

#[test]
fn matzalantli_loots_and_uses_distinct_permanent_types_while_core_counts_cards() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let door = lci(
            &mut game,
            player,
            "Matzalantli, the Great Door",
            Zone::Battlefield,
        );
        let draw = put(&mut game, player, "Shock", Zone::Library);
        let land = put(&mut game, player, "Forest", Zone::Hand);
        let other = player.other();
        put(&mut game, player, "Opt", Zone::Graveyard);
        put(&mut game, player, "Gingerbrute", Zone::Graveyard);
        put(&mut game, other, "Impact Tremors", Zone::Graveyard);
        act(&mut game, activate(door, 0));
        pay(&mut game);
        until_choice(&mut game);
        assert_eq!(game.object_view(draw).zone, Zone::Hand);
        act(&mut game, Action::Discard { card: land });
        assert_eq!(game.object_view(land).zone, Zone::Graveyard);

        game.state.objects[door as usize].tapped = false;
        reject(&mut game, activate(door, 1));
        put(&mut game, player, "Impact Tremors", Zone::Graveyard);
        game.state.step = Step::Upkeep;
        game.state.active = other;
        game.state.priority = player;
        let reference = game.object_ref(door);
        act(&mut game, activate(door, 1));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_ref(door), reference);
        assert_eq!(game.object_view(door).name, "The Core");
        assert!(game.object_view(door).tapped);

        game.state.priority = player;
        game.state.objects[door as usize].tapped = false;
        game.state.players[player.index()].mana = [0; 6];
        let mana = |production| Action::ActivateMana {
            card: door,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production,
        };
        reject(&mut game, mana([0, 4, 0, 0, 0, 0]));
        reject(&mut game, mana([0, 2, 0, 1, 0, 0]));
        act(&mut game, mana([0, 3, 0, 0, 0, 0]));
        assert_eq!(game.state.players[player.index()].mana, [0, 3, 0, 0, 0, 0]);
        assert!(game.state.stack.is_empty());
    }
}
