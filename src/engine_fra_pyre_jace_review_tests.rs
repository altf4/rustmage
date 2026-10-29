use super::*;

fn review_pyre(g: &mut Game, player: Player) -> CardId {
    let id = put(g, player, "Bear Cub", Zone::Battlefield);
    g.state.objects[id as usize].card = Card::from_name("Pyre Rhymer // Molten Tide").unwrap();
    g.register_battlefield_sources(id);
    id
}

fn blank_pyre(g: &mut Game, player: Player, id: CardId) {
    let source = g.object_ref(id);
    let mut item = g.activation_item(player, id, 0);
    item.targets = vec![BoundTarget { group: 0, target: Target::Card(id), object: Some(source) }];
    g.remove_target_abilities_until_end_of_turn(&item);
}

#[test]
fn pyre_blanking_prevents_new_prowess_but_not_an_already_queued_trigger() {
    for player in [Player::P0, Player::P1] {
        for before in [true, false] {
            let mut g = scenario(player);
            let pyre = review_pyre(&mut g, player);
            let shock = put(&mut g, player, "Shock", Zone::Hand);
            if before { blank_pyre(&mut g, player, pyre); }
            cast(&mut g, shock, Some(Target::Player(player.other())));
            g.prepare_triggers();
            if !before { blank_pyre(&mut g, player, pyre); }
            let saved = g.snapshot();
            for _ in 0..2 {
                settle(&mut g);
                assert_eq!(stats(&g, pyre), if before { (3, 3) } else { (4, 4) });
                g.restore(&saved).unwrap();
            }
        }
    }
}

#[test]
fn copied_pyre_token_enters_prepared_and_carries_the_prepare_face() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let original = review_pyre(&mut g, player);
        let spell = put(&mut g, player, "Molten Duplication", Zone::Hand);
        let saved = g.snapshot();
        for _ in 0..2 {
            cast(&mut g, spell, Some(Target::Card(original)));
            settle(&mut g);
            let copy = g.state.objects.iter().enumerate().find_map(|(id, object)| {
                (object.zone == Zone::Battlefield && object.controller == player
                    && object.is_copy_token() && object.card == Card::from_name("Pyre Rhymer // Molten Tide").unwrap())
                    .then_some(id as CardId)
            }).expect("copied Pyre token");
            let reference = g.object_ref(copy);
            assert!(g.is_prepared(reference));
            let row = g.state.preparations.as_ref().unwrap().iter().find(|r| r.permanent == reference).unwrap();
            assert_eq!(g.state.objects[row.spell_copy.card as usize].card, Card::test_index(5461));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn jace_delayed_effect_tracks_attacker_after_its_controller_changes() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[source as usize].card = Card::from_name("Jace, Reality Sculptor").unwrap();
        g.state.objects[source as usize].counters.loyalty = 5;
        g.register_battlefield_sources(source);
        let attacker = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
        act(&mut g, Action::AnnounceActivate { card: source, ability: 1, counter_payment: Counters::default() });
        pay(&mut g);
        settle(&mut g);
        g.move_object(source, Zone::Exile);
        g.state.active = player.other();
        g.state.priority = player.other();
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        g.prepare_triggers();
        let reference = g.object_ref(attacker);
        g.change_controller(reference, player);
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert_eq!(g.object_view(attacker).controller, player);
            assert_eq!(stats(&g, attacker), (-2, 3));
            g.restore(&saved).unwrap();
        }
    }
}
