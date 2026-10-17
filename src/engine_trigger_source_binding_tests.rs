use super::tests::{scenario, spawn};
use super::*;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn checked(g: &mut Game, action: Action) {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let before = g.snapshot();
    assert_eq!(
        g.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(g.snapshot(), before);
    let mut branch = {
        let parent = g.clone();
        parent.clone()
    };
    branch.apply_index(actor, revision, index).unwrap();
    g.apply(actor, action.clone()).unwrap();
    assert_eq!(g.snapshot(), branch.snapshot());
    for player in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(player)).unwrap(),
            serde_json::to_value(branch.observe(player)).unwrap()
        );
    }
    let after = g.snapshot();
    g.restore(&before).unwrap();
    assert_eq!(
        g.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    g.apply(actor, action).unwrap();
    assert_eq!(g.snapshot(), after);
}
fn settle(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..100 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .or_else(|| actions.iter().find(|a| matches!(a, Action::FinishTargets)))
                .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                .unwrap()
                .clone()
        };
        checked(g, action);
    }
    panic!("unsettled {:?}", g.state.pending);
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    g.state.players[p.index()].mana = [30; 6];
    g
}

#[test]
fn paid_loyalist_excludes_exact_grave_source_and_returns_real_elf_or_changeling_both_seats() {
    for p in [Player::P0, Player::P1] {
        for name in ["Llanowar Elves", "Three Tree Mascot"] {
            let mut g = setup(p);
            let ally = named(&mut g, p, name, Zone::Battlefield);
            let original_stats = g.stats(ally);
            let loyalist = named(&mut g, p, "Morcant's Loyalist", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: loyalist });
            settle(&mut g);
            assert_eq!(g.stats(ally).power, original_stats.power + 1);
            assert_eq!(g.stats(ally).toughness, original_stats.toughness + 1);
            assert_eq!(
                g.stats(loyalist),
                Stats {
                    power: 3,
                    toughness: 2
                }
            );
            let before = g.object_ref(loyalist);
            let bounds: Vec<_> = [loyalist, ally]
                .into_iter()
                .map(|id| BoundTarget {
                    group: 0,
                    target: Target::Card(id),
                    object: Some(g.object_ref(id)),
                })
                .collect();
            g.move_target_batch(&bounds, Zone::Graveyard);
            let after = g.object_ref(loyalist);
            g.prepare_triggers();
            let item = &g.state.announcing.as_ref().unwrap().item;
            assert_eq!(item.object, before);
            assert_eq!(item.target_source(), after);
            assert_eq!(item.controller, p);
            assert_eq!(item.source_move.as_ref().unwrap().origin, Zone::Battlefield);
            assert!(!g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(loyalist)
            }));
            let saved = g.snapshot();
            let revision = g.revision();
            assert_eq!(
                g.apply(
                    p,
                    Action::ChooseTarget {
                        target: Target::Card(loyalist)
                    }
                ),
                Err(Error::IllegalAction)
            );
            assert_eq!(g.snapshot(), saved);
            assert_eq!(g.revision(), revision);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(ally),
                },
            );
            settle(&mut g);
            assert_eq!(g.object_view(ally).zone, Zone::Hand);
            assert_eq!(g.object_view(loyalist).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn subsequent_source_reentry_is_other_but_selected_target_never_rebinds() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Morcant's Loyalist", Zone::Battlefield);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        g.move_object(source, Zone::Graveyard);
        let tracked = g.object_ref(source);
        g.move_object(source, Zone::Hand);
        g.move_object(source, Zone::Graveyard);
        g.prepare_triggers();
        assert_eq!(
            g.state.announcing.as_ref().unwrap().item.target_source(),
            tracked
        );
        assert!(g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(source)
        }));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(elf),
            },
        );
        g.move_object(elf, Zone::Hand);
        g.move_object(elf, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(elf).zone, Zone::Graveyard);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    }
}

