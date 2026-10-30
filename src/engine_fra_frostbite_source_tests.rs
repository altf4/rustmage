use super::*;

#[test]
fn frostbite_sacrifices_its_source_after_it_becomes_a_land() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let frostbite = put(&mut game, player, "Frostbite Pyromental", Zone::Battlefield);
        let source = game.object_ref(frostbite);
        put(&mut game, player, "High Fae Trickster", Zone::Battlefield);
        let aura = put(&mut game, player, "Imprisoned in the Moon", Zone::Hand);

        game.begin_step(Step::End);
        game.prepare_triggers();
        assert!(game.state.stack.iter().any(|item| {
            item.source.is_some_and(|s| s.object == source)
                && matches!(item.kind, StackKind::TriggeredAbility { ability: 1 })
        }));
        cast(&mut game, aura, Some(Target::Card(frostbite)));
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert_eq!(game.current_kinds(frostbite), 2);
        assert_eq!(game.object_ref(frostbite), source);
        assert!(game.state.stack.iter().any(|item| {
            item.source.is_some_and(|s| s.object == source)
                && matches!(item.kind, StackKind::TriggeredAbility { ability: 1 })
        }));

        let saved = game.snapshot();
        for _ in 0..2 {
            for _ in 0..32 {
                if game.state.pending == Pending::Priority
                    && game.state.stack.is_empty()
                    && game.state.resolving.is_none()
                {
                    break;
                }
                choose(&mut game, |action| {
                    matches!(action, Action::Pass | Action::PutTrigger { .. })
                        || matches!(action, Action::Sacrifice { object } if *object == source)
                });
            }
            assert_eq!(game.object_view(frostbite).zone, Zone::Graveyard);
            game.restore(&saved).unwrap();
        }
    }
}
