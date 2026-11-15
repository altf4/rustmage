//! Star Trek A–D token, Aura, draw and characteristic scenarios.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn owned(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}
fn drain(game: &mut Game) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("resolution budget");
}
fn tokens(game: &Game, card: Card) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == Player::P0
                && o.is_token()
                && o.card == card)
                .then_some(id as CardId)
        })
        .collect()
}

#[test]
fn boarding_action_and_investigator_make_printed_haste_klingons() {
    let mut game = scenario(Player::P0);
    let boarding = owned(&mut game, Player::P0, "Boarding Action", Zone::Hand);
    cast(&mut game, boarding, None);
    drain(&mut game);
    assert_eq!(tokens(&game, Card(6010)).len(), 1);
    assert_eq!(stats(&game, tokens(&game, Card(6010))[0]), (2, 1));
    assert!(
        game.keywords(tokens(&game, Card(6010))[0])
            .contains(Keywords::HASTE)
    );
    let investigator = owned(&mut game, Player::P0, "Cha'DIch Investigator", Zone::Hand);
    cast(&mut game, investigator, None);
    drain(&mut game);
    assert_eq!(stats(&game, investigator), (4, 4));
    assert_eq!(tokens(&game, Card(6010)).len(), 2);
}

#[test]
fn boarding_action_beam_bounces_a_creature_and_exiles_the_spell() {
    let mut game = scenario(Player::P0);
    let returner = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let spell = owned(&mut game, Player::P0, "Boarding Action", Zone::Graveyard);
    let action = Action::AnnounceBeamMeUp {
        card: spell,
        face: CastFace::Normal,
        cost: ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 2,
        },
        flash_surcharge: false,
        provider: None,
    };
    assert!(game.legal_actions().contains(&action));
    act(&mut game, action);
    act(&mut game, Action::BeginOrderedCosts);
    let mana = game
        .observe(Player::P0)
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| !row.paid && row.clause == CastCostClause::Mana)
        .unwrap() as u16;
    act(&mut game, Action::ChooseCastCost { clause: mana });
    pay(&mut game);
    let bounce = game
        .observe(Player::P0)
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| !row.paid && matches!(row.clause, CastCostClause::BeamMeUpReturn(_, _)))
        .unwrap() as u16;
    act(&mut game, Action::ChooseCastCost { clause: bounce });
    let exact = game.object_ref(returner);
    act(&mut game, Action::PayBeamMeUpReturn { object: exact });
    drain(&mut game);
    assert_eq!(game.object_view(returner).zone, Zone::Hand);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    assert_eq!(tokens(&game, Card(6010)).len(), 1);
}

#[test]
fn cloistered_telepath_draws_then_requires_an_actual_discard() {
    let mut game = scenario(Player::P0);
    let telepath = owned(&mut game, Player::P0, "Cloistered Telepath", Zone::Hand);
    assert!(
        Card::from_name("Cloistered Telepath")
            .unwrap()
            .keywords()
            .contains(Keywords::FLASH)
    );
    let previous = put(&mut game, Player::P0, "Grizzly Bears", Zone::Hand);
    let draws = game.state.players[0].cards_drawn_this_turn;
    cast(&mut game, telepath, None);
    for _ in 0..16 {
        if matches!(game.state.pending, Pending::EffectDiscard { .. }) {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.state.players[0].cards_drawn_this_turn, draws + 1);
    assert_eq!(stats(&game, telepath), (2, 1));
    act(&mut game, Action::Discard { card: previous });
    drain(&mut game);
    assert_eq!(game.object_view(previous).zone, Zone::Graveyard);
}

#[test]
fn collective_drone_power_counts_only_controlled_artifact_creatures() {
    let mut game = scenario(Player::P0);
    let drone = owned(&mut game, Player::P0, "Collective Drone", Zone::Battlefield);
    assert_eq!(stats(&game, drone), (1, 2));
    let borg = owned(
        &mut game,
        Player::P0,
        "Assembly Specialist",
        Zone::Battlefield,
    );
    assert_eq!(stats(&game, drone), (2, 2));
    let ordinary = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let _enemy = owned(
        &mut game,
        Player::P1,
        "Assembly Specialist",
        Zone::Battlefield,
    );
    assert_eq!(stats(&game, drone), (2, 2));
    game.move_object(borg, Zone::Graveyard);
    assert_eq!(stats(&game, drone), (1, 2));
    assert_eq!(stats(&game, ordinary), (2, 2));
}

#[test]
fn captains_tutelage_grants_entry_first_strike_and_lasting_power() {
    let mut game = scenario(Player::P0);
    let bearer = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let aura = owned(&mut game, Player::P0, "Captain's Tutelage", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(bearer)));
    drain(&mut game);
    assert_eq!(game.object_view(aura).zone, Zone::Battlefield);
    assert_eq!(stats(&game, bearer), (4, 2));
    assert!(game.keywords(bearer).contains(Keywords::FIRST_STRIKE));
    game.move_object(aura, Zone::Graveyard);
    assert_eq!(stats(&game, bearer), (2, 2));
}

#[test]
fn david_and_carol_create_one_tapped_new_planet() {
    let mut game = scenario(Player::P0);
    let source = owned(
        &mut game,
        Player::P0,
        "David and Carol Marcus, Creators",
        Zone::Hand,
    );
    cast(&mut game, source, None);
    drain(&mut game);
    let created = tokens(&game, Card(6009));
    assert_eq!(created.len(), 1);
    assert!(game.object_view(created[0]).tapped);
    assert_eq!(stats(&game, source), (1, 3));
}
