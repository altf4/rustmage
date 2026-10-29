use super::*;

fn activate_tam(g: &mut Game, source: CardId) {
    let controller = g.state.objects[source as usize].controller;
    g.state.players[controller.index()].mana = [1, 1, 1, 1, 1, 0];
    act(g, Action::AnnounceActivate { card: source, ability: 0, counter_payment: Counters::default() });
    pay(g);
}

fn advance_to_proliferate_or_done(g: &mut Game) {
    for _ in 0..40 {
        if matches!(g.state.pending, Pending::Proliferate { .. })
            || (g.state.pending == Pending::Priority && g.state.stack.is_empty() && g.state.resolving.is_none()) { return; }
        act(g, Action::Pass);
    }
    panic!("Tam activation did not advance: {:?}", g.state.pending);
}

#[test]
fn tam_discount_enables_real_constrained_cast_and_respects_suppression() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let walker = put(&mut g, player, "Ajani, Caller of the Pride", Zone::Hand);
        g.state.players[player.index()].mana = [2, 0, 0, 0, 0, 0];
        let cast = Action::AnnounceCast { card: walker };
        assert!(!g.legal_actions().contains(&cast));
        let tam = put(&mut g, player, "Tam, the Possibility", Zone::Battlefield);
        assert!(g.legal_actions().contains(&cast));
        let aura = put(&mut g, player.other(), "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(tam));
        assert!(!g.legal_actions().contains(&cast));
        let suppressed = g.snapshot();
        assert!(g.apply(player, cast.clone()).is_err());
        assert_eq!(g.snapshot(), suppressed);
        g.move_object(aura, Zone::Graveyard);
        assert!(g.legal_actions().contains(&cast));
        let before = g.snapshot();
        for _ in 0..2 {
            act(&mut g, cast.clone());
            pay(&mut g);
            assert_eq!(g.state.players[player.index()].mana, [0; 6]);
            settle(&mut g);
            assert_eq!(g.object_view(walker).zone, Zone::Battlefield);
            g.restore(&before).unwrap();
        }
    }
}

#[test]
fn tam_zero_and_two_types_use_resolution_controller_after_source_changes() {
    for player in [Player::P0, Player::P1] {
        for count in [0, 2] {
            for leave in [false, true] {
                let mut g = scenario(player);
                let tam = put(&mut g, player, "Tam, the Possibility", Zone::Battlefield);
                if count == 2 {
                    let ajani = put(&mut g, player, "Ajani, Caller of the Pride", Zone::Battlefield);
                    let kaito = put(&mut g, player, "Kaito, Cunning Infiltrator", Zone::Battlefield);
                    g.state.objects[ajani as usize].counters.loyalty = 3;
                    g.state.objects[kaito as usize].counters.loyalty = 3;
                }
                let creature = put(&mut g, player, "Bear Cub", Zone::Battlefield);
                g.add_counter(creature, CounterKind::PlusOne, 1);
                activate_tam(&mut g, tam);
                if leave { g.move_object(tam, Zone::Graveyard); }
                else {
                    let source = g.object_ref(tam);
                    g.change_controller(source, player.other());
                }
                let saved = g.snapshot();
                for _ in 0..2 {
                    advance_to_proliferate_or_done(&mut g);
                    for _ in 0..count {
                        assert!(matches!(g.state.pending, Pending::Proliferate { player: p } if p == player));
                        let object = g.object_ref(creature);
                        act(&mut g, Action::ProliferatePermanent { object });
                        act(&mut g, Action::FinishProliferate);
                    }
                    settle(&mut g);
                    assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 1 + count);
                    g.restore(&saved).unwrap();
                }
            }
        }
    }
}

#[test]
fn copied_tam_activation_uses_copy_controllers_planeswalker_types() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let tam = put(&mut g, player, "Tam, the Possibility", Zone::Battlefield);
        let ajani = put(&mut g, player.other(), "Ajani, Caller of the Pride", Zone::Battlefield);
        let kaito = put(&mut g, player.other(), "Kaito, Cunning Infiltrator", Zone::Battlefield);
        g.state.objects[ajani as usize].counters.loyalty = 3;
        g.state.objects[kaito as usize].counters.loyalty = 3;
        let creature = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        g.add_counter(creature, CounterKind::PlusOne, 1);
        activate_tam(&mut g, tam);
        let original = g.state.stack.last().unwrap().clone();
        let mut copier = original.clone();
        copier.controller = player.other();
        copier.targets = vec![BoundTarget { group: 0, target: Target::Stack(original.id), object: None }];
        g.copy_target_ability(&copier, 0);
        assert!(g.state.stack.last().unwrap().is_copy);
        let saved = g.snapshot();
        for _ in 0..2 {
            advance_to_proliferate_or_done(&mut g);
            for _ in 0..2 {
                assert!(matches!(g.state.pending, Pending::Proliferate { player: p } if p == player.other()));
                let object = g.object_ref(creature);
                act(&mut g, Action::ProliferatePermanent { object });
                act(&mut g, Action::FinishProliferate);
            }
            settle(&mut g);
            assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 3);
            g.restore(&saved).unwrap();
        }
    }
}
