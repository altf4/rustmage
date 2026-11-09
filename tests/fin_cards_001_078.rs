//! Native scenarios for the first 78 Final Fantasy cards.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn reach(game: &mut Game, predicate: impl Fn(&Action) -> bool) {
    for _ in 0..48 {
        if game.legal_actions().iter().any(&predicate) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("scenario action was never offered");
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn announce_ability(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn mana(card: CardId, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}

#[test]
fn ambrosia_flashes_in_and_bounces_to_the_owner_then_benefits_from_landfall() {
    let player = Player::P1;
    let owner = player.other();
    let mut game = scenario(owner);
    let bird = put(&mut game, player, "Ambrosia Whiteheart", Zone::Hand);
    let stolen = put(&mut game, owner, "Grizzly Bears", Zone::Battlefield);
    game.record_control(game.object_ref(stolen), player, None);
    game.state.objects[stolen as usize].gained_keywords = Keywords::PROTECTION_EVERYTHING;
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(player));
    cast(&mut game, bird, None);
    let bounce = Action::ChooseEffectObject {
        object: game.object_ref(stolen),
    };
    reach(&mut game, |a| *a == bounce);
    let own_source = Action::ChooseEffectObject {
        object: game.object_ref(bird),
    };
    reject(&mut game, own_source);
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    assert_eq!(decline.object_view(stolen).zone, Zone::Battlefield);

    act(&mut game, bounce);
    assert_eq!(game.object_view(stolen).zone, Zone::Hand);
    assert!(
        game.observe(owner)
            .objects
            .iter()
            .any(|card| card.id == stolen)
    );
    assert_eq!(stats(&game, bird), (2, 2));

    game.state.active = player;
    game.state.priority = player;
    let land = put(&mut game, player, "Plains", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    settle(&mut game);
    assert_eq!(stats(&game, bird), (3, 2));
}

#[test]
fn cloud_of_darkness_counts_only_own_graveyard_permanents() {
    let player = Player::P0;
    let mut game = scenario(player);
    let cloud = put(&mut game, player, "Cloud of Darkness", Zone::Hand);
    let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let opposing = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    put(&mut game, player, "Forest", Zone::Graveyard);
    put(&mut game, player, "Swiftfoot Boots", Zone::Graveyard);
    put(&mut game, player, "Shock", Zone::Graveyard);
    put(&mut game, player.other(), "Grizzly Bears", Zone::Graveyard);

    cast(&mut game, cloud, None);
    reach(&mut game, |a| *a == target(opposing));
    reject(&mut game, target(own));
    act(&mut game, target(opposing));
    settle(&mut game);
    assert_eq!(stats(&game, opposing), (1, 1));
    assert_eq!(stats(&game, cloud), (3, 3));
    assert!(game.keywords(cloud).contains(Keywords::FLYING));
}

#[test]
fn crossroads_chooses_as_it_enters_and_only_produces_that_color() {
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Crossroads Village", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    assert!(matches!(game.state.pending, Pending::EntryValue { .. }));
    assert_eq!(
        game.legal_actions()
            .iter()
            .filter(|action| { matches!(action, Action::ChooseEntryValue { .. }) })
            .count(),
        5,
    );
    let selection = Action::ChooseEntryValue {
        value: EntryChoiceValue::Color(ChoiceColor::Red),
    };
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == selection)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, selection);
    let revision = indexed.revision();
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(indexed.snapshot(), game.snapshot());
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert!(game.object_view(land).tapped);

    let red = [0, 0, 0, 1, 0, 0];
    reject(&mut game, mana(land, red));
    game.state.objects[land as usize].tapped = false;
    game.state.players[player.index()].mana = [0; 6];
    reject(&mut game, mana(land, [1, 0, 0, 0, 0, 0]));
    act(&mut game, mana(land, red));
    assert_eq!(game.state.players[player.index()].mana, red);
}

