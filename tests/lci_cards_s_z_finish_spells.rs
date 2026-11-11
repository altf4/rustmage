//! Cast modes, death batches, paid discard and sacrifice ordinals.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn candidate(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}

fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn target(target: Target) -> Action {
    Action::ChooseTarget { target }
}

fn advance(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    for _ in 0..24 {
        if stop(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("expected a resolution decision");
}

fn token_count(g: &Game, p: Player, recipe: Card) -> usize {
    g.observe(p)
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.controller == p && o.card == recipe)
        .count()
}

#[test]
fn trumpeting_carnosaur_discovers_five_and_can_be_discarded_to_damage_a_creature() {
    let p = Player::P0;
    let mut game = scenario(p);
    let carnosaur = candidate(&mut game, p, "Trumpeting Carnosaur", Zone::Hand);
    let victim = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let hit = put(&mut game, p, "Hill Giant", Zone::Library);
    let mut discard = game.clone();
    act(&mut discard, activate(carnosaur));
    reject(&mut discard, target(Target::Player(p.other())));
    act(&mut discard, target(Target::Card(victim)));
    pay(&mut discard);
    assert_eq!(discard.object_view(carnosaur).zone, Zone::Graveyard);
    settle(&mut discard);
    assert_eq!(discard.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(discard.object_view(hit).zone, Zone::Library);

    cast(&mut game, carnosaur, None);
    advance(&mut game, |g| g.observe(p).cascade.is_some());
    assert_eq!(stats(&game, carnosaur), (7, 6));
    assert!(game.keywords(carnosaur).contains(Keywords::TRAMPLE));
    assert_eq!(
        game.observe(p.other()).cascade.unwrap().discovered,
        Some(game.object_ref(hit))
    );
    act(&mut game, Action::FinishResolutionCasts);
    assert_eq!(game.object_view(hit).zone, Zone::Hand);
}

#[test]
fn vito_counts_resolutions_of_separate_sacrifices_and_creates_the_printed_demon() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = candidate(&mut game, p, "Vito, Fanatic of Aclazotz", Zone::Battlefield);
    game.create_tokens(p, Card(5302), 4, false);
    for i in 1..=4 {
        choose(
            &mut game,
            |a| matches!(a, Action::ActivateMana { card, .. } if *card != source),
        );
        settle(&mut game);
        assert_eq!(game.state.players[p.index()].life, STARTING_LIFE + 2);
        assert_eq!(
            game.state.players[p.other().index()].life,
            STARTING_LIFE - if i >= 2 { 2 } else { 0 }
        );
        assert_eq!(token_count(&game, p, Card(5578)), usize::from(i >= 3));
    }
    assert!(game.keywords(source).contains(Keywords::FLYING));
}

#[test]
fn thunderhulk_enters_with_counters_and_attack_samples_departed_power() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = candidate(&mut game, p, "Threefold Thunderhulk", Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 3);
    assert_eq!(token_count(&game, p, Card(5267)), 3);
    let gnome = game
        .observe(p)
        .objects
        .into_iter()
        .find(|o| o.card == Card(5267))
        .unwrap()
        .reference;
    act(&mut game, activate(source));
    let own = game.object_ref(source);
    reject(&mut game, Action::ChooseAdditionalCost { object: own });
    act(&mut game, Action::ChooseAdditionalCost { object: gnome });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, source), (4, 4));
    assert_eq!(token_count(&game, p, Card(5267)), 2);
    game.state.objects[source as usize].summoning_sick = false;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: source });
    act(&mut game, Action::FinishAttackers);
    game.move_object(source, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(token_count(&game, p, Card(5267)), 6);
}

#[test]
fn skullspore_nexus_reduces_only_generic_and_sizes_one_token_from_a_death_batch() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = candidate(&mut game, p, "The Skullspore Nexus", Zone::Hand);
    let bear = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let giant = put(&mut game, p, "Hill Giant", Zone::Battlefield);
    game.add_counter(giant, CounterKind::PlusOne, 3);
    game.create_tokens(p, Card(5327), 1, false);
    game.state.players[p.index()].mana = [0, 0, 0, 0, 0, 8];
    reject(&mut game, Action::AnnounceCast { card: source });
    game.state.players[p.index()].mana = [0, 0, 0, 0, 2, 0];
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].mana, [0; 6]);

    game.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
    act(&mut game, activate(source));
    act(&mut game, target(Target::Card(bear)));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, bear), (4, 2));
    assert!(game.object_view(source).tapped);

    let wipe = put(&mut game, p, "Day of Judgment", Zone::Hand);
    game.state.players[p.index()].mana = [2, 0, 0, 0, 0, 2];
    cast(&mut game, wipe, None);
    settle(&mut game);
    let o = game
        .observe(p)
        .objects
        .into_iter()
        .find(|o| o.card == Card(5572))
        .unwrap();
    assert_eq!(token_count(&game, p, Card(5572)), 1);
    assert_eq!(stats(&game, o.id), (10, 10));
}

