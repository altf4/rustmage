use super::*;

fn announce_vanguard(g: &mut Game, action: Action, target: Option<CardId>) {
    act(g, action);
    for _ in 0..32 {
        if g.state.announcing.is_none() {
            return;
        }
        choose(g, |a| match a {
            Action::ChooseTarget {
                target: Target::Card(card),
            } => Some(*card) == target,
            Action::ChooseSpellCost { .. } | Action::FinishTargets | Action::PayCosts { .. } => {
                true
            }
            _ => false,
        });
    }
    panic!("Vanguard announcement did not finish");
}

#[test]
fn vanguard_hybrid_faces_prepare_cast_and_self_vigilance() {
    for p in [Player::P0, Player::P1] {
        for white in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Vigorbloom Vanguard // Seed Suture", Zone::Hand);
            g.state.players[p.index()].mana = if white {
                [1, 0, 0, 0, 0, 1]
            } else {
                [0, 0, 0, 0, 1, 1]
            };
            announce_vanguard(&mut g, Action::AnnounceCast { card: source }, None);
            settle(&mut g);
            assert_eq!(stats(&g, source), (2, 2));
            assert!(!g.keywords(source).contains(Keywords::VIGILANCE));
            let permanent = g.object_ref(source);
            let prepared = g
                .state
                .preparations
                .as_ref()
                .unwrap()
                .iter()
                .find(|r| r.permanent == permanent)
                .unwrap()
                .spell_copy;
            assert_eq!(
                g.state.objects[prepared.card as usize].card,
                Card::test_index(5456)
            );
            g.state.players[p.index()].mana = if white {
                [1, 0, 0, 0, 0, 0]
            } else {
                [0, 0, 0, 0, 1, 0]
            };
            let life = g.state.players[p.index()].life;
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                announce_vanguard(
                    &mut g,
                    Action::AnnounceExile {
                        card: prepared.card,
                    },
                    Some(source),
                );
                assert!(!g.is_prepared(permanent));
                settle(&mut g);
                assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
                assert!(g.keywords(source).contains(Keywords::VIGILANCE));
                assert_eq!(g.state.players[p.index()].life, life + 1);
                assert_eq!(g.object_view(prepared.card).zone, Zone::Outside);
            }
        }
    }
}

#[test]
fn vanguard_vigilance_follows_live_counters_and_controller_and_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(
            &mut g,
            p,
            "Vigorbloom Vanguard // Seed Suture",
            Zone::Battlefield,
        );
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.add_counter(source, CounterKind::PlusOne, 1);
        g.add_counter(own, CounterKind::PlusOne, 1);
        g.add_counter(enemy, CounterKind::PlusOne, 1);
        assert!(g.keywords(source).contains(Keywords::VIGILANCE));
        assert!(g.keywords(own).contains(Keywords::VIGILANCE));
        assert!(!g.keywords(enemy).contains(Keywords::VIGILANCE));
        g.change_controller(g.object_ref(own), p.other());
        assert!(!g.keywords(own).contains(Keywords::VIGILANCE));
        g.change_controller(g.object_ref(own), p);
        let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(source));
        assert!(!g.keywords(own).contains(Keywords::VIGILANCE));
        g.move_object(aura, Zone::Graveyard);
        assert!(g.keywords(own).contains(Keywords::VIGILANCE));
        g.move_object(source, Zone::Graveyard);
        assert!(!g.keywords(own).contains(Keywords::VIGILANCE));
    }
}

#[test]
fn seed_suture_fizzles_when_target_leaves_and_gains_no_life() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Vigorbloom Vanguard // Seed Suture", Zone::Hand);
        announce_vanguard(&mut g, Action::AnnounceCast { card: source }, None);
        settle(&mut g);
        let prepared = g
            .state
            .preparations
            .as_ref()
            .unwrap()
            .iter()
            .find(|r| r.permanent == g.object_ref(source))
            .unwrap()
            .spell_copy;
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let life = g.state.players[p.index()].life;
        announce_vanguard(
            &mut g,
            Action::AnnounceExile {
                card: prepared.card,
            },
            Some(target),
        );
        g.move_object(target, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life);
    }
}
