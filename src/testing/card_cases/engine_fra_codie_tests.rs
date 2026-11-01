//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn codie_settle(g: &mut Game) {
    for _ in 0..100 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::ArrangeLibrary {
                        destination: LibraryDestination::Top,
                        ..
                    }
            )
        });
    }
    panic!("Codie resolution did not finish");
}
fn codie_announce(g: &mut Game, action: Action) {
    act(g, action);
    for _ in 0..20 {
        if g.state.announcing.is_none() {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::ChooseSpellCost { .. } | Action::FinishTargets | Action::PayCosts { .. }
            )
        });
    }
    panic!("Codie announcement did not finish");
}
fn prepared_peer(g: &mut Game, player: Player) -> ObjectRef {
    let creature = put(
        g,
        player,
        "Fatehold Chronologist // Peer Review",
        Zone::Battlefield,
    );
    let reference = g.object_ref(creature);
    g.prepare_permanent(reference);
    g.state
        .preparations
        .as_ref()
        .unwrap()
        .iter()
        .find(|row| row.permanent == reference)
        .unwrap()
        .spell_copy
}

#[test]
fn codie_prepares_only_controlled_eligible_creatures_without_duplicate_preparations() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let codie = put(&mut g, player, "Codie, Ravenous Codex", Zone::Battlefield);
        let before = prepared_peer(&mut g, player);
        let eligible = put(
            &mut g,
            player,
            "Semester Foreseer // Peer Review",
            Zone::Battlefield,
        );
        let other = put(
            &mut g,
            player.other(),
            "Prudent Fateseer // Peer Review",
            Zone::Battlefield,
        );
        let ordinary = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[codie as usize].summoning_sick = false;
        codie_announce(
            &mut g,
            Action::AnnounceActivate {
                card: codie,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        codie_settle(&mut g);
        assert!(g.is_prepared(g.object_ref(eligible)));
        assert!(!g.is_prepared(g.object_ref(other)));
        assert!(!g.is_prepared(g.object_ref(ordinary)));
        assert!(
            g.state
                .preparations
                .as_ref()
                .unwrap()
                .iter()
                .any(|row| row.spell_copy == before)
        );
        assert_eq!(g.state.preparations.as_ref().unwrap().len(), 2);
    }
}
