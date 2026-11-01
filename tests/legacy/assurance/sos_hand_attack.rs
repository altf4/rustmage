//! Paid partial targets, private permanent placement, and exact temporary attack grants.
use super::big_helpers::{cast as paid, checked, settle as finish};
use super::*;
fn pause(g: &mut Game) {
    for _ in 0..20 {
        if g.state.pending != Pending::Priority {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("choice expected");
}
#[test]
fn sos_hand_attack_render_reveals_nonlands_and_keeps_optional_partial_targets_independent() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..4 {
            let mut g = scenario(p);
            let victim = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
            let land = put(&mut g, p.other(), "Forest", Zone::Hand);
            let body = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let own = put(&mut g, p, "Shock", Zone::Hand);
            if mode == 3 {
                g.move_object(victim, Zone::Library);
            }
            let spell = put(&mut g, p, "Render Speechless", Zone::Hand);
            let mut targets = vec![Target::Player(p.other())];
            if mode != 0 {
                targets.push(Target::Card(body));
            }
            assert!(
                g.observe(p)
                    .objects
                    .iter()
                    .all(|o| o.id != victim && o.id != land)
            );
            paid(&mut g, spell, None, &targets);
            if mode == 2 {
                g.move_object(body, Zone::Hand);
                g.move_object(body, Zone::Battlefield);
            }
            if mode != 3 {
                pause(&mut g);
                assert_eq!(g.actor(), Some(p));
                assert!(g.observe(p).objects.iter().any(|o| o.id == victim));
                assert!(g.observe(p).objects.iter().any(|o| o.id == land));
                for id in [land, own] {
                    let object = g.object_ref(id);
                    reject(&mut g, Action::ChooseRevealedDiscard { object });
                }
                let object = g.object_ref(victim);
                checked(&mut g, Action::ChooseRevealedDiscard { object });
            }
            finish(&mut g);
            assert_eq!(
                g.counter_count(body, CounterKind::PlusOne),
                if mode == 1 || mode == 3 { 2 } else { 0 }
            );
            assert_eq!(
                g.object_view(victim).zone,
                if mode == 3 {
                    Zone::Library
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}
#[test]
fn sos_hand_attack_mind_x_private_any_permanent_tapped_entry_and_decline() {
    for p in [Player::P0, Player::P1] {
        for (x, name) in [
            (0, "Island"),
            (2, "Goldvein Pick"),
            (2, "Pacifism"),
            (3, "Adaptive Automaton"),
        ] {
            for accept in [false, true] {
                let mut g = scenario(p);
                let body = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                let chosen = put(&mut g, p, name, Zone::Hand);
                let bad = put(&mut g, p, "Shock", Zone::Hand);
                let expensive = put(&mut g, p, "Colossus of the Blood Age", Zone::Hand);
                let foreign = put(&mut g, p.other(), "Island", Zone::Hand);
                let spell = put(&mut g, p, "Mind into Matter", Zone::Hand);
                let hand = g.hand_count(p) - 1;
                checked(&mut g, Action::AnnounceCast { card: spell });
                checked(&mut g, Action::ChooseX { value: x });
                pay(&mut g);
                pause(&mut g);
                assert_eq!(g.hand_count(p), hand + x as usize);
                assert!(g.observe(p.other()).effect_choice.is_none());
                for id in [bad, expensive, foreign] {
                    let object = g.object_ref(id);
                    reject(&mut g, Action::PutHandPermanent { object });
                }
                if accept {
                    let object = g.object_ref(chosen);
                    checked(&mut g, Action::PutHandPermanent { object });
                    if name == "Adaptive Automaton" {
                        let value = EntryChoiceValue::CreatureType(
                            crate::cards::CreatureType::from_name("Robot").unwrap(),
                        );
                        checked(&mut g, Action::ChooseEntryValue { value });
                    }
                    if name == "Pacifism" {
                        let object = g.object_ref(body);
                        checked(&mut g, Action::ChooseAuraAttachment { object });
                    }
                } else {
                    checked(&mut g, Action::DeclineHandEntry);
                }
                finish(&mut g);
                assert_eq!(
                    g.object_view(chosen).zone,
                    if accept {
                        Zone::Battlefield
                    } else {
                        Zone::Hand
                    }
                );
                if accept {
                    assert!(g.object_view(chosen).tapped);
                    assert!(!g.object_view(chosen).attacking);
                }
                if accept && name == "Pacifism" {
                    assert_eq!(g.object_view(chosen).attached_to, Some(g.object_ref(body)));
                }
            }
        }
        // Even a completely ineligible hidden hand presents the same optional phase.
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Mind into Matter", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseX { value: 0 });
        pay(&mut g);
        pause(&mut g);
        assert!(g.observe(p.other()).effect_choice.is_none());
        checked(&mut g, Action::DeclineHandEntry);
        finish(&mut g);
    }
}
#[test]
fn sos_hand_attack_root_freezes_recipients_stacks_triggers_and_expires() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let c = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        for _ in 0..2 {
            let spell = put(&mut g, p, "Root Manipulation", Zone::Hand);
            paid(&mut g, spell, None, &[]);
            finish(&mut g);
        }
        for id in [a, b] {
            assert_eq!(stats(&g, id), (6, 6));
            assert!(g.keywords(id).contains(Keywords::MENACE));
        }
        assert_eq!(stats(&g, c), (2, 2));
        assert!(!g.keywords(c).contains(Keywords::MENACE));
        let newcomer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert_eq!(stats(&g, newcomer), (2, 2));
        g.move_object(b, Zone::Hand);
        g.move_object(b, Zone::Battlefield);
        assert_eq!(stats(&g, b), (2, 2));
        assert!(!g.keywords(b).contains(Keywords::MENACE));
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: a });
        checked(&mut g, Action::Attack { card: newcomer });
        checked(&mut g, Action::FinishAttackers);
        assert_eq!(
            g.state
                .stack
                .iter()
                .filter(|s| s.source.unwrap().card == Card::from_name("Root Manipulation").unwrap())
                .count()
                + g.state.ordering_triggers.len(),
            2
        );
        g.move_object(a, Zone::Exile);
        finish(&mut g);
        assert_eq!(
            g.state.players[p.index()].life,
            22,
            "each granted attack ability survives recipient departure"
        );
        g.finish_cleanup();
        assert!(g.observe(p).temporary_abilities.is_empty());
        assert_eq!(stats(&g, b), (2, 2));
    }
}
