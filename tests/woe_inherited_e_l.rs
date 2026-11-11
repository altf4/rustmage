//! Paid outcome proofs for inherited Wilds of Eldraine definitions, E through L.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const FOOD: Card = Card::from_catalogue_id(5246);
const RAT: Card = Card::from_catalogue_id(5323);

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn tokens(game: &Game, player: Player, card: Card) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, object)| {
            object.zone == Zone::Battlefield && object.controller == player && object.card == card
        })
        .map(|(id, _)| id as CardId)
        .collect()
}

fn end_turn(game: &mut Game) {
    game.state.step = Step::End;
    game.give_priority();
    pass_pair(game);
}

fn next_own_main(game: &mut Game, player: Player) {
    end_turn(game);
    end_turn(game);
    assert_eq!(game.state.active, player);
    pass_pair(game);
    pass_pair(game);
    assert_eq!(game.state.step, Step::Main1);
}

#[test]
fn edgewall_pack_and_harried_spearguard_create_rats_at_distinct_events() {
    let player = Player::P0;
    let mut game = scenario(player);
    let pack = put(&mut game, player, "Edgewall Pack", Zone::Hand);
    let guard = put(&mut game, player, "Harried Spearguard", Zone::Hand);
    let destroy = put(&mut game, player, "Murder", Zone::Hand);
    announce(&mut game, pack, &[]);
    settle(&mut game);
    assert_eq!(stats(&game, pack), (3, 3));
    assert!(game.object_view(pack).keywords.contains(Keywords::MENACE));
    assert_eq!(tokens(&game, player, RAT).len(), 1);

    announce(&mut game, guard, &[]);
    settle(&mut game);
    assert_eq!(stats(&game, guard), (1, 1));
    assert!(game.object_view(guard).keywords.contains(Keywords::HASTE));
    assert_eq!(tokens(&game, player, RAT).len(), 1);
    cast(&mut game, destroy, Some(Target::Card(guard)));
    settle(&mut game);
    assert_eq!(game.object_view(guard).zone, Zone::Graveyard);
    let rats = tokens(&game, player, RAT);
    assert_eq!(rats.len(), 2);
    for rat in rats {
        let rat = game.object_view(rat);
        assert_eq!((rat.power, rat.toughness, rat.colors), (1, 1, 0b00100));
        assert!(rat.is_token() && rat.has_subtype("Rat"));
        assert!(rat.keywords.contains(Keywords::CANT_BLOCK));
    }
}

#[test]
fn archivist_tracks_independent_entry_quotas_and_confectioner_observes_sacrifices() {
    let player = Player::P1;
    let mut game = scenario(player);
    let archivist = put(&mut game, player, "Elvish Archivist", Zone::Hand);
    let goose = put(&mut game, player, "The Goose Mother", Zone::Hand);
    let confectioner = put(&mut game, player, "Experimental Confectioner", Zone::Hand);
    let destroy = put(&mut game, player, "Disenchant", Zone::Hand);
    let aura = put(&mut game, player, "Bestial Bloodline", Zone::Hand);
    let second_aura = put(&mut game, player, "Bestial Bloodline", Zone::Hand);
    let next_aura = put(&mut game, player, "Bestial Bloodline", Zone::Hand);
    let later_artifact = put(&mut game, player, "Gingerbrute", Zone::Hand);
    let forests = [0; 3].map(|_| put(&mut game, player, "Forest", Zone::Battlefield));
    announce(&mut game, archivist, &[]);
    settle(&mut game);
    act(&mut game, Action::AnnounceCast { card: goose });
    act(&mut game, Action::ChooseX { value: 3 });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(tokens(&game, player, FOOD).len(), 2);
    assert_eq!(game.counter_count(archivist, CounterKind::PlusOne), 2);

    announce(&mut game, confectioner, &[]);
    settle(&mut game);
    let food = tokens(&game, player, FOOD);
    assert_eq!(food.len(), 3);
    assert_eq!(game.counter_count(archivist, CounterKind::PlusOne), 2);
    let mut destroyed = game.clone();
    cast(&mut destroyed, destroy, Some(Target::Card(food[0])));
    settle(&mut destroyed);
    assert!(tokens(&destroyed, player, RAT).is_empty());

    act(&mut game, activate(food[0], 0));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(tokens(&game, player, RAT).len(), 1);
    assert_eq!(tokens(&game, player, FOOD).len(), 2);
    assert_eq!(game.state.players[player.index()].life, 23);

    for enchantment in [aura, second_aura] {
        cast(&mut game, enchantment, Some(Target::Card(archivist)));
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    }
    next_own_main(&mut game, player);
    for card in forests {
        act(&mut game, Action::TapMana { card });
    }
    announce(&mut game, later_artifact, &[]);
    settle(&mut game);
    assert_eq!(game.counter_count(archivist, CounterKind::PlusOne), 4);
    cast(&mut game, next_aura, Some(Target::Card(archivist)));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}

