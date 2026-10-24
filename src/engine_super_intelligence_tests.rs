//! The upkeep player is stable while its queued draw remains in this turn.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn pay(g: &mut Game) {
    for _ in 0..20 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, action);
    }
    panic!("unpaid Super Intelligence action");
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    settle(g);
    id
}
fn aura(g: &mut Game, p: Player, target: CardId) -> CardId {
    paid(g, p, "Super Intelligence", &[Target::Card(target)])
}
fn packet(g: &mut Game, aura: CardId) -> SpellView {
    g.prepare_triggers();
    for _ in 0..80 {
        if let Some(s) = g
            .state
            .stack
            .iter()
            .find(|s| s.card == aura && matches!(s.kind, StackKind::TriggeredAbility { .. }))
        {
            return s.clone();
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                )
            })
            .unwrap();
        checked(g, action);
    }
    panic!("missing upkeep packet");
}
fn fixture_main(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.pending = Pending::Priority;
    g.state.step = Step::Main1;
    g.state.players[p.index()].mana = [30; 6];
}
#[test]
fn msh_super_paid_aura_cost_targets_attachment_and_private_hand() {
    for p in [Player::P0, Player::P1] {
        for host in [p, p.other()] {
            let mut g = setup(p);
            let bear = named(&mut g, host, "Bear Cub", Zone::Battlefield);
            let land = named(&mut g, p, "Island", Zone::Battlefield);
            let id = named(&mut g, p, "Super Intelligence", Zone::Hand);
            assert!(g.observe(p.other()).objects.iter().all(|o| o.id != id));
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
            checked(&mut g, Action::AnnounceCast { card: id });
            reject(&mut g, Action::FinishTargets);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(host),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                checked(&mut g, Action::FinishTargets);
            }
            let request = g.announcement_payment_request(p).unwrap();
            assert_eq!((request.required, request.generic), ([0, 1, 0, 0, 0, 0], 0));
            assert!(g.announcement_payment_request(p.other()).is_none());
            reject(&mut g, Action::PayCosts { payment: [0; 6] });
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.object_view(id).attached_to, Some(g.object_ref(bear)));
            assert_eq!(g.object_view(id).controller, p);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            assert!(g.state.step_sources.contains(&id));
        }
    }
}
#[test]
fn msh_super_actual_turns_follow_recipient_controller_not_aura_controller() {
    for p in [Player::P0, Player::P1] {
        for host in [p, p.other()] {
            let mut g = setup(p);
            let bear = named(&mut g, host, "Bear Cub", Zone::Battlefield);
            aura(&mut g, p, bear);
            next_turn(&mut g);
            assert_eq!(g.state.active, p.other());
            assert_eq!(g.state.step, Step::Upkeep);
            let top = *g.state.players[p.other().index()].library.last().unwrap();
            settle(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].cards_drawn_this_turn,
                u64::from(host == p.other())
            );
            if host == p.other() {
                assert_eq!(g.object_view(top).zone, Zone::Hand);
                assert!(g.observe(p).objects.iter().all(|o| o.id != top));
            }
            next_turn(&mut g);
            assert_eq!(g.state.active, p);
            assert_eq!(g.state.step, Step::Upkeep);
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                u64::from(host == p)
            );
        }
    }
}
#[test]
fn msh_super_queued_draw_survives_aura_and_recipient_departure() {
    for p in [Player::P0, Player::P1] {
        for kill_aura in [false, true] {
            let mut g = setup(p);
            let host = p.other();
            let bear = named(&mut g, host, "Bear Cub", Zone::Battlefield);
            let id = aura(&mut g, p, bear);
            next_turn(&mut g);
            let item = packet(&mut g, id);
            assert_eq!(g.state.active, host);
            assert_eq!(item.controller, p);
            assert!(item.targets.is_empty());
            g.state.players[host.index()].mana = [30; 6];
            paid(
                &mut g,
                host,
                if kill_aura { "Disenchant" } else { "Unsummon" },
                &[Target::Card(if kill_aura { id } else { bear })],
            );
            assert_eq!(g.state.active, host);
            assert_eq!(g.state.step, Step::Upkeep);
            assert_eq!(g.state.players[host.index()].cards_drawn_this_turn, 1);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            assert!(!g.state.step_sources.contains(&id));
        }
    }
}
#[test]
fn msh_super_queued_draw_ignores_later_attachment_and_aura_control_changes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let host = p.other();
        let bear = named(&mut g, host, "Bear Cub", Zone::Battlefield);
        let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let id = aura(&mut g, p, bear);
        next_turn(&mut g);
        let item = packet(&mut g, id);
        assert_eq!(item.controller, p);
        // A valid changed attachment/control fixture distinguishes event-time
        // qualification from a live lookup during this already queued program.
        g.state.objects[id as usize].attached_to = Some(g.object_ref(own));
        let reference = g.object_ref(id);
        g.change_controller(reference, host);
        assert!(!steps::attached_controller_turn(&g, reference, host));
        settle(&mut g);
        assert_eq!(g.state.active, host);
        assert_eq!(g.state.players[host.index()].cards_drawn_this_turn, 1);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        next_turn(&mut g);
        settle(&mut g);
        assert_eq!(g.state.active, p);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
    }
}
#[test]
fn msh_super_paid_continuous_control_before_and_after_upkeep_qualification() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let q = p.other();
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let id = aura(&mut g, p, bear);
        fixture_main(&mut g, q);
        let control = paid(&mut g, q, "Confiscate", &[Target::Card(bear)]);
        assert_eq!(g.object_view(bear).controller, q);
        next_turn(&mut g);
        settle(&mut g);
        assert_eq!(g.state.active, p);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        next_turn(&mut g);
        packet(&mut g, id);
        assert_eq!(g.state.active, q);
        g.state.players[q.index()].mana = [30; 6];
        paid(&mut g, q, "Disenchant", &[Target::Card(control)]);
        assert_eq!(g.object_view(bear).controller, p);
        assert_eq!(g.state.players[q.index()].cards_drawn_this_turn, 1);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
    }
}
#[test]
fn msh_super_multiple_aura_owners_each_draw_for_same_upkeep_player() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let first = aura(&mut g, p, bear);
        fixture_main(&mut g, p.other());
        let second = aura(&mut g, p.other(), bear);
        next_turn(&mut g);
        assert_eq!(g.state.active, p);
        g.prepare_triggers();
        assert_eq!(
            g.state
                .pending_triggers
                .iter()
                .chain(&g.state.ordering_triggers)
                .chain(&g.state.stack)
                .filter(|s| [first, second].contains(&s.card))
                .count(),
            2
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 0);
    }
}
#[test]
fn msh_super_all_invalid_aura_target_never_installs_upkeep_observer() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let id = named(&mut g, p, "Super Intelligence", Zone::Hand);
        cast(&mut g, id, None, &[Target::Card(bear)]);
        paid(&mut g, p, "Unsummon", &[Target::Card(bear)]);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert!(g.object_view(id).attached_to.is_none());
        assert!(!g.state.step_sources.contains(&id));
        next_turn(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 0);
    }
}
#[test]
fn msh_super_exact_attachment_guard_rejects_stale_source_and_recipient_incarnations() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let id = aura(&mut g, p, bear);
        let reference = g.object_ref(id);
        assert!(steps::attached_controller_turn(&g, reference, p));
        assert!(!steps::attached_controller_turn(&g, reference, p.other()));
        let before = g.snapshot();
        g.move_object_raw(bear, Zone::Hand);
        g.move_object_raw(bear, Zone::Battlefield);
        assert!(!steps::attached_controller_turn(&g, reference, p));
        g.restore(&before).unwrap();
        g.move_object_raw(id, Zone::Hand);
        g.move_object_raw(id, Zone::Battlefield);
        assert!(!steps::attached_controller_turn(&g, reference, p));
    }
}
