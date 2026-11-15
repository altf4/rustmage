//! Full E–M card effects through legal actions and exact owned state.
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
        choose(game, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("resolution budget");
}

#[test]
fn lore_entry_modes_target_boost_evasion_or_surveil_and_expire() {
    for mode in [0, 1] {
        let player = Player::P0;
        let mut game = scenario(player);
        let target = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let top = put(&mut game, player, "Forest", Zone::Library);
        let lore = owned(&mut game, player, "Lore, Positronic Provocateur", Zone::Hand);
        cast(&mut game, lore, None);
        for _ in 0..12 {
            if game.legal_actions().contains(&Action::ChooseMode { mode }) { break; }
            choose(&mut game, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
        }
        reject(&mut game, Action::ChooseMode { mode: 9 });
        act(&mut game, Action::ChooseMode { mode });
        if mode == 0 {
            act(&mut game, Action::ChooseTarget { target: Target::Card(target) });
            drain(&mut game);
            assert_eq!(stats(&game, target), (3, 2));
            assert!(game.keywords(target).contains(Keywords::CANT_BE_BLOCKED));
            game.state.step = Step::Cleanup;
            game.finish_cleanup();
            assert_eq!(stats(&game, target), (2, 2));
            assert!(!game.keywords(target).contains(Keywords::CANT_BE_BLOCKED));
        } else {
            for _ in 0..8 {
                if game.observe(player).library_choice.is_some() { break; }
                choose(&mut game, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
            }
            assert!(game.observe(player).library_choice.unwrap().surveil);
            assert_eq!(game.observe(player).library_choice.as_ref().unwrap().remaining.len(), 2);
            act(&mut game, Action::ArrangeLibrary { card: top, destination: LibraryDestination::Graveyard });
            let remaining = game.observe(player).library_choice.as_ref().unwrap().remaining[0].id;
            act(&mut game, Action::ArrangeLibrary { card: remaining, destination: LibraryDestination::Top });
            drain(&mut game);
            assert_eq!(game.object_view(top).zone, Zone::Graveyard);
            assert_eq!(stats(&game, target), (2, 2));
        }
    }
}

#[test]
fn holodeck_links_exile_to_exact_incarnation_and_ward_counters_unpaid_spell() {
    let player = Player::P0;
    let mut game = scenario(player);
    let victim = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    let holodeck = owned(&mut game, player, "Malfunctioning Holodeck", Zone::Hand);
    cast(&mut game, holodeck, None);
    for _ in 0..12 {
        if game.legal_actions().contains(&Action::ChooseTarget { target: Target::Card(victim) }) { break; }
        choose(&mut game, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    reject(&mut game, Action::ChooseTarget { target: Target::Card(land) });
    act(&mut game, Action::ChooseTarget { target: Target::Card(victim) });
    drain(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Exile);
    let active = game.snapshot();
    let mut branch = game.clone();
    branch.move_object(holodeck, Zone::Graveyard);
    assert_eq!(branch.object_view(victim).zone, Zone::Battlefield);
    branch.restore(&active).unwrap();
    assert_eq!(branch.snapshot(), active);

    game.state.active = player.other();
    game.state.priority = player.other();
    game.state.players[player.other().index()].mana = [0; 6];
    let spell = put(&mut game, player.other(), "Disenchant", Zone::Hand);
    game.state.players[player.other().index()].mana = [1, 0, 0, 0, 0, 1];
    cast(&mut game, spell, Some(Target::Card(holodeck)));
    assert_eq!(game.state.stack.len(), 2);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    act(&mut game, Action::DeclineResolutionMana);
    drain(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(game.object_view(holodeck).zone, Zone::Battlefield);
    assert_eq!(game.object_view(victim).zone, Zone::Exile);
}