#[test]
fn food_coma_and_glass_casket_return_the_exact_exile_and_handle_early_departure() {
    for (player, name, food_count) in [
        (Player::P0, "Food Coma", 1),
        (Player::P1, "Glass Casket", 0),
    ] {
        let opponent = player.other();
        let mut game = scenario(player);
        let prison = put(&mut game, player, name, Zone::Hand);
        let victim = put(&mut game, opponent, "Redcap Thief", Zone::Battlefield);
        let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let large = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
        let destroy = put(&mut game, opponent, "Disenchant", Zone::Hand);
        let original = game.object_ref(victim);
        announce(&mut game, prison, &[]);
        pass_pair(&mut game);
        reject(&mut game, target(own));
        if name == "Glass Casket" {
            reject(&mut game, target(large));
        }
        act(&mut game, target(victim));
        let mut early = game.clone();
        act(&mut early, Action::Pass);
        cast(&mut early, destroy, Some(Target::Card(prison)));
        settle(&mut early);
        assert_eq!(early.object_ref(victim), original);
        assert_eq!(early.object_view(victim).zone, Zone::Battlefield);
        assert_eq!(tokens(&early, player, FOOD).len(), food_count);

        settle(&mut game);
        assert_eq!(game.object_view(victim).zone, Zone::Exile);
        let exiled = game.object_ref(victim);
        assert_ne!(exiled, original);
        assert_eq!(tokens(&game, player, FOOD).len(), food_count);
        act(&mut game, Action::Pass);
        cast(&mut game, destroy, Some(Target::Card(prison)));
        settle(&mut game);
        assert_eq!(game.object_view(prison).zone, Zone::Graveyard);
        let returned = game.object_view(victim);
        assert_eq!(returned.zone, Zone::Battlefield);
        assert_eq!(returned.controller, opponent);
        assert_ne!(returned.reference, exiled);
    }
}

#[test]
fn faerie_dreamthief_surveils_then_pays_to_exile_itself_from_the_graveyard() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let dreamthief = put(&mut game, player, "Faerie Dreamthief", Zone::Hand);
        let remove = put(&mut game, player, "Murder", Zone::Hand);
        let top = game.top_library_card(player).unwrap();
        announce(&mut game, dreamthief, &[]);
        pass_pair(&mut game);
        assert!(
            game.object_view(dreamthief)
                .keywords
                .contains(Keywords::FLYING)
        );
        reject(&mut game, activate(dreamthief, 0));
        pass_pair(&mut game);
        let view = game.observe(player).library_choice.unwrap();
        assert!(view.surveil);
        assert_eq!(view.remaining.len(), 1);
        assert!(game.observe(player.other()).library_choice.is_none());
        act(
            &mut game,
            Action::ArrangeLibrary {
                card: top,
                destination: LibraryDestination::Graveyard,
            },
        );
        assert_eq!(game.object_view(top).zone, Zone::Graveyard);

        cast(&mut game, remove, Some(Target::Card(dreamthief)));
        settle(&mut game);
        assert_eq!(game.object_view(dreamthief).zone, Zone::Graveyard);
        let drawn = game.top_library_card(player).unwrap();
        act(&mut game, activate(dreamthief, 0));
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 1, 0, 0, 2],
            },
        );
        assert_eq!(game.object_view(dreamthief).zone, Zone::Exile);
        settle(&mut game);
        assert_eq!(game.object_view(drawn).zone, Zone::Hand);
        assert_eq!(game.state.players[player.index()].life, 19);
    }
}

