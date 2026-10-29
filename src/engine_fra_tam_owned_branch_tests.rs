//! Owned branching at real Tam activation and repeated proliferate decisions.
use super::*;

#[test]
fn tam_owned_raw_index_restore_and_parent_disposal() {
    for player in [Player::P0, Player::P1] {
        let mut parent = scenario(player);
        let tam = put(&mut parent, player, "Tam, the Possibility", Zone::Battlefield);
        for name in ["Ajani, Caller of the Pride", "Kaito, Cunning Infiltrator"] {
            let walker = put(&mut parent, player, name, Zone::Battlefield);
            parent.state.objects[walker as usize].counters.loyalty = 3;
        }
        let creature = put(&mut parent, player, "Bear Cub", Zone::Battlefield);
        parent.add_counter(creature, CounterKind::PlusOne, 1);
        parent.state.players[player.index()].mana = [1, 1, 1, 1, 1, 0];
        let activate = Action::AnnounceActivate { card: tam, ability: 0, counter_payment: Counters::default() };
        let index = parent.legal_actions().iter().position(|a| *a == activate).unwrap();
        let revision = parent.revision();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        drop(parent);
        raw.apply(player, activate).unwrap();
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        pay(&mut raw);
        pay(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        for _ in 0..40 {
            if matches!(raw.state.pending, Pending::Proliferate { .. }) { break; }
            act(&mut raw, Action::Pass);
            act(&mut indexed, Action::Pass);
        }
        assert!(matches!(raw.state.pending, Pending::Proliferate { player: p } if p == player));
        let checkpoint = raw.snapshot();
        let object = raw.object_ref(creature);
        let selection = Action::ProliferatePermanent { object };
        let selection_revision = raw.revision();
        let selection_index = raw.legal_actions().iter().position(|a| *a == selection).unwrap();
        assert!(raw.apply(player.other(), selection.clone()).is_err());
        assert_eq!(raw.snapshot(), checkpoint);
        raw.apply(player, selection.clone()).unwrap();
        indexed.apply_index(player, selection_revision, selection_index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        let partial = raw.snapshot();
        let finish = Action::FinishProliferate;
        act(&mut raw, finish.clone());
        act(&mut indexed, finish.clone());
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.counter_count(creature, CounterKind::PlusOne), 2);
        assert!(matches!(raw.state.pending, Pending::Proliferate { player: p } if p == player));
        let saved_revision = raw.revision();
        let mut replay = raw.clone();
        drop(raw);
        let object = replay.object_ref(creature);
        act(&mut replay, Action::ProliferatePermanent { object });
        act(&mut replay, Action::FinishProliferate);
        let expected = replay.snapshot();
        assert_eq!(replay.counter_count(creature, CounterKind::PlusOne), 3);
        replay.restore(&partial).unwrap();
        assert!(replay.revision() > saved_revision);
        assert!(replay.apply_index(player, selection_revision, selection_index).is_err());
        act(&mut replay, finish);
        let object = replay.object_ref(creature);
        act(&mut replay, Action::ProliferatePermanent { object });
        act(&mut replay, Action::FinishProliferate);
        assert_eq!(replay.snapshot(), expected);
        for viewer in [Player::P0, Player::P1] {
            let value = serde_json::to_value(replay.observe(viewer)).unwrap();
            assert_eq!(value["viewer"], serde_json::to_value(viewer).unwrap());
        }
    }
}
