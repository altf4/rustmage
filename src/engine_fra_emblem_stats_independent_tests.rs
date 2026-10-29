use super::*;
use crate::engine::ability_world::CharacteristicWorld;

#[test]
fn command_emblem_modifiers_stack_follow_control_and_restore() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let other = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        game.create_emblem(player, Card::test_index(5463));
        game.create_emblem(player, Card::test_index(5463));
        assert_eq!(stats(&game, own), (6, 6));
        assert_eq!(stats(&game, other), (3, 3));
        let saved = game.snapshot();
        for _ in 0..2 {
            game.change_controller(game.object_ref(other), player);
            assert_eq!(stats(&game, other), (7, 7));
            game.change_controller(game.object_ref(own), player.other());
            assert_eq!(stats(&game, own), (2, 2));
            game.restore(&saved).unwrap();
        }
    }
}

#[test]
fn command_emblem_modifiers_apply_in_projected_characteristic_world() {
    struct Projected<'a> { game: &'a Game, changed: CardId, object: Object }
    impl CharacteristicWorld for Projected<'_> {
        fn game(&self) -> &Game { self.game }
        fn object(&self, id: CardId) -> &Object {
            if id == self.changed { &self.object } else { &self.game.state.objects[id as usize] }
        }
        fn life(&self, p: Player) -> i128 { CharacteristicWorld::life(self.game, p) }
        fn gained_life(&self, p: Player) -> bool { CharacteristicWorld::gained_life(self.game, p) }
        fn attacking(&self, id: CardId) -> bool { CharacteristicWorld::attacking(self.game, id) }
    }
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let creature = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        game.create_emblem(player, Card::test_index(5463));
        let before = game.snapshot();
        let mut object = game.state.objects[creature as usize].clone();
        object.controller = player;
        let world = Projected { game: &game, changed: creature, object };
        let query = characteristic_query::CharacteristicQuery::in_world(&world);
        assert_eq!(query.stats(creature).power, 4);
        assert_eq!(query.stats(creature).toughness, 4);
        assert_eq!(stats(&game, creature), (2, 2));
        assert_eq!(game.snapshot(), before);
    }
}

#[test]
fn ajani_emblem_enables_power_dependent_mana_to_cast_and_survives_suppression() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Doc Samson, Super Psychiatrist", Zone::Battlefield);
        let spell = put(&mut game, player, "Tajuru Pathwarden", Zone::Hand);
        game.state.players[player.index()].mana = [0; 6];
        let announce = Action::AnnounceCast { card: spell };
        assert!(!game.legal_actions().contains(&announce));
        game.create_emblem(player, Card::test_index(5463));
        assert_eq!(stats(&game, source), (5, 8));
        assert!(game.legal_actions().contains(&announce));
        let saved = game.snapshot();
        for _ in 0..2 {
            act(&mut game, announce.clone());
            let mana = game.legal_actions().into_iter().find(|action| matches!(action,
                Action::ActivateMana { card, ability: 0, production: [0, 0, 0, 0, 5, 0], .. } if *card == source
            )).expect("five green mana from the emblem-boosted source");
            act(&mut game, mana);
            pay(&mut game);
            settle(&mut game);
            assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
            game.restore(&saved).unwrap();
        }
        let aura = put(&mut game, player.other(), "Fresh Start", Zone::Battlefield);
        game.state.objects[aura as usize].attached_to = Some(game.object_ref(source));
        assert_eq!(stats(&game, source), (0, 8));
        assert!(!game.legal_actions().contains(&announce));
        assert!(!game.legal_actions().iter().any(|action| matches!(action,
            Action::ActivateMana { card, .. } if *card == source)));
        let suppressed = game.snapshot();
        game.move_object(aura, Zone::Graveyard);
        assert_eq!(stats(&game, source), (5, 8));
        assert!(game.legal_actions().contains(&announce));
        game.restore(&suppressed).unwrap();
        assert_eq!(stats(&game, source), (0, 8));
        assert!(!game.legal_actions().contains(&announce));
    }
}
