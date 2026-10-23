//! Actual hybrid payments and persistent evolution/protection interactions.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;
fn paid(g: &mut Game, p: Player, white: bool) -> CardId {
    let id = named(g, p, "Figure of Fable", Zone::Hand);
    let mut payment = [0; 6];
    payment[if white { 0 } else { 4 }] = 1;
    g.state.players[p.index()].mana = payment;
    checked(g, Action::AnnounceCast { card: id });
    checked(
        g,
        Action::ChooseSpellCost {
            cost: u16::from(white),
        },
    );
    checked(g, Action::PayCosts { payment });
    settle(g);
    id
}
fn evolve(g: &mut Game, id: CardId, stage: u16, white: u16) {
    let p = g.state.objects[id as usize].controller;
    g.state.priority = p;
    let count = stage + 1;
    let generic = [0, 1, 3][stage as usize];
    let mut payment = [0; 6];
    payment[0] = u32::from(white);
    payment[4] = u32::from(count - white);
    payment[5] = generic;
    g.state.players[p.index()].mana = payment;
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: stage,
            counter_payment: Counters::default(),
        },
    );
    checked(g, Action::ChooseActivationCost { cost: white });
    checked(g, Action::PayCosts { payment });
    settle(g);
}
fn avatar(g: &mut Game, p: Player) -> CardId {
    let id = paid(g, p, false);
    evolve(g, id, 0, 0);
    evolve(g, id, 1, 0);
    evolve(g, id, 2, 0);
    id
}
#[test]
fn ecl_figure_all_paid_hybrid_stages_both_seats_and_persistent_downgrade() {
    for p in [Player::P0, Player::P1] {
        for a in 0..=1 {
            for b in 0..=2 {
                for c in 0..=3 {
                    let mut g = setup(p);
                    let id = paid(&mut g, p, a == 1);
                    evolve(&mut g, id, 0, a);
                    assert_eq!(
                        g.stats(id),
                        Stats {
                            power: 2,
                            toughness: 3
                        }
                    );
                    assert!(g.has_subtype(id, "Scout"));
                    evolve(&mut g, id, 1, b);
                    assert_eq!(
                        g.stats(id),
                        Stats {
                            power: 4,
                            toughness: 5
                        }
                    );
                    assert!(!g.has_subtype(id, "Scout"));
                    assert!(g.has_subtype(id, "Soldier"));
                    evolve(&mut g, id, 2, c);
                    assert_eq!(
                        g.stats(id),
                        Stats {
                            power: 7,
                            toughness: 8
                        }
                    );
                    assert!(g.has_subtype(id, "Avatar"));
                    assert!(!g.has_subtype(id, "Soldier"));
                    assert!(g.keywords(id).contains(Keywords::PROTECTION_OPPONENTS));
                    g.finish_cleanup();
                    assert_eq!(g.stats(id).power, 7);
                    evolve(&mut g, id, 0, 0);
                    assert_eq!(g.stats(id).power, 2);
                    assert!(g.keywords(id).contains(Keywords::PROTECTION_OPPONENTS));
                }
            }
        }
    }
}
#[test]
fn ecl_figure_prerequisites_checked_at_resolution_and_exact_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, false);
        evolve(&mut g, id, 1, 1);
        assert_eq!(g.stats(id).power, 1);
        evolve(&mut g, id, 2, 2);
        assert_eq!(g.stats(id).power, 1);
        assert!(!g.keywords(id).contains(Keywords::PROTECTION_OPPONENTS));
        // Paid higher stage can resolve after a lower stage resolves above it.
        g.state.players[p.index()].mana = [0, 0, 0, 0, 20, 0];
        g.state.priority = p;
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: id,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        checked(&mut g, Action::ChooseActivationCost { cost: 0 });
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 3, 0],
            },
        );
        evolve(&mut g, id, 0, 0);
        assert_eq!(g.stats(id).power, 4);
        // Leaving invalidates the ability's exact source, even when the same card returns.
        g.state.priority = p;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 6, 0];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: id,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        checked(&mut g, Action::ChooseActivationCost { cost: 0 });
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 6, 0],
            },
        );
        g.move_object(id, Zone::Hand);
        g.move_object(id, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(
            g.stats(id),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert!(!g.keywords(id).contains(Keywords::PROTECTION_OPPONENTS));
    }
}
#[test]
fn ecl_figure_opponent_protection_damage_and_colorless_blocks() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = avatar(&mut g, p);
        let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Rooftop Percher", Zone::Battlefield);
        for (source, preventable, damage) in [(own, true, 1), (enemy, true, 1), (enemy, false, 2)] {
            let source = g.damage_source(source);
            g.apply_damage_batch(&[DamageEvent {
                source,
                target: Target::Card(id),
                amount: 1,
                combat: false,
                preventable,
            }]);
            while g.advance_replacements(false) {
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::ChooseReplacement { .. }))
                    .unwrap();
                checked(&mut g, a);
            }
            assert_eq!(g.object_view(id).damage, damage);
        }
        g.state.objects[id as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: id });
        checked(&mut g, Action::FinishAttackers);
        settle(&mut g);
        g.state.step = Step::Blockers;
        g.state.pending = Pending::Blockers;
        assert!(!g.can_block_pair(enemy, id));
    }
}
#[test]
fn ecl_figure_own_targets_attachments_and_protection_tracks_new_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = avatar(&mut g, p);
        g.state.players[p.index()].mana = [30; 6];
        g.state.priority = p;
        let aura = named(&mut g, p, "Blossombind", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(id)]);
        settle(&mut g);
        assert_eq!(g.state.objects[aura as usize].zone, Zone::Battlefield);
        let old_source = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let offering = named(&mut g, p, "Harmless Offering", Zone::Hand);
        g.state.priority = p;
        cast(
            &mut g,
            offering,
            None,
            &[Target::Player(p.other()), Target::Card(id)],
        );
        settle(&mut g);
        assert_eq!(g.state.objects[id as usize].controller, p.other());
        assert_eq!(g.state.objects[aura as usize].zone, Zone::Graveyard);
        let source = g.damage_source(old_source);
        g.apply_damage_batch(&[DamageEvent {
            source,
            target: Target::Card(id),
            amount: 2,
            combat: false,
            preventable: true,
        }]);
        while g.advance_replacements(false) {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::ChooseReplacement { .. }))
                .unwrap();
            checked(&mut g, a);
        }
        assert_eq!(g.object_view(id).damage, 0);
    }
}
#[test]
fn ecl_figure_protection_rejects_opponents_targets_but_keeps_own_growth_and_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = avatar(&mut g, p);
        g.set_counters(id, Counters::of(CounterKind::PlusOne, 2));
        assert_eq!(
            g.stats(id),
            Stats {
                power: 9,
                toughness: 10
            }
        );
        g.state.players[p.index()].mana = [30; 6];
        g.state.priority = p;
        let growth = named(&mut g, p, "Giant Growth", Zone::Hand);
        cast(&mut g, growth, None, &[Target::Card(id)]);
        settle(&mut g);
        assert_eq!(
            g.stats(id),
            Stats {
                power: 12,
                toughness: 13
            }
        );
        let aura = named(&mut g, p, "Noggle the Mind", Zone::Hand);
        g.state.priority = p;
        cast(&mut g, aura, None, &[Target::Card(id)]);
        settle(&mut g);
        assert!(!g.keywords(id).contains(Keywords::PROTECTION_OPPONENTS));
        g.move_object(aura, Zone::Graveyard);
        settle(&mut g);
        assert!(g.keywords(id).contains(Keywords::PROTECTION_OPPONENTS));
        let other = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let bounce = named(&mut g, p.other(), "Unsummon", Zone::Hand);
        g.state.priority = p.other();
        checked(&mut g, Action::AnnounceCast { card: bounce });
        let before = g.snapshot();
        assert_eq!(
            g.apply(
                p.other(),
                Action::ChooseTarget {
                    target: Target::Card(id)
                }
            ),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), before);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(other),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        assert_eq!(g.state.objects[other as usize].zone, Zone::Hand);
        assert!(g.on_battlefield(id));
        g.finish_cleanup();
        assert_eq!(
            g.stats(id),
            Stats {
                power: 9,
                toughness: 10
            }
        );
    }
}
