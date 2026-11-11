//! Printed class levels, typed attachment and per-event Supply counters.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn activate(game: &mut Game, source: CardId, slot: u16) {
    choose(game, |action| match *action {
        Action::Activate { card, ability, .. } | Action::AnnounceActivate { card, ability, .. } => {
            card == source && ability == slot
        }
        _ => false,
    });
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
    settle(game);
}

fn level(game: &mut Game, source: CardId, value: u8) {
    activate(game, source, u16::from(value - 2));
    assert_eq!(game.object_view(source).class_level, Some(value));
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn step(game: &mut Game, at: Step) {
    game.begin_step(at);
    game.prepare_triggers();
}

#[test]
fn artists_talent_loots_reduces_noncreature_costs_and_adds_noncombat_damage() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let talent = put(&mut game, player, "Artist's Talent", Zone::Battlefield);
    let discarded = put(&mut game, player, "Forest", Zone::Hand);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(opponent)));
    pass_pair(&mut game);
    act(&mut game, Action::Discard { card: discarded });
    settle(&mut game);
    assert_eq!(game.object_view(discarded).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(game.state.players[opponent.index()].life, 18);

    level(&mut game, talent, 2);
    let lash = put(&mut game, player, "Flame Lash", Zone::Hand);
    let base = ManaCost {
        colored: [0, 0, 0, 1, 0, 0],
        generic: 3,
    };
    let total = game
        .spell_total_for_definition(player, lash, base, game.object_view(lash).card)
        .unwrap();
    assert_eq!(total.generic, 2);

    level(&mut game, talent, 3);
    cast(&mut game, lash, Some(Target::Player(opponent)));
    pass_pair(&mut game);
    act(&mut game, Action::DeclineEffectPayment);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(game.state.players[opponent.index()].life, 12);
}

#[test]
fn blacksmiths_talent_creates_and_attaches_sword_and_grants_own_turn_keywords() {
    let player = Player::P1;
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let talent = put(&mut game, player, "Blacksmith's Talent", Zone::Hand);
    cast(&mut game, talent, None);
    settle(&mut game);
    let sword = game
        .observe(player)
        .objects
        .into_iter()
        .find(|object| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.is_token()
                && object.has_subtype("Equipment")
        })
        .unwrap();
    assert_eq!(sword.card, Card::from_catalogue_id(5620));
    assert_eq!(sword.attached_to, None);
    level(&mut game, talent, 2);
    level(&mut game, talent, 3);

    step(&mut game, Step::BeginCombat);
    act(&mut game, target(sword.id));
    let mut declined = game.clone();
    act(&mut declined, Action::FinishTargets);
    settle(&mut declined);
    assert_eq!(declined.object_view(sword.id).attached_to, None);

    act(&mut game, target(creature));
    settle(&mut game);
    assert_eq!(
        game.object_view(sword.id).attached_to,
        Some(game.object_ref(creature))
    );
    assert_eq!(stats(&game, creature), (3, 3));
    assert!(
        game.keywords(creature)
            .contains(Keywords::DOUBLE_STRIKE.union(Keywords::HASTE))
    );
    game.finish_cleanup();
    assert_eq!(stats(&game, creature), (3, 3));
    assert!(
        !game
            .keywords(creature)
            .intersects(Keywords::DOUBLE_STRIKE.union(Keywords::HASTE))
    );
}

#[test]
fn innkeepers_talent_doubles_combat_and_pantry_counters_after_granting_ward() {
    let player = Player::P0;
    let mut game = scenario(player);
    let talent = put(&mut game, player, "Innkeeper's Talent", Zone::Battlefield);
    let pantry = put(&mut game, player, "Stocking the Pantry", Zone::Battlefield);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let artifact = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    game.add_counter(artifact, CounterKind::Charge, 1);
    assert!(
        game.characteristic_query()
            .static_ward_grants(artifact)
            .is_empty()
    );
    level(&mut game, talent, 2);
    assert_eq!(
        game.characteristic_query()
            .static_ward_grants(artifact)
            .len(),
        1
    );
    assert!(
        game.characteristic_query()
            .static_ward_grants(creature)
            .is_empty()
    );

    level(&mut game, talent, 3);
    step(&mut game, Step::BeginCombat);
    act(&mut game, target(creature));
    settle(&mut game);
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(pantry, CounterKind::Supply), 2);
    assert_eq!(
        game.characteristic_query()
            .static_ward_grants(creature)
            .len(),
        1
    );
    activate(&mut game, pantry, 0);
    assert_eq!(game.counter_count(pantry, CounterKind::Supply), 1);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn hunters_talent_damages_on_entry_and_checks_attack_and_end_step_levels() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let giant = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let bears = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let talent = put(&mut game, player, "Hunter's Talent", Zone::Hand);
    cast(&mut game, talent, None);
    pass_pair(&mut game);
    act(&mut game, target(giant));
    act(&mut game, target(bears));
    settle(&mut game);
    assert_eq!(game.object_view(bears).zone, Zone::Graveyard);
    assert_eq!(game.object_view(giant).damage, 0);
    level(&mut game, talent, 2);
    level(&mut game, talent, 3);

    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    game.state.priority = player;
    act(&mut game, Action::Attack { card: giant });
    act(&mut game, Action::FinishAttackers);
    act(&mut game, target(giant));
    settle(&mut game);
    assert_eq!(stats(&game, giant), (4, 3));
    assert!(game.keywords(giant).contains(Keywords::TRAMPLE));

    step(&mut game, Step::End);
    let mut weakened = game.clone();
    weakened.state.players[player.index()].mana = [8; 6];
    let savor = put(&mut weakened, player, "Savor", Zone::Hand);
    cast(&mut weakened, savor, Some(Target::Card(giant)));
    settle(&mut weakened);
    assert_eq!(
        weakened.state.players[player.index()].cards_drawn_this_turn,
        0
    );

    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}
