//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn resolve_canopy_targets(g: &mut Game, target: CardId, count: usize) {
    let mut chosen = 0;
    for _ in 0..80 {
        g.prepare_triggers();
        let actions = g.legal_actions();
        let choice = Action::ChooseTarget {
            target: Target::Card(target),
        };
        if chosen < count && actions.contains(&choice) {
            act(g, choice);
            chosen += 1;
        } else if chosen == count
            && g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            break;
        } else {
            choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
        }
    }
    assert_eq!(chosen, count);
}

#[test]
fn canopy_enters_tapped_then_taps_for_green() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let canopy = put(&mut g, p, "Roiling Canopy", Zone::Hand);
        act(&mut g, Action::PlayLand { card: canopy });
        assert!(g.object_view(canopy).tapped);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ActivateMana { card, .. } if *card == canopy))
        );
        g.state.objects[canopy as usize].tapped = false;
        let before = g.state.players[p.index()].mana[4];
        let action = g.legal_actions().into_iter().find(|a| matches!(a, Action::ActivateMana { card, ability: 0, production: [0, 0, 0, 0, 1, 0], .. } if *card == canopy)).unwrap();
        act(&mut g, action);
        assert_eq!(g.state.players[p.index()].mana[4], before + 1);
    }
}

#[test]
fn canopy_requires_five_other_forests_at_trigger_and_resolution() {
    {
        let p = Player::P0;
        for other_count in [4, 5] {
            for remove_other in [false, true] {
                let mut g = scenario(p);
                let canopy = put(&mut g, p, "Roiling Canopy", Zone::Battlefield);
                let forests: Vec<_> = (0..other_count)
                    .map(|_| put(&mut g, p, "Forest", Zone::Battlefield))
                    .collect();
                let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
                let incoming = put(&mut g, p, "Forest", Zone::Hand);
                g.move_object(incoming, Zone::Battlefield);
                g.prepare_triggers();
                if other_count == 4 {
                    assert!(
                        !g.state
                            .pending_triggers
                            .iter()
                            .chain(g.state.stack.iter())
                            .any(|item| item.source.is_some_and(|source| source.id == canopy))
                    );
                    continue;
                }
                if remove_other {
                    g.move_object(forests[0], Zone::Graveyard);
                }
                resolve_canopy_targets(&mut g, target, 1);
                assert_eq!(
                    stats(&g, target),
                    if remove_other { (2, 2) } else { (5, 5) }
                );
            }
        }
    }
}