#[test]
fn deadly_embrace_draws_for_the_destroyed_creature_and_prior_deaths() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Deadly Embrace", Zone::Hand);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    let earlier = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    let victim = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    cast(&mut game, shock, Some(Target::Card(earlier)));
    settle(&mut game);
    assert_eq!(game.object_view(earlier).zone, Zone::Graveyard);
    cast(&mut game, spell, Some(Target::Card(victim)));
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}

#[test]
fn eden_mills_before_its_reflexive_trigger_and_cannot_return_itself() {
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Eden, Seat of the Sanctum", Zone::Hand);
    let instant = put(&mut game, player, "Shock", Zone::Library);
    let permanent = put(&mut game, player, "Grizzly Bears", Zone::Library);
    act(&mut game, Action::PlayLand { card: land });
    let mut tap = game.clone();
    tap.state.players[player.index()].mana = [0; 6];
    let colorless = [0, 0, 0, 0, 0, 1];
    act(&mut tap, mana(land, colorless));
    assert_eq!(tap.state.players[player.index()].mana, colorless);

    act(&mut game, announce_ability(land, 1));
    pay(&mut game);
    let sacrifice = Action::ChooseOptional { accept: true };
    reach(&mut game, |a| *a == sacrifice);
    assert_eq!(game.object_view(permanent).zone, Zone::Graveyard);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    act(&mut game, sacrifice);
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    reject(&mut game, target(land));
    reject(&mut game, target(instant));
    act(&mut game, target(permanent));
    settle(&mut game);
    assert_eq!(game.object_view(permanent).zone, Zone::Hand);
}

#[test]
fn elixir_enters_tapped_exiles_as_cost_and_gains_life_for_nonlands_only() {
    let player = Player::P0;
    let mut game = scenario(player);
    let artifact = put(&mut game, player, "Elixir", Zone::Hand);
    let spell = put(&mut game, player, "Shock", Zone::Graveyard);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let land = put(&mut game, player, "Forest", Zone::Graveyard);
    let life = game.state.players[player.index()].life;
    cast(&mut game, artifact, None);
    settle(&mut game);
    assert!(game.object_view(artifact).tapped);
    reject(&mut game, announce_ability(artifact, 0));
    game.state.objects[artifact as usize].tapped = false;
    act(&mut game, announce_ability(artifact, 0));
    pay(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Exile);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Library);
    assert_eq!(game.object_view(creature).zone, Zone::Library);
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, life + 2);
}

#[test]
fn ether_funds_the_next_spell_and_its_copy_can_have_a_different_target() {
    let player = Player::P0;
    let mut game = scenario(player);
    let artifact = put(&mut game, player, "Ether", Zone::Hand);
    let first = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let second = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, player, "Unsummon", Zone::Hand);
    cast(&mut game, artifact, None);
    settle(&mut game);
    game.state.players[player.index()].mana = [0; 6];

    let blue = [0, 1, 0, 0, 0, 0];
    act(&mut game, mana(artifact, blue));
    assert_eq!(game.object_view(artifact).zone, Zone::Exile);
    assert_eq!(game.state.players[player.index()].mana, blue);
    assert!(game.state.stack.is_empty());
    cast(&mut game, spell, Some(Target::Card(first)));
    let retarget = Action::SetStackTarget {
        index: 0,
        target: Target::Card(second),
    };
    reach(&mut game, |a| *a == retarget);
    act(&mut game, retarget);
    act(&mut game, Action::FinishStackTargets);
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Hand);
    assert_eq!(game.object_view(second).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 2);
}

#[test]
fn evil_reawakened_adds_two_counters_to_the_creatures_normal_entry() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Evil Reawakened", Zone::Hand);
    let creature = put(&mut game, player, "District Mascot", Zone::Graveyard);
    let foreign = put(&mut game, player.other(), "Grizzly Bears", Zone::Graveyard);

    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, target(foreign));
    act(&mut game, target(creature));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    assert!(!game.object_view(creature).tapped);
    assert_eq!(game.object_view(creature).counters.plus_one, 3);
}
