use super::*;

#[test]
fn axe_granted_attack_ability_removed_before_declaration_does_not_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bearer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let axe = put(&mut g, p, "Hunter's Axe", Zone::Battlefield);
        let reference = g.object_ref(bearer);
        g.state.objects[axe as usize].attached_to = Some(reference);
        let blank = put(&mut g, p.other(), "Fresh Start", Zone::Battlefield);
        g.state.objects[blank as usize].attached_to = Some(reference);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: bearer });
        act(&mut g, Action::FinishAttackers);
        for _ in 0..48 {
            let actions = g.legal_actions();
            assert!(
                !actions
                    .iter()
                    .any(|a| matches!(a, Action::ChooseKeyword { .. }))
            );
            if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none()
            {
                break;
            }
            let next = actions
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                .expect("attack continuation");
            act(&mut g, next);
        }
        assert!(!g.keywords(bearer).contains(Keywords::TRAMPLE));
        assert!(!g.keywords(bearer).contains(Keywords::DEATHTOUCH));
    }
}

#[test]
fn axe_queued_trigger_keeps_controller_after_loss_of_ability_and_control() {
    for p in [Player::P0, Player::P1] {
        for change_control in [false, true] {
            let mut g = scenario(p);
            let bearer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let axe = put(&mut g, p, "Hunter's Axe", Zone::Battlefield);
            let reference = g.object_ref(bearer);
            g.state.objects[axe as usize].attached_to = Some(reference);
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            act(&mut g, Action::Attack { card: bearer });
            act(&mut g, Action::FinishAttackers);
            assert!(
                g.state
                    .pending_triggers
                    .iter()
                    .chain(g.state.stack.iter())
                    .any(|item| item.source.is_some_and(|source| source.object == reference)
                        && matches!(item.kind, StackKind::TriggeredAbility { .. }))
            );
            let blank = put(&mut g, p.other(), "Fresh Start", Zone::Battlefield);
            g.state.objects[blank as usize].attached_to = Some(reference);
            if change_control {
                g.change_controller(reference, p.other());
            }
            for _ in 0..48 {
                if g.legal_actions()
                    .contains(&Action::ChooseKeyword { index: 0 })
                {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            assert_eq!(g.actor(), Some(p));
            let before = g.snapshot();
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&before).unwrap();
                }
                act(&mut g, Action::ChooseKeyword { index: 0 });
                assert!(g.keywords(bearer).contains(Keywords::TRAMPLE));
            }
        }
    }
}
