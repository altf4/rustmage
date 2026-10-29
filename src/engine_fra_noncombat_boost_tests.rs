//! Replacement effects apply to individual controlled-source damage events.
use super::*;

#[test]
fn tomik_adds_one_to_own_noncombat_spell_and_has_prowess() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let tomik = put(&mut game, player, "Tomik, Izzet Sparkmage", Zone::Battlefield);
        let spell = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, spell, Some(Target::Player(player.other())));
        settle(&mut game);
        assert_eq!(game.state.players[player.other().index()].life, 17);
        assert_eq!((game.stats(tomik).power, game.stats(tomik).toughness), (2, 3));
        super::super::contracts::branch_contract(game);
    }
}

#[test]
fn tomik_damage_filter_handles_combat_controller_and_recipient() {
    for player in [Player::P0, Player::P1] {
        for (source_owner, target_owner, combat) in [
            (player, player.other(), false),
            (player, player, false),
            (player.other(), player.other(), false),
            (player, player.other(), true),
        ] {
            let mut game = scenario(player);
            put(&mut game, player, "Tomik, Izzet Sparkmage", Zone::Battlefield);
            let source = put(&mut game, source_owner, "Bear Cub", Zone::Battlefield);
            let event = DamageEvent {
                preventable: true,
                source: game.damage_source(source),
                target: Target::Player(target_owner),
                amount: 1,
                combat,
            };
            game.apply_damage_batch(&[event]);
            game.advance_replacements(false);
            let expected = if source_owner == player && target_owner != player && !combat { 2 } else { 1 };
            assert_eq!(game.state.players[target_owner.index()].life, 20 - expected);
        }
    }
}

#[test]
fn tomik_uses_damage_source_controller_after_source_departure_and_live_recipient_controller() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        put(&mut game, player, "Tomik, Izzet Sparkmage", Zone::Battlefield);
        let source = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let recipient = put(&mut game, player.other(), "Monoist Sentry", Zone::Battlefield);
        game.state.objects[recipient as usize].growth.toughness = 7;
        let facts = game.damage_source(source);
        game.move_object(source, Zone::Graveyard);
        let event = DamageEvent { preventable: true, source: facts, target: Target::Card(recipient), amount: 1, combat: false };
        game.apply_damage_batch(&[event]);
        game.advance_replacements(false);
        assert_eq!(game.state.objects[recipient as usize].damage, 2);
        game.change_controller(game.object_ref(recipient), player);
        game.apply_damage_batch(&[event]);
        game.advance_replacements(false);
        assert_eq!(game.state.objects[recipient as usize].damage, 3);
    }
}

#[test]
fn tomik_and_damage_doubler_apply_in_affected_players_chosen_order() {
    for player in [Player::P0, Player::P1] {
        for boost_first in [false, true] {
            let mut game = scenario(player);
            let tomik = put(&mut game, player, "Tomik, Izzet Sparkmage", Zone::Battlefield);
            let tyrant = put(&mut game, player, "Twinflame Tyrant", Zone::Battlefield);
            let source = put(&mut game, player, "Bear Cub", Zone::Battlefield);
            let event = DamageEvent {
                preventable: true,
                source: game.damage_source(source),
                target: Target::Player(player.other()),
                amount: 2,
                combat: false,
            };
            game.apply_damage_batch(&[event]);
            game.advance_replacements(false);
            assert_eq!(game.actor(), Some(player.other()));
            let selected = game.object_ref(if boost_first { tomik } else { tyrant });
            assert!(game.legal_actions().contains(&Action::ChooseReplacement { source: selected, ability: 0 }));
            super::super::contracts::branch_contract(game.clone());
            act(&mut game, Action::ChooseReplacement { source: selected, ability: 0 });
            assert_eq!(game.state.players[player.other().index()].life, if boost_first { 14 } else { 15 });
        }
    }
}
