//! Complete T–Z Star Trek definitions exercised through the public card catalogue.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn drain(game: &mut Game, target: Option<CardId>) {
    for _ in 0..64 {
        let actions = game.legal_actions();
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.resolving.is_none()
            && game.state.pending_triggers.is_empty()
        {
            return;
        }
        if let Some(target) = target
            && actions.contains(&Action::ChooseTarget {
                target: Target::Card(target),
            })
        {
            act(game, Action::ChooseTarget { target: Target::Card(target) });
        } else if actions.contains(&Action::FinishStackTargets) {
            act(game, Action::FinishStackTargets);
        } else if actions.contains(&Action::ChooseOptional { accept: true }) {
            act(game, Action::ChooseOptional { accept: true });
        } else if let Some(action) = actions.iter().find(|action| {
            matches!(action, Action::ArrangeLibrary { destination: LibraryDestination::Top, .. })
        }) {
            act(game, action.clone());
        } else if actions.contains(&Action::FinishSearch) {
            act(game, Action::FinishSearch);
        } else if let Some(action) = actions.into_iter().find(|action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        }) {
            act(game, action);
        } else {
            panic!("unhandled stack choice: {:?}", game.state.pending);
        }
    }
    panic!("stack resolution exceeded 64 actions");
}

#[test]
fn analyst_and_xenobotanist_use_real_tap_abilities() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let analyst = put(&mut game, player, "Tal Shiar Analyst", Zone::Battlefield);
        let botanist = put(&mut game, player, "Xenobotanist", Zone::Battlefield);
        for source in [analyst, botanist] {
            game.state.objects[source as usize].summoning_sick = false;
        }
        assert_eq!(game.object_view(analyst).power, 3);
        let life = game.state.players[player.other().index()].life;
        act(&mut game, Action::AnnounceActivate {
            card: analyst,
            ability: 0,
            counter_payment: Counters::default(),
        });
        pay(&mut game);
        drain(&mut game, None);
        assert!(game.state.objects[analyst as usize].tapped);
        assert_eq!(game.state.players[player.other().index()].life, life - 1);

        let spell = put(&mut game, player, "Giant Growth", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(analyst)));
        drain(&mut game, None);
        assert!(!game.state.objects[analyst as usize].tapped);

        let before = game.state.players[player.index()].mana;
        let mana = game.legal_actions().into_iter().find(|action| {
            matches!(action, Action::ActivateMana { card, .. } if *card == botanist)
        }).expect("Xenobotanist mana ability");
        act(&mut game, mana);
        assert!(game.state.objects[botanist as usize].tapped);
        assert!(game.state.players[player.index()].mana.iter().zip(before).any(|(after, before)| *after > before));
    }
}

#[test]
fn xindi_landfall_and_vulcan_entry_are_owner_scoped() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let xindi = put(&mut game, player, "Xindi Surveyors", Zone::Battlefield);
        let life = game.state.players[player.index()].life;
        let vulcan = put(&mut game, player, "Vulcan", Zone::Hand);
        act(&mut game, Action::PlayLand { card: vulcan });
        drain(&mut game, None);
        assert!(game.state.objects[vulcan as usize].tapped);
        assert_eq!(game.state.players[player.index()].life, life + 1);
        assert_eq!(game.counter_count(xindi, CounterKind::PlusOne), 1);
        let mut opponent_turn = scenario(player.other());
        let xindi = put(&mut opponent_turn, player, "Xindi Surveyors", Zone::Battlefield);
        let opposing = put(&mut opponent_turn, player.other(), "Forest", Zone::Hand);
        act(&mut opponent_turn, Action::PlayLand { card: opposing });
        drain(&mut opponent_turn, None);
        assert_eq!(opponent_turn.counter_count(xindi, CounterKind::PlusOne), 0);
    }
}

