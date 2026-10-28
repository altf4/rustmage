//! Seed Suture's two prepared creatures and hybrid spell.
use super::big_helpers::{checked, settle, setup};
use super::*;

fn announce(g: &mut Game, action: Action, target: Option<Target>) {
    checked(g, action);
    let mut target = target;
    for _ in 0..12 {
        if g.state.announcing.is_none() { return; }
        let actions = g.legal_actions();
        let choice = target.map(|target| Action::ChooseTarget { target });
        let action = if choice.as_ref().is_some_and(|a| actions.contains(a)) {
            target = None; choice.unwrap()
        } else {
            actions.into_iter().find(|a| matches!(a, Action::ChooseSpellCost { .. } | Action::FinishTargets | Action::PayCosts { .. })).unwrap()
        };
        checked(g, action);
    }
    panic!("announcement did not complete");
}
#[test]
fn seed_suture_both_seats_and_hybrid_colors_with_invalidated_target() {
    for p in [Player::P0, Player::P1] {
        for (name, flying) in [("Emergency Phytomedic // Seed Suture", false), ("Blossom-Blessed Angel // Seed Suture", true)] {
            for green in [false, true] {
                for invalid in [false, true] {
                    let mut g = setup(p);
                    let source = put(&mut g, p, name, Zone::Hand);
                    announce(&mut g, Action::AnnounceCast { card: source }, None);
                    settle(&mut g);
                    assert_eq!(g.keywords(source).contains(Keywords::FLYING), flying);
                    assert_eq!(g.keywords(source).contains(Keywords::VIGILANCE), flying);
                    let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                    let copy = g.state.preparations.as_ref().unwrap().iter().find(|r| r.permanent == g.object_ref(source)).unwrap().spell_copy.card;
                    g.state.players[p.index()].mana = if green { [0,0,0,0,1,0] } else { [1,0,0,0,0,0] };
                    let life = g.state.players[p.index()].life;
                    announce(&mut g, Action::AnnounceExile { card: copy }, Some(Target::Card(target)));
                    assert!(!g.is_prepared(g.object_ref(source)));
                    if invalid { g.move_object(target, Zone::Exile); }
                    settle(&mut g);
                    assert_eq!(g.counter_count(target, CounterKind::PlusOne), u16::from(!invalid));
                    assert_eq!(g.state.players[p.index()].life, life + i32::from(!invalid));
                    assert_eq!(g.object_view(copy).zone, Zone::Outside);
                }
            }
        }
    }
}
