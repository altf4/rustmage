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
fn enter(g: &mut Game, id: CardId) {
    for _ in 0..32 {
        if g.object_view(id).zone == Zone::Battlefield {
            return;
        }
        act(g, Action::Pass);
    }
    panic!("Celebrate not entered");
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
                contracts::branch_contract(g.clone());
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
            contracts::branch_contract(g.clone());
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
fn celebrate_paid_independent_entry_triggers_ordered_exile_and_private_recruit() {
    for p in [Player::P0, Player::P1] {
        for first in [0, 1] {
            for nonland in [false, true] {
                let mut g = scenario(p);
                let target = put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
                put(&mut g, p.other(), "Forest", Zone::Battlefield);
                let discard = put(
                    &mut g,
                    p,
                    if nonland { "Shock" } else { "Forest" },
                    Zone::Hand,
                );
                let drawn = put(&mut g, p, "Island", Zone::Library);
                let id = source(&mut g, p);
                run(&mut g, p, Some(target), discard, first, true);
                assert_eq!(g.object_view(target).zone, Zone::Exile);
                assert_eq!(g.object_view(drawn).zone, Zone::Hand);
                assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
                assert_eq!(soldiers(&g, p).len(), usize::from(nonland));
                if let Some(token) = soldiers(&g, p).first() {
                    let v = g.object_view(*token);
                    assert_eq!((v.power, v.toughness, v.colors), (1, 1, 1));
                    assert!(v.card.has_subtype("Human") && v.card.has_subtype("Soldier"));
                }
                let old = g.object_ref(target);
                g.move_object(id, Zone::Graveyard);
                assert_eq!(g.object_view(target).zone, Zone::Battlefield);
                assert_ne!(g.object_ref(target), old);
                assert_eq!(g.object_view(target).controller, p.other());
            }
        }
    }
}
#[test]
fn celebrate_zero_target_still_recruits_and_source_departure_does_not_cancel_recruit() {
    for gone in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let discard = put(&mut g, p, "Shock", Zone::Hand);
        let id = source(&mut g, p);
        if gone {
            enter(&mut g, id);
            g.move_object(id, Zone::Hand);
        }
        run(&mut g, p, None, discard, 0, false);
        assert_eq!(soldiers(&g, p).len(), 1);
        assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
    }
}
#[test]
fn celebrate_source_gone_before_resolution_never_exiles_but_target_stale_does_not_block_recruit() {
    for mode in 0..2 {
        let p = Player::P0;
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let discard = put(&mut g, p, "Shock", Zone::Hand);
        let id = source(&mut g, p);
        enter(&mut g, id);
        if mode == 0 {
            g.move_object(id, Zone::Hand);
        } else {
            for _ in 0..32 {
                if matches!(g.state.pending, Pending::Targets { .. }) {
                    break;
                }
                if matches!(g.state.pending, Pending::EffectDiscard { .. }) {
                    act(&mut g, Action::Discard { card: discard });
                } else {
                    choose(&mut g, |a| {
                        matches!(a, Action::Pass | Action::PutTrigger { .. })
                    });
                }
            }
            assert!(matches!(g.state.pending, Pending::Targets { .. }));
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            let old = g.object_ref(target);
            g.move_object(target, Zone::Hand);
            g.move_object(target, Zone::Battlefield);
            assert_ne!(g.object_ref(target), old);
        }
        run(
            &mut g,
            p,
            if mode == 0 { Some(target) } else { None },
            discard,
            0,
            false,
        );
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert_eq!(soldiers(&g, p).len(), 1);
    }
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
