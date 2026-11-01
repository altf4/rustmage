//! Focused printed outcomes; shared engine invariants live in core tests.
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
        Card::from_name("Variable Chaser // Arc of Fortune")
            .unwrap()
            .prepared_spell()
            .unwrap()
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
fn arc_draws_seven_even_when_discard_is_replaced_or_the_hand_is_empty() {
    {
        let p = Player::P0;
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