#[test]
fn foreign_death_source_keeps_controller_and_owner_grave_routing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p.other(), "Morcant's Loyalist", Zone::Battlefield);
        g.state.objects[source as usize].controller = p;
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        let decoy = named(&mut g, p.other(), "Llanowar Elves", Zone::Graveyard);
        g.move_object(source, Zone::Graveyard);
        g.prepare_triggers();
        assert_eq!(g.actor(), Some(p));
        assert_eq!(g.state.announcing.as_ref().unwrap().item.controller, p);
        assert_eq!(g.object_view(source).controller, p.other());
        assert_eq!(g.object_view(source).owner, p.other());
        assert!(!g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(decoy)
        }));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(elf),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(elf).zone, Zone::Hand);
    }
}

#[test]
fn simultaneous_other_death_observer_does_not_inherit_its_unrelated_source_move() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let observer = named(&mut g, p.other(), "Agent Venom", Zone::Battlefield);
        g.state.objects[observer as usize].controller = p;
        let before = g.object_ref(observer);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let bounds: Vec<_> = [observer, elf]
            .into_iter()
            .map(|id| BoundTarget {
                group: 0,
                target: Target::Card(id),
                object: Some(g.object_ref(id)),
            })
            .collect();
        g.move_target_batch(&bounds, Zone::Graveyard);
        assert_eq!(g.state.pending_triggers.len(), 1);
        let item = &g.state.pending_triggers[0];
        assert_eq!(item.object, before);
        assert_eq!(item.controller, p);
        assert_eq!(item.source_move, None);
        assert_eq!(item.target_source(), before);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 19);
        assert_eq!(g.observe(p).players[p.index()].hand_count, 1);
    }
}

#[test]
fn mandatory_other_target_disappears_when_only_the_new_source_is_in_graveyard() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Morcant's Loyalist", Zone::Battlefield);
        g.move_object(source, Zone::Graveyard);
        assert!(g.state.pending_triggers[0].source_move.is_some());
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert!(g.state.stack.is_empty());
        assert!(g.state.announcing.is_none());
    }
}

#[test]
fn reannouncing_bound_ability_keeps_exact_exclusion_and_can_select_a_later_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Morcant's Loyalist", Zone::Battlefield);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        g.move_object(source, Zone::Graveyard);
        let tracked = g.object_ref(source);
        g.prepare_triggers();
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(elf),
            },
        );
        while g.state.announcing.is_some() {
            checked(&mut g, Action::FinishTargets);
        }
        let mut item = g.state.stack.pop().unwrap();
        item.targets.clear();
        g.state.pending = Pending::Priority;
        g.state.priority = p;
        // Exercise the shared reannouncement primitive; this does not admit an
        // unsupported printed spell that changes another ability's targets.
        assert!(g.start_announcement(item, true));
        assert!(!g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(source)
        }));
        g.move_object(source, Zone::Hand);
        g.move_object(source, Zone::Graveyard);
        assert_eq!(
            g.state.announcing.as_ref().unwrap().item.target_source(),
            tracked
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Hand);
    }
}

#[test]
fn source_move_context_is_sparse_owned_and_accounted_in_pending_and_stack() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, "Morcant's Loyalist", Zone::Battlefield);
    let elf = named(&mut g, Player::P0, "Llanowar Elves", Zone::Graveyard);
    g.move_object(source, Zone::Graveyard);
    let mut without = g.clone();
    without.state.pending_triggers[0].source_move = None;
    assert_eq!(
        g.state_bytes() - without.state_bytes(),
        std::mem::size_of::<TriggerSourceMoveView>()
    );
    g.prepare_triggers();
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(elf),
        },
    );
    while g.state.announcing.is_some() {
        checked(&mut g, Action::FinishTargets);
    }
    let mut without = g.clone();
    without.state.stack[0].source_move = None;
    assert_eq!(
        g.state_bytes() - without.state_bytes(),
        std::mem::size_of::<TriggerSourceMoveView>()
    );
}
