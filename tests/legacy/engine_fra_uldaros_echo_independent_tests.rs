//! Independent Uldaros test; link under assurance for grouped validation.
use super::*;

#[test]
fn echo_legally_copies_uldaros_trigger_and_retargets_with_rollback() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let echo = put(
            &mut g,
            player,
            "Echo, Perceptive Prodigy",
            Zone::Battlefield,
        );
        let first = put(&mut g, player, "Bear Cub", Zone::Graveyard);
        let second = put(&mut g, player, "Hill Giant", Zone::Graveyard);
        let source = put(&mut g, player, "Shock", Zone::Hand);
        g.state.objects[source as usize].card = Card::test_index(4739);
        assert_eq!(
            g.state.objects[source as usize].card.identity(),
            "Uldaros Theorix"
        );
        act(&mut g, Action::AnnounceCast { card: source });
        pay(&mut g);
        for _ in 0..40 {
            if matches!(g.state.pending, Pending::Targets { .. }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        act(
            &mut g,
            Action::ChooseTargetCardType {
                target: Target::Card(first),
                type_mask: 1,
            },
        );
        act(&mut g, Action::FinishTargets);
        let original = g.state.stack.last().unwrap().id;
        act(
            &mut g,
            Action::AnnounceActivate {
                card: echo,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Stack(original),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        act(&mut g, Action::Pass);
        act(&mut g, Action::Pass);
        assert!(matches!(g.state.pending, Pending::Retarget { player: p } if p == player));
        let saved = g.snapshot();
        let views = [player, player.other()].map(|p| {
            let mut view = g.observe(p);
            view.revision = 0;
            serde_json::to_string(&view).unwrap()
        });
        let revision = g.revision();
        for _ in 0..2 {
            assert_eq!(
                [player, player.other()].map(|p| {
                    let mut view = g.observe(p);
                    view.revision = 0;
                    serde_json::to_string(&view).unwrap()
                }),
                views
            );
            act(
                &mut g,
                Action::SetStackTarget {
                    index: 0,
                    target: Target::Card(second),
                },
            );
            act(&mut g, Action::FinishStackTargets);
            assert!(g.state.stack.last().unwrap().is_copy);
            act(&mut g, Action::Pass);
            act(&mut g, Action::Pass);
            assert_eq!(g.state.objects[second as usize].zone, Zone::Exile);
            assert_eq!(g.state.objects[first as usize].zone, Zone::Graveyard);
            assert_eq!(
                g.state
                    .resolution_free_casts
                    .as_ref()
                    .unwrap()
                    .remaining_mana_value,
                Some(6)
            );
            act(&mut g, Action::FinishResolutionCasts);
            act(&mut g, Action::Pass);
            act(&mut g, Action::Pass);
            assert_eq!(g.state.objects[first as usize].zone, Zone::Exile);
            assert_eq!(
                g.state
                    .resolution_free_casts
                    .as_ref()
                    .unwrap()
                    .remaining_mana_value,
                Some(6)
            );
            g.restore(&saved).unwrap();
            assert!(g.revision() > revision);
            assert_eq!(
                g.apply_index(player, revision, 0),
                Err(Error::StaleRevision)
            );
        }
    }
}