#[test]
fn starving_revenant_draws_for_each_surveillance_keep_and_drains_for_each_draw() {
    let p = Player::P0;
    let mut game = scenario(p);
    let card = candidate(&mut game, p, "Starving Revenant", Zone::Hand);
    for _ in 0..7 {
        put(&mut game, p, "Forest", Zone::Graveyard);
    }
    let permanent = put(&mut game, p, "Island", Zone::Library);
    let spell = put(&mut game, p, "Opt", Zone::Library);
    cast(&mut game, card, None);
    advance(&mut game, |g| g.observe(p).library_choice.is_some());
    assert!(game.observe(p.other()).library_choice.is_none());
    let mut all_milled = game.clone();
    for id in [permanent, spell] {
        act(
            &mut all_milled,
            Action::ArrangeLibrary {
                card: id,
                destination: LibraryDestination::Graveyard,
            },
        );
    }
    settle(&mut all_milled);
    assert_eq!(all_milled.state.players[p.index()].cards_drawn_this_turn, 0);
    assert_eq!(all_milled.state.players[p.index()].life, STARTING_LIFE);

    act(
        &mut game,
        Action::ArrangeLibrary {
            card: permanent,
            destination: LibraryDestination::Graveyard,
        },
    );
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: spell,
            destination: LibraryDestination::Top,
        },
    );
    advance(&mut game, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    assert_eq!(game.object_view(spell).zone, Zone::Hand);
    assert_eq!(game.state.players[p.index()].life, STARTING_LIFE - 3);
    reject(&mut game, target(Target::Player(p)));
    act(&mut game, target(Target::Player(p.other())));
    let mut below = game.clone();
    below.move_object(permanent, Zone::Exile);
    settle(&mut below);
    assert_eq!(below.state.players[p.other().index()].life, STARTING_LIFE);
    settle(&mut game);
    assert_eq!(
        game.state.players[p.other().index()].life,
        STARTING_LIFE - 1
    );
    assert_eq!(game.state.players[p.index()].life, STARTING_LIFE - 2);
}

#[test]
fn wail_only_offers_combined_modes_at_eight_and_keeps_their_printed_order() {
    let p = Player::P1;
    let other = p.other();
    let mut game = scenario(p);
    let wail = candidate(&mut game, p, "Wail of the Forgotten", Zone::Hand);
    let permanent = put(&mut game, other, "Bear Cub", Zone::Battlefield);
    let first = put(&mut game, p, "Opt", Zone::Library);
    let second = put(&mut game, p, "Shock", Zone::Library);
    let third = put(&mut game, p, "Island", Zone::Library);
    for _ in 0..7 {
        put(&mut game, p, "Forest", Zone::Graveyard);
    }
    let modes = |g: &Game| {
        g.legal_actions()
            .iter()
            .filter(|a| matches!(a, Action::ChooseMode { .. }))
            .count()
    };
    let mut below = game.clone();
    act(&mut below, Action::AnnounceCast { card: wail });
    assert_eq!(modes(&below), 3);
    reject(&mut below, Action::ChooseMode { mode: 6 });

    let eighth = put(&mut game, p, "Short Bow", Zone::Graveyard);
    act(&mut game, Action::AnnounceCast { card: wail });
    assert_eq!(modes(&game), 7);
    act(&mut game, Action::ChooseMode { mode: 6 });
    act(&mut game, target(Target::Card(permanent)));
    act(&mut game, target(Target::Player(other)));
    pay(&mut game);
    game.move_object(eighth, Zone::Exile);
    advance(&mut game, |g| {
        matches!(g.state.pending, Pending::EffectDiscard { .. })
    });
    assert_eq!(game.object_view(permanent).zone, Zone::Hand);
    act(&mut game, Action::Discard { card: permanent });
    assert_eq!(game.object_view(permanent).zone, Zone::Graveyard);
    let choice = game.observe(p).library_selection.unwrap();
    assert_eq!(choice.remaining.len(), 3);
    assert!(game.observe(other).library_selection.is_none());
    act(&mut game, Action::SelectLibrary { card: second });
    act(&mut game, Action::FinishLibrarySelection);
    settle(&mut game);
    assert_eq!(game.object_view(second).zone, Zone::Hand);
    for id in [first, third] {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
}
