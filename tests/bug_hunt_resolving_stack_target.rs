mod support;
use rustmage as _;
use rustmage_core::engine::*;
use support::*;

#[test]
fn cancel_can_be_redirected_to_a_spell_that_is_resolving() {
    let p = Player::P0;
    let mut g = scenario(p);
    let shock = put(&mut g, p, "Shock", Zone::Hand);
    let cancel = put(&mut g, p.other(), "Cancel", Zone::Hand);
    let bend = put(&mut g, p, "Bolt Bend", Zone::Hand);
    cast(&mut g, shock, Some(Target::Player(p.other())));
    act(&mut g, Action::Pass);
    cast(&mut g, cancel, Some(Target::Spell(shock)));
    act(&mut g, Action::Pass);
    cast(&mut g, bend, Some(Target::Spell(cancel)));
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);
    assert_eq!(g.object_view(bend).zone, Zone::Stack);
    reject(&mut g, Action::FinishStackTargets);
    reject(
        &mut g,
        Action::SetStackTarget {
            index: 0,
            target: Target::Spell(cancel),
        },
    );
    let redirect = Action::SetStackTarget {
        index: 0,
        target: Target::Spell(bend),
    };
    let index = g
        .legal_actions()
        .iter()
        .position(|a| *a == redirect)
        .expect("Bolt Bend remains a spell during its own resolution");
    let checkpoint = g.snapshot();
    let mut indexed = g.clone();
    indexed
        .apply_index(g.actor().unwrap(), g.revision(), index)
        .unwrap();
    act(&mut g, redirect.clone());
    assert_eq!(indexed.snapshot(), g.snapshot());
    act(&mut g, Action::FinishStackTargets);
    assert_eq!(g.object_view(bend).zone, Zone::Graveyard);
    assert_eq!(
        g.state.stack.last().unwrap().targets[0].target,
        Target::Spell(bend)
    );
    settle(&mut g);
    assert_eq!(g.object_view(cancel).zone, Zone::Graveyard);
    assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
    assert_eq!(g.state.players[p.other().index()].life, 18);
    indexed.restore(&checkpoint).unwrap();
    assert!(indexed.legal_actions().contains(&redirect));
}