#[test]
fn tactical_officer_uses_total_power_at_attack_declaration() {
    let mut game = scenario(Player::P0);
    let officer = put(&mut game, Player::P0, "Tactical Officer", Zone::Battlefield);
    put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    game.state.objects[officer as usize].summoning_sick = false;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: officer });
    act(&mut game, Action::FinishAttackers);
    while !game.state.stack.is_empty() || !game.state.pending_triggers.is_empty() {
        choose(&mut game, |action| matches!(action, Action::Pass | Action::PutTrigger { .. }));
    }
    assert_eq!(game.counter_count(officer, CounterKind::PlusOne), 1);
    assert!(!game.state.objects[officer as usize].tapped);
}

#[test]
fn vexing_drone_taps_only_an_opposing_creature() {
    let mut game = scenario(Player::P0);
    let target = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let drone = put(&mut game, Player::P0, "Vexing Drone", Zone::Hand);
    cast(&mut game, drone, None);
    drain(&mut game, Some(target));
    assert!(game.state.objects[target as usize].tapped);
    assert_eq!(game.object_view(drone).zone, Zone::Battlefield);
}

#[test]
fn holodeck_graveyard_permission_is_once_each_turn_and_restores() {
    let mut game = scenario(Player::P1);
    let source = put(&mut game, Player::P1, "The Holodeck", Zone::Battlefield);
    let first = put(&mut game, Player::P1, "Forest", Zone::Graveyard);
    let second = put(&mut game, Player::P1, "Mountain", Zone::Graveyard);
    let before = game.snapshot();
    let play = |game: &Game, card| Action::PlaySourceGraveyardLand {
        card,
        source: game.object_ref(source),
        play_kind: SourcePlayKind::GraveyardOrOutsidePermanent,
    };
    assert!(game.legal_actions().contains(&play(&game, first)));
    let first_play = play(&game, first);
    act(&mut game, first_play);
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
    let second_play = play(&game, second);
    reject(&mut game, second_play);
    game.restore(&before).unwrap();
    assert!(game.legal_actions().contains(&play(&game, first)));
}

#[test]
fn traveler_counts_distinct_controlled_land_names() {
    let mut game = scenario(Player::P0);
    let traveler = put(&mut game, Player::P0, "The Traveler", Zone::Battlefield);
    assert_eq!(stats(&game, traveler), (0, 3));
    put(&mut game, Player::P0, "Forest", Zone::Battlefield);
    put(&mut game, Player::P0, "Forest", Zone::Battlefield);
    put(&mut game, Player::P1, "Mountain", Zone::Battlefield);
    assert_eq!(stats(&game, traveler), (1, 3));
    put(&mut game, Player::P0, "Island", Zone::Battlefield);
    assert_eq!(stats(&game, traveler), (2, 3));
    assert!(game.object_view(traveler).keywords.contains(Keywords::REACH.union(Keywords::VIGILANCE)));
}

#[test]
fn tribbles_duplicate_at_own_upkeep_and_tokens_keep_the_trigger() {
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Troubling Tribbles", Zone::Battlefield);
    game.begin_step(Step::Upkeep);
    game.advance_replacements(false);
    game.give_priority();
    drain(&mut game, None);
    let count = |g: &Game| g.state.objects.iter().filter(|o| o.zone == Zone::Battlefield && o.card == Card::from_name("Troubling Tribbles").unwrap()).count();
    assert_eq!(count(&game), 2);
    assert!(game.object_view(source).keywords.contains(Keywords::CANT_BLOCK));
    game.begin_step(Step::Upkeep);
    game.advance_replacements(false);
    game.give_priority();
    drain(&mut game, None);
    assert_eq!(count(&game), 4);
}

