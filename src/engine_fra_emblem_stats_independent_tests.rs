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
