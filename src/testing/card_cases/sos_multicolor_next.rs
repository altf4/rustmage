//! Focused printed outcomes; shared engine invariants live in core tests.
//! Printed outcomes for the parked next SOS multicolor batch.
use super::big_helpers::{checked, settle as finish};
use super::*;

#[test]
fn sos_next_berta_increment_counter_events_and_stack_mana() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let berta = put(&mut g, p, "Berta, Wise Extrapolator", Zone::Battlefield);
        let growth = put(&mut g, p, "Growth Curve", Zone::Hand);
        let before: u32 = g.state.players[p.index()].mana.iter().sum();
        checked(&mut g, Action::AnnounceCast { card: growth });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(berta),
            },
        );
        pay(&mut g);
        // The counter trigger is an ordinary stack ability. Mana is not immediate.
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            before - 2
        );
        finish(&mut g);
        assert_eq!(g.counter_count(berta, CounterKind::PlusOne), 4);
        // Increment, the first Growth Curve counter, then its doubling: three events.
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            before + 1
        );
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Card(berta)));
        finish(&mut g);
        assert_eq!(g.counter_count(berta, CounterKind::PlusOne), 4);
        // An opponent can place the counters and the source's controller still gets mana.
        g.state.priority = p.other();
        let growth = put(&mut g, p.other(), "Silverquill Charm", Zone::Hand);
        let before: u32 = g.state.players[p.index()].mana.iter().sum();
        checked(&mut g, Action::AnnounceCast { card: growth });
        checked(&mut g, Action::ChooseMode { mode: 0 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(berta),
            },
        );
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.counter_count(berta, CounterKind::PlusOne), 6);
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            before + 1
        );
    }
}
#[test]
fn sos_next_berta_paid_x_fractal_and_zero_sba() {
    {
        let p = Player::P0;
        for x in [0, 3] {
            let mut g = scenario(p);
            let berta = put(&mut g, p, "Berta, Wise Extrapolator", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: berta,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            checked(&mut g, Action::ChooseX { value: x });
            pay(&mut g);
            finish(&mut g);
            assert!(g.object_view(berta).tapped);
            assert_eq!(g.state.players[p.index()].mana[5], 3 - x);
            let tokens: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(_, o)| o.zone == Zone::Battlefield && o.card == Card::SosFractalToken)
                .map(|(i, _)| i as CardId)
                .collect();
            assert_eq!(tokens.len(), usize::from(x != 0));
            if let Some(&token) = tokens.first() {
                assert_eq!(stats(&g, token), (i128::from(x), i128::from(x)));
            }
        }
    }
}
#[test]
fn sos_next_lorehold_charm_all_modes_and_choices() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let first = put(&mut g, p.other(), "Adventuring Gear", Zone::Battlefield);
        let second = put(&mut g, p.other(), "Basilisk Collar", Zone::Battlefield);
        let treasure = put(
            &mut g,
            p.other(),
            "@token/b60f6cd3b44b22bd289fe4e2",
            Zone::Battlefield,
        );
        let own = put(&mut g, p, "Adventuring Gear", Zone::Battlefield);
        let spell = put(&mut g, p, "Lorehold Charm", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseMode { mode: 0 });
        pay(&mut g);
        for _ in 0..10 {
            if g.state.pending != Pending::Priority {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        assert_eq!(g.actor(), Some(p.other()));
        let bad = g.object_ref(treasure);
        reject(&mut g, Action::Sacrifice { object: bad });
        let bad = g.object_ref(own);
        reject(&mut g, Action::Sacrifice { object: bad });
        let chosen = g.object_ref(second);
        checked(&mut g, Action::Sacrifice { object: chosen });
        finish(&mut g);
        assert_eq!(g.object_view(second).zone, Zone::Graveyard);
        for id in [first, treasure, own] {
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        }
        let high = put(&mut g, p, "Hill Giant", Zone::Graveyard);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
        let bear = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        g.state.priority = p;
        let spell = put(&mut g, p, "Lorehold Charm", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseMode { mode: 1 });
        for bad in [high, enemy] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bad),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        let spell = put(&mut g, p, "Lorehold Charm", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseMode { mode: 2 });
        pay(&mut g);
        finish(&mut g);
        assert_eq!(stats(&g, bear), (3, 3));
        assert!(g.keywords(bear).contains(Keywords::TRAMPLE));
        let later = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert_eq!(stats(&g, later), (2, 2));
        assert!(!g.keywords(later).contains(Keywords::TRAMPLE));
        g.finish_cleanup();
        assert_eq!(stats(&g, bear), (2, 2));
        assert!(!g.keywords(bear).contains(Keywords::TRAMPLE));
    }
}
#[test]
fn sos_next_blech_union_one_per_event_and_controller() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let mut own = vec![put(&mut g, p, "Blech, Loafing Pest", Zone::Battlefield)];
        for name in [
            "Dreaded Bat-Cloud",
            "Aatchik, Emerald Radian",
            "Ankle Biter",
            "Attercop",
            "Three Tree Mascot",
        ] {
            own.push(put(&mut g, p, name, Zone::Battlefield));
        }
        let other = put(&mut g, p.other(), "Three Tree Mascot", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.gain_life(p, 5);
        finish(&mut g);
        for &id in &own {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        }
        for id in [other, bear] {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
        }
        g.gain_life(p, 0);
        g.gain_life(p.other(), 4);
        finish(&mut g);
        for &id in &own {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        }
        g.gain_life(p, 1);
        let source = own[0];
        g.move_object(source, Zone::Graveyard);
        finish(&mut g);
        for &id in &own[1..] {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 2);
        }
    }
}

