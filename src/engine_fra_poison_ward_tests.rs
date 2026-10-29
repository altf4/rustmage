use super::*;

#[test]
fn serpent_ward_uses_targeting_player_cost_and_keeps_choice_transactional() {
    for owner in [Player::P0, Player::P1] {
        let mut game = scenario(owner);
        let society = put(&mut game, owner, "The Serpent Society", Zone::Battlefield);
        let attacker = owner.other();
        let spell = put(&mut game, attacker, "Unsummon", Zone::Hand);
        game.state.priority = attacker;
        game.state.players[attacker.index()].mana = [0, 1, 0, 0, 0, 0];
        act(&mut game, Action::Cast {
            card: spell, target: Some(Target::Card(society)), payment: [0, 1, 0, 0, 0, 0],
        });
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert!(matches!(game.state.pending, Pending::EffectOptional { player, .. } if player == attacker));
        let saved = game.snapshot();
        for pay_cost in [false, true] {
            game.restore(&saved).unwrap();
            let action = Action::ChooseOptional { accept: pay_cost };
            assert!(game.apply(owner, action.clone()).is_err());
            assert_eq!(game.snapshot(), saved);
            act(&mut game, action);
            settle(&mut game);
            assert_eq!(game.state.players[attacker.index()].poison_counters, if pay_cost { 5 } else { 0 });
            assert_eq!(game.state.players[owner.index()].poison_counters, 0);
            assert_eq!(game.object_view(society).zone, if pay_cost { Zone::Hand } else { Zone::Battlefield });
        }
    }
}

#[test]
fn serpent_ward_still_offers_payment_after_related_spell_leaves() {
    for owner in [Player::P0, Player::P1] {
        let mut game = scenario(owner);
        let society = put(&mut game, owner, "The Serpent Society", Zone::Battlefield);
        let attacker = owner.other();
        let spell = put(&mut game, attacker, "Unsummon", Zone::Hand);
        game.state.priority = attacker;
        game.state.players[attacker.index()].mana = [0, 1, 0, 0, 0, 0];
        act(&mut game, Action::Cast {
            card: spell, target: Some(Target::Card(society)), payment: [0, 1, 0, 0, 0, 0],
        });
        game.prepare_triggers();
        game.move_object(spell, Zone::Graveyard);
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert!(matches!(game.state.pending, Pending::EffectOptional { player, .. } if player == attacker));
        let saved = game.snapshot();
        for pay_cost in [false, true] {
            game.restore(&saved).unwrap();
            act(&mut game, Action::ChooseOptional { accept: pay_cost });
            settle(&mut game);
            assert_eq!(game.state.players[attacker.index()].poison_counters, if pay_cost { 5 } else { 0 });
            assert_eq!(game.object_view(society).zone, Zone::Battlefield);
        }
    }
}
