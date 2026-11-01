//! Focused printed outcomes; shared engine invariants live in core tests.
//! Real casts count all earlier spells; the independent trigger owns its source spell packet.
use super::big_helpers::{cast as paid, checked, settle as finish};
use super::*;
fn pause(g: &mut Game) {
    for _ in 0..30 {
        if g.state.pending != Pending::Priority {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("choice expected");
}
fn storm(g: &Game) -> &SpellView {
    g.state
        .stack
        .iter()
        .chain(&g.state.pending_triggers)
        .chain(&g.state.ordering_triggers)
        .find(|s| {
            s.copy_event.is_some()
                && s.trigger_definition() == Card::from_name("Prismari, the Inspiration").unwrap()
        })
        .unwrap()
}
#[test]
fn sos_storm_counts_both_players_countered_creatures_and_never_copies_as_casts() {
    {
        let p = Player::P0;
        {
            let mut g = scenario(p);
            let bear = put(&mut g, p, "Bear Cub", Zone::Hand);
            paid(&mut g, bear, None, &[]);
            checked(&mut g, Action::Pass);
            let negate = put(&mut g, p.other(), "Essence Scatter", Zone::Hand);
            paid(&mut g, negate, None, &[Target::Spell(bear)]);
            finish(&mut g);
            assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
            g.state.priority = p;
            let provider = put(&mut g, p, "Prismari, the Inspiration", Zone::Battlefield);
            let permission = g.object_ref(provider);
            let shock = put(&mut g, p, "Shock", Zone::Hand);
            paid(&mut g, shock, None, &[Target::Player(p.other())]);
            let packet = storm(&g);
            assert_eq!(packet.source.unwrap().id, shock);
            assert_eq!(packet.copy_event.as_ref().unwrap().amount, 2);
            assert_eq!(
                packet.attachment_trigger.as_ref().unwrap().grantor,
                permission
            );
            assert_eq!(
                packet.attachment_trigger.as_ref().unwrap().recipient,
                g.object_ref(shock)
            );
            checked(&mut g, Action::Pass);
            let negate = put(&mut g, p.other(), "Negate", Zone::Hand);
            paid(&mut g, negate, None, &[Target::Spell(shock)]);

            finish(&mut g);
            assert_eq!(g.state.players[p.other().index()].life, 16);
            assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 2);
            assert_eq!(g.state.players[p.other().index()].spells_cast_this_turn, 2);
            {
                g.state.priority = p;
                let shock = put(&mut g, p, "Shock", Zone::Hand);
                paid(&mut g, shock, None, &[Target::Player(p.other())]);
                assert_eq!(storm(&g).copy_event.as_ref().unwrap().amount, 4);
                finish(&mut g);
                assert_eq!(g.state.players[p.other().index()].life, 6);
            }
        }
    }
}

#[test]
fn sos_storm_frozen_x_and_owned_retarget_preserve_mana_value_but_not_actual_payment() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let body = put(&mut g, p, "Bear Cub", Zone::Hand);
        paid(&mut g, body, None, &[]);
        finish(&mut g);
        let enemy = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        put(&mut g, p, "Prismari, the Inspiration", Zone::Battlefield);
        let spell = put(&mut g, p, "Fractalize", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseX { value: 2 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(body),
            },
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        assert_eq!(storm(&g).copy_event.as_ref().unwrap().amount, 1);
        pause(&mut g);
        let view = g.observe(p).retarget.unwrap();
        assert_eq!(view.item.x, Some(2));
        assert_eq!(view.item.mana_value(), 3);
        assert_eq!(view.item.cast.unwrap().payment, [0; 6]);
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(enemy),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        finish(&mut g);
        assert_eq!(stats(&g, enemy), (3, 3));
        assert_eq!(stats(&g, body), (3, 3));
        assert!(g.has_subtype(enemy, "Fractal"));
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 2);
    }
}

#[test]
fn sos_storm_cannot_copy_choreographed_sparks_and_does_not_recast_copies() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Prismari, the Inspiration", Zone::Battlefield);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        paid(&mut g, shock, None, &[Target::Player(p.other())]);
        let sparks = put(&mut g, p, "Choreographed Sparks", Zone::Hand);
        paid(&mut g, sparks, Some(0), &[Target::Spell(shock)]);
        let definition = Card::from_name("Choreographed Sparks").unwrap();
        for _ in 0..100 {
            assert!(
                !g.state
                    .stack
                    .iter()
                    .any(|s| s.is_copy && s.source.unwrap().card == definition)
            );
            if g.state.stack.is_empty()
                && g.state.resolving.is_none()
                && g.state.pending_triggers.is_empty()
            {
                break;
            }
            let actions = g.legal_actions();
            let a = if g.state.pending == Pending::Priority {
                Action::Pass
            } else if actions.contains(&Action::FinishStackTargets) {
                Action::FinishStackTargets
            } else {
                actions
                    .into_iter()
                    .find(|a| !matches!(a, Action::Concede))
                    .unwrap()
            };
            checked(&mut g, a);
        }
        assert!(g.state.stack.is_empty());
        assert_eq!(g.state.players[p.other().index()].life, 16);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 2);
    }
}