#[test]
fn sos_next_paintmage_first_main_restricted_mana_and_hybrid_costs() {
    {
        let p = Player::P0;
        for blue in [false, true] {
            let mut g = scenario(p);
            g.state.players[p.index()].mana = if blue {
                [0, 2, 0, 1, 0, 0]
            } else {
                [0, 1, 0, 2, 0, 0]
            };
            let mage = put(&mut g, p, "Abstract Paintmage", Zone::Hand);
            cast(&mut g, mage, None);
            finish(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            g.begin_step(Step::Main1);
            finish(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0, 1, 0, 1, 0, 0]);
            let turtle = put(&mut g, p, "Aegis Turtle", Zone::Hand);
            reject(&mut g, Action::AnnounceCast { card: turtle });
            g.move_object(mage, Zone::Graveyard);
            let spell = put(&mut g, p, "Prismari Charm", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: spell });
            checked(&mut g, Action::ChooseMode { mode: 0 });
            pay(&mut g);
            finish(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            g.begin_step(Step::Main2);
            finish(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn sos_next_duelist_draw_life_and_optional_repartee_delayed_return() {
    {
        let p = Player::P0;
        for accept in [false, true] {
            let mut g = scenario(p);
            let duelist = put(&mut g, p, "Conciliator's Duelist", Zone::Hand);
            cast(&mut g, duelist, None);
            finish(&mut g);
            assert_eq!(g.state.players[p.index()].life, 19);
            assert_eq!(g.state.players[p.other().index()].life, 19);
            assert_eq!(g.hand_count(p), 1);
            assert_eq!(stats(&g, duelist), (4, 3));
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let growth = put(&mut g, p, "Growth Curve", Zone::Hand);
            cast(&mut g, growth, Some(Target::Card(own)));
            for _ in 0..12 {
                if g.state.pending != Pending::Priority {
                    break;
                }
                checked(&mut g, Action::Pass);
            }
            if accept {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
            } else {
                checked(&mut g, Action::FinishTargets);
            }
            finish(&mut g);
            assert_eq!(
                g.object_view(enemy).zone,
                if accept {
                    Zone::Exile
                } else {
                    Zone::Battlefield
                }
            );
            g.move_object(duelist, Zone::Graveyard);
            g.begin_step(Step::End);
            finish(&mut g);
            assert_eq!(g.object_view(enemy).zone, Zone::Battlefield);
            assert_eq!(g.object_view(enemy).controller, p.other());
        }
    }
}
