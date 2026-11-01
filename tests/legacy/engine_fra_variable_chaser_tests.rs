use super::*;

fn variable_finish(g: &mut Game) {
    for _ in 0..96 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Variable Chaser resolution did not finish");
}
fn variable_prepare(g: &mut Game, p: Player) -> (CardId, ObjectRef) {
    let source = put(g, p, "Variable Chaser // Arc of Fortune", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
    act(g, Action::AnnounceCast { card: source });
    pay(g);
    variable_finish(g);
    let reference = g.object_ref(source);
    let copy = g
        .state
        .preparations
        .as_ref()
        .unwrap()
        .iter()
        .find(|r| r.permanent == reference)
        .unwrap()
        .spell_copy;
    assert_eq!(
        g.state.objects[copy.card as usize].card,
        Card::test_index(5460)
    );
    assert!(g.keywords(source).contains(Keywords::FLYING));
    (source, copy)
}
fn variable_wait_choice(g: &mut Game) {
    for _ in 0..64 {
        if matches!(g.state.pending, Pending::EffectOptional { .. }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("optional wheel choice did not start");
}

#[test]
fn arc_collects_both_apnap_choices_before_discard_and_draw() {
    for p in [Player::P0, Player::P1] {
        for first_accept in [false, true] {
            for second_accept in [false, true] {
                let mut g = scenario(p);
                let (source, copy) = variable_prepare(&mut g, p);
                let own = put(&mut g, p, "Shock", Zone::Hand);
                let other = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
                g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
                act(&mut g, Action::AnnounceExile { card: copy.card });
                pay(&mut g);
                assert!(!g.is_prepared(g.object_ref(source)));
                variable_wait_choice(&mut g);
                assert_eq!(stats(&g, source), (3, 4));
                assert_eq!(g.actor(), Some(p));
                assert_eq!(
                    g.observe(p.other()).optional_wheel_choices,
                    Some([None, None])
                );
                let saved = g.snapshot();
                assert_eq!(
                    g.apply(p.other(), Action::ChooseOptional { accept: true }),
                    Err(Error::WrongActor)
                );
                assert_eq!(g.snapshot(), saved);
                act(
                    &mut g,
                    Action::ChooseOptional {
                        accept: first_accept,
                    },
                );
                assert_eq!(g.actor(), Some(p.other()));
                assert_eq!(g.object_view(own).zone, Zone::Hand);
                assert_eq!(g.object_view(other).zone, Zone::Hand);
                let mut choices = [None, None];
                choices[p.index()] = Some(first_accept);
                assert_eq!(g.observe(p.other()).optional_wheel_choices, Some(choices));
                super::contracts::branch_contract(g.clone());
                act(
                    &mut g,
                    Action::ChooseOptional {
                        accept: second_accept,
                    },
                );
                variable_finish(&mut g);
                assert_eq!(g.hand_count(p), if first_accept { 7 } else { 1 });
                assert_eq!(g.hand_count(p.other()), if second_accept { 7 } else { 1 });
                assert_eq!(
                    g.object_view(own).zone,
                    if first_accept {
                        Zone::Graveyard
                    } else {
                        Zone::Hand
                    }
                );
                assert_eq!(
                    g.object_view(other).zone,
                    if second_accept {
                        Zone::Graveyard
                    } else {
                        Zone::Hand
                    }
                );
            }
        }
    }
}

#[test]
fn arc_draws_seven_even_when_discard_is_replaced_or_the_hand_is_empty() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (_, copy) = variable_prepare(&mut g, p);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let discarded = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
        act(&mut g, Action::AnnounceExile { card: copy.card });
        pay(&mut g);
        variable_wait_choice(&mut g);
        act(&mut g, Action::ChooseOptional { accept: true });
        act(&mut g, Action::ChooseOptional { accept: true });
        variable_finish(&mut g);
        assert_eq!(g.hand_count(p), 7);
        assert_eq!(g.hand_count(p.other()), 7);
        assert_eq!(g.object_view(discarded).zone, Zone::Exile);
    }
}

#[test]
fn arc_discards_both_hands_before_interrupted_first_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (_, copy) = variable_prepare(&mut g, p);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        put(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
        let first = put(&mut g, p, "Forest", Zone::Hand);
        let second = put(&mut g, p.other(), "Island", Zone::Hand);
        put(&mut g, p, "Shock", Zone::Library);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
        act(&mut g, Action::AnnounceExile { card: copy.card });
        pay(&mut g);
        variable_wait_choice(&mut g);
        act(&mut g, Action::ChooseOptional { accept: true });
        let saved = g.snapshot();
        super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            act(&mut g, Action::ChooseOptional { accept: true });
            assert!(matches!(g.state.pending, Pending::MiracleReveal { player } if player == p));
            assert_eq!(g.object_view(first).zone, Zone::Exile);
            assert_eq!(g.object_view(second).zone, Zone::Exile);
            assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
            assert_eq!(
                g.state.players[p.other().index()].cards_discarded_this_turn,
                1
            );
            assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 0);
        }
    }
}
