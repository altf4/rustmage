//! Paid blight choices preserve exact objects and owned continuations.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;

fn until_payment(g: &mut Game) {
    for _ in 0..30 {
        if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
            return;
        }
        let a = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PutTrigger { .. } | Action::FinishTargets))
                .unwrap()
        };
        checked(g, a);
    }
    panic!("missing mana choice");
}

#[test]
fn ecl_chaos_spewer_paid_or_blight_both_hybrid_routes_and_seats() {
    for p in [Player::P0, Player::P1] {
        for black in [false, true] {
            for pay in [false, true] {
                let mut g = setup(p);
                let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
                let foe = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
                let spewer = named(&mut g, p, "Chaos Spewer", Zone::Hand);
                let mut mana = [0; 6];
                mana[if black { 2 } else { 3 }] = 1;
                mana[5] = 2;
                g.state.players[p.index()].mana = mana;
                cast(&mut g, spewer, None, &[]);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                until_payment(&mut g);
                // The trigger is independent of its source, including a later incarnation.
                g.move_object(spewer, Zone::Exile);
                g.move_object(spewer, Zone::Battlefield);
                if pay {
                    g.state.players[p.index()].mana[5] = 2;
                    let a = g
                        .legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::PayResolutionMana { .. }))
                        .unwrap();
                    checked(&mut g, a);
                } else {
                    checked(&mut g, Action::DeclineResolutionMana);
                    assert!(
                        matches!(g.state.pending, Pending::EffectChoice { player } if player == p)
                    );
                    let invalid = Action::Blight {
                        object: g.object_ref(foe),
                    };
                    let before = g.snapshot();
                    let rev = g.revision();
                    assert_eq!(g.apply(p, invalid), Err(Error::IllegalAction));
                    assert_eq!(g.snapshot(), before);
                    assert_eq!(g.revision(), rev);
                    let object = g.object_ref(own);
                    checked(&mut g, Action::Blight { object });
                }
                settle(&mut g);
                assert_eq!(
                    g.counter_count(own, CounterKind::MinusOne),
                    if pay { 0 } else { 2 }
                );
                assert_eq!(g.counter_count(foe, CounterKind::MinusOne), 0);
                assert_eq!(g.stats(own).toughness, if pay { 3 } else { 1 });
            }
        }
    }
}

#[test]
fn ecl_mandatory_blight_without_creatures_finishes_and_source_can_blight_itself() {
    for p in [Player::P0, Player::P1] {
        for departed in [false, true] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Chaos Spewer", Zone::Hand);
            cast(&mut g, source, None, &[]);
            until_payment(&mut g);
            if departed {
                g.move_object(source, Zone::Exile);
            }
            checked(&mut g, Action::DeclineResolutionMana);
            if !departed {
                let object = g.object_ref(source);
                checked(&mut g, Action::Blight { object });
            }
            settle(&mut g);
            assert_eq!(
                g.counter_count(source, CounterKind::MinusOne),
                if departed { 0 } else { 2 }
            );
        }
    }
}