#[test]
fn farsight_ritual_looks_at_four_or_eight_and_keeps_exactly_two_privately() {
    for player in [Player::P0, Player::P1] {
        let mut parent = scenario(player);
        let ritual = put(&mut parent, player, "Farsight Ritual", Zone::Hand);
        let food = spawn(&mut parent, player, FOOD, Zone::Battlefield);
        let illegal = put(&mut parent, player, "Bear Cub", Zone::Battlefield);
        let library = parent.state.players[player.index()].library.clone();
        act(&mut parent, Action::AnnounceCast { card: ritual });
        let object = parent.object_ref(illegal);
        reject(
            &mut parent,
            Action::ChooseBargain {
                sacrifice: Some(object),
            },
        );

        for (sacrifice, look) in [(None, 4), (Some(parent.object_ref(food)), 8)] {
            let mut game = parent.clone();
            act(&mut game, Action::ChooseBargain { sacrifice });
            pay(&mut game);
            assert_eq!(
                game.object_view(food).zone == Zone::Battlefield,
                sacrifice.is_none()
            );
            pass_pair(&mut game);
            let view = game.observe(player).library_selection.unwrap();
            assert_eq!(view.remaining.len(), look);
            assert_eq!(view.minimum, 2);
            assert!(game.observe(player.other()).library_selection.is_none());
            let mut looked = library[library.len() - look..].to_vec();
            let second = looked.pop().unwrap();
            let first = looked.remove(0);
            act(&mut game, Action::SelectLibrary { card: first });
            reject(&mut game, Action::FinishLibrarySelection);
            act(&mut game, Action::SelectLibrary { card: second });
            let mut replay = game.clone();
            for branch in [&mut game, &mut replay] {
                act(branch, Action::FinishLibrarySelection);
                assert_eq!(branch.object_view(first).zone, Zone::Hand);
                assert_eq!(branch.object_view(second).zone, Zone::Hand);
                let mut bottom = branch.state.players[player.index()].library[..look - 2].to_vec();
                bottom.sort_unstable();
                assert_eq!(bottom, looked);
            }
            assert_eq!(replay.snapshot(), game.snapshot());
        }
    }
}

#[test]
fn frostbridge_guard_requires_an_untapped_source_and_freeze_stuns_an_already_tapped_creature() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let guard = put(&mut game, player, "Frostbridge Guard", Zone::Hand);
    let spell = put(&mut game, player, "Freeze in Place", Zone::Hand);
    let victim = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let plains = [0; 3].map(|_| put(&mut game, player, "Plains", Zone::Battlefield));
    let islands = [0; 2].map(|_| put(&mut game, player, "Island", Zone::Battlefield));
    announce(&mut game, guard, &[]);
    settle(&mut game);
    reject(&mut game, activate(guard, 0));

    next_own_main(&mut game, player);
    for card in plains {
        act(&mut game, Action::TapMana { card });
    }
    act(&mut game, activate(guard, 0));
    act(&mut game, target(victim));
    act(
        &mut game,
        Action::PayCosts {
            payment: [3, 0, 0, 0, 0, 0],
        },
    );
    settle(&mut game);
    assert!(game.object_view(guard).tapped);
    assert!(game.object_view(victim).tapped);
    for card in islands {
        act(&mut game, Action::TapMana { card });
    }
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, target(guard));
    act(&mut game, target(victim));
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.counter_count(victim, CounterKind::Stun), 3);
    let view = game.observe(player).library_choice.unwrap();
    assert_eq!(view.remaining.len(), 2);
    assert!(!view.surveil);
    assert!(game.observe(opponent).library_choice.is_none());
    for card in view.remaining {
        act(
            &mut game,
            Action::ArrangeLibrary {
                card: card.id,
                destination: LibraryDestination::Bottom,
            },
        );
    }
    settle(&mut game);
    end_turn(&mut game);
    assert!(game.object_view(victim).tapped);
    assert_eq!(game.counter_count(victim, CounterKind::Stun), 2);
}