#[test]
fn territory_searches_a_basic_tapped_and_sacrifices_as_cost() {
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Unexplored Territory", Zone::Battlefield);
    let basic = put(&mut game, Player::P0, "Island", Zone::Library);
    act(&mut game, Action::AnnounceActivate { card: source, ability: 1, counter_payment: Counters::default() });
    pay(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    while !matches!(game.state.pending, Pending::EffectSearch { player: Player::P0 }) {
        choose(&mut game, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    act(&mut game, Action::ChooseSearch { card: basic });
    act(&mut game, Action::FinishSearch);
    drain(&mut game, None);
    assert_eq!(game.object_view(basic).zone, Zone::Battlefield);
    assert!(game.state.objects[basic as usize].tapped);
}

#[test]
fn warp_core_search_and_charge_counters_are_actual_actions() {
    let mut game = scenario(Player::P0);
    let basic = put(&mut game, Player::P0, "Island", Zone::Library);
    let spacecraft = put(&mut game, Player::P0, "Specimen Freighter", Zone::Battlefield);
    let core = put(&mut game, Player::P0, "Warp Core", Zone::Hand);
    cast(&mut game, core, None);
    while !matches!(game.state.pending, Pending::EffectSearch { player: Player::P0 }) {
        choose(&mut game, |a| matches!(a, Action::Pass | Action::PutTrigger { .. } | Action::ChooseOptional { accept: true }));
    }
    act(&mut game, Action::ChooseSearch { card: basic });
    act(&mut game, Action::FinishSearch);
    drain(&mut game, None);
    assert_eq!(game.object_view(basic).zone, Zone::Hand);
    act(&mut game, Action::AnnounceActivate { card: core, ability: 0, counter_payment: Counters::default() });
    act(&mut game, Action::ChooseTarget { target: Target::Card(spacecraft) });
    if game.legal_actions().contains(&Action::FinishTargets) { act(&mut game, Action::FinishTargets); }
    pay(&mut game);
    drain(&mut game, None);
    assert_eq!(game.counter_count(spacecraft, CounterKind::Charge), 2);
}

#[test]
fn tricorder_equips_and_surveils_when_its_creature_attacks() {
    let mut game = scenario(Player::P0);
    let bearer = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let tricorder = put(&mut game, Player::P0, "Tricorder", Zone::Battlefield);
    act(&mut game, Action::AnnounceActivate { card: tricorder, ability: 0, counter_payment: Counters::default() });
    act(&mut game, Action::ChooseTarget { target: Target::Card(bearer) });
    if game.legal_actions().contains(&Action::FinishTargets) { act(&mut game, Action::FinishTargets); }
    pay(&mut game);
    drain(&mut game, None);
    assert_eq!(game.state.objects[tricorder as usize].attached_to, Some(game.object_ref(bearer)));
    assert_eq!(stats(&game, bearer), (3, 3));
    game.state.objects[bearer as usize].summoning_sick = false;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: bearer });
    act(&mut game, Action::FinishAttackers);
    let before = game.state.players[Player::P0.index()].library.len();
    drain(&mut game, None);
    assert_eq!(game.state.players[Player::P0.index()].library.len(), before);
    assert!(game.state.players[Player::P0.index()].scried_or_surveilled_this_turn);
}

#[test]
fn vash_surveils_and_draws_when_a_card_leaves_her_graveyard() {
    let mut game = scenario(Player::P0);
    put(&mut game, Player::P0, "Vash, Freelance Archaeologist", Zone::Battlefield);
    let holodeck = put(&mut game, Player::P0, "The Holodeck", Zone::Battlefield);
    let land = put(&mut game, Player::P0, "Forest", Zone::Graveyard);
    game.begin_step(Step::Upkeep);
    game.advance_replacements(false);
    game.give_priority();
    drain(&mut game, None);
    assert!(game.state.players[Player::P0.index()].scried_or_surveilled_this_turn);
    game.state.step = Step::Main1;
    let before = game.state.objects.iter().filter(|o| o.owner == Player::P0 && o.zone == Zone::Hand).count();
    let play = Action::PlaySourceGraveyardLand { card: land, source: game.object_ref(holodeck), play_kind: SourcePlayKind::GraveyardOrOutsidePermanent };
    act(&mut game, play);
    drain(&mut game, None);
    assert_eq!(game.state.objects.iter().filter(|o| o.owner == Player::P0 && o.zone == Zone::Hand).count(), before + 1);
}
