use super::*;

fn make_copy(g: &mut Game, player: Player, target: Player) -> CardId {
    put(g, target, "Bear Cub", Zone::Battlefield);
    let spell = put(g, player, "Face Yourself", Zone::Hand);
    cast(g, spell, Some(Target::Player(target)));
    settle(g);
    g.state.objects.iter().enumerate().find_map(|(id, object)| {
        (object.zone == Zone::Battlefield && object.controller == player
            && object.is_copy_token() && object.card.name() == "Bear Cub").then_some(id as CardId)
    }).unwrap()
}

#[test]
fn face_yourself_copies_target_players_creatures_with_copiable_haste() {
    for player in [Player::P0, Player::P1] {
        for target in [player, player.other()] {
            let mut g = scenario(player);
            let copy = make_copy(&mut g, player, target);
            assert!(g.keywords(copy).contains(Keywords::HASTE));
            assert_eq!(g.copiable_recipe(g.object_ref(copy)).exceptions.conditional_sacrifice_abilities, 1);
            assert_eq!(stats(&g, copy), (2, 2));
            super::super::contracts::branch_contract(g.clone());
        }
    }
}

#[test]
fn face_yourself_checks_planeswalker_at_trigger_and_resolution() {
    for player in [Player::P0, Player::P1] {
        for case in 0..3 {
            let mut g = scenario(player);
            let token = make_copy(&mut g, player, player.other());
            let walker = if case == 1 {
                Some(put(&mut g, player, "Ajani, Caller of the Pride", Zone::Battlefield))
            } else { None };
            if let Some(id) = walker { g.state.objects[id as usize].counters.loyalty = 3; }
            g.begin_step(Step::End);
            if case == 1 { g.move_object(walker.unwrap(), Zone::Graveyard); }
            if case == 2 {
                let id = put(&mut g, player, "Ajani, Caller of the Pride", Zone::Battlefield);
                g.state.objects[id as usize].counters.loyalty = 3;
            }
            g.prepare_triggers();
            let saved = g.snapshot();
            for replay in 0..2 {
                if replay == 1 { g.restore(&saved).unwrap(); }
                settle(&mut g);
                assert_eq!(g.object_view(token).zone == Zone::Battlefield, case != 0);
            }
        }
    }
}
