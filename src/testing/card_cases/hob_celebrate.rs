//! Focused printed outcomes; shared engine invariants live in core tests.
//! Separate targeted exile and private recruit entry abilities.
use super::*;
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
}
fn source(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Celebrate the Mountain-king", Zone::Hand);
    g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 3];
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}

fn run(
    g: &mut Game,
    p: Player,
    target: Option<CardId>,
    discard: CardId,
    first: u16,
    legality: bool,
) {
    let mut checked = false;
    for _ in 0..96 {
        if quiet(g) {
            return;
        }
        if matches!(g.state.pending, Pending::Targets { .. }) {
            if legality && !checked {
                let own = g
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .find(|(_, o)| o.owner == p && o.zone == Zone::Battlefield)
                    .map(|(i, _)| i as CardId)
                    .unwrap();
                reject(
                    g,
                    Action::ChooseTarget {
                        target: Target::Card(own),
                    },
                );
                reject(
                    g,
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    },
                );
                let enemy_land = g
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .find(|(_, o)| {
                        o.owner == p.other() && o.zone == Zone::Battlefield && o.card.is_land()
                    })
                    .map(|(i, _)| i as CardId)
                    .unwrap();
                reject(
                    g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy_land),
                    },
                );
                checked = true;
            }
            if let Some(id) = target {
                act(
                    g,
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    },
                );
            }
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(g, Action::FinishTargets);
            }
        } else if matches!(g.state.pending, Pending::EffectDiscard { .. }) {
            assert_eq!(g.actor(), Some(p));
            assert!(!g.observe(p.other()).objects.iter().any(|v| v.id == discard));

            act(g, Action::Discard { card: discard });
        } else {
            let actions = g.legal_actions();
            let preferred = g
                .state
                .ordering_triggers
                .iter()
                .find(|item| item.kind == StackKind::TriggeredAbility { ability: first })
                .map(|item| Action::PutTrigger { id: item.id });
            let action = preferred
                .filter(|a| actions.contains(a))
                .or_else(|| {
                    actions
                        .into_iter()
                        .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                })
                .unwrap_or_else(|| panic!("unhandled {:?}", g.state.pending));
            act(g, action);
        }
    }
    panic!("Celebrate resolution budget {:?}", g.state.pending);
}
fn soldiers(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.card == Card::catalogue().nth(5241).unwrap()
        })
        .map(|(id, _)| id as CardId)
        .collect()
}

#[test]
fn celebrate_stolen_enemy_permanent_returns_to_owner_and_replaced_discard_recruits() {
    let p = Player::P0;
    let mut g = scenario(p);
    let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let old = g.object_ref(target);
    g.change_controller(old, p.other());
    put(&mut g, p, "Rest in Peace", Zone::Battlefield);
    let discard = put(&mut g, p, "Shock", Zone::Hand);
    let id = source(&mut g, p);
    run(&mut g, p, Some(target), discard, 0, false);
    assert_eq!(g.object_view(target).zone, Zone::Exile);
    assert_eq!(g.object_view(discard).zone, Zone::Exile);
    assert_eq!(soldiers(&g, p).len(), 1);
    g.move_object(id, Zone::Hand);
    assert_eq!(g.object_view(target).zone, Zone::Battlefield);
    assert_eq!(g.object_view(target).controller, p);
}
