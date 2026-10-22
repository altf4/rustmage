//! Paid first-year student interactions with owned raw/index branching and restore.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::AnimationDuration;
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn pay(g: &mut Game) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, action);
}
fn activate(g: &mut Game, source: CardId, target: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(g);
}
#[test]
fn ecl_student_tam_both_hybrid_payments_tap_cost_self_target_and_source_departure() {
    for p in [Player::P0, Player::P1] {
        for cost in [0, 1] {
            let mut g = setup(p);
            let tam = named(&mut g, p, "Tam, Mindful First-Year", Zone::Hand);
            let mut mana = [0; 6];
            mana[if cost == 0 { 4 } else { 1 }] = 1;
            mana[5] = 1;
            g.state.players[p.index()].mana = mana;
            checked(&mut g, Action::AnnounceCast { card: tam });
            checked(&mut g, Action::ChooseSpellCost { cost });
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(
                g.stats(tam),
                Stats {
                    power: 2,
                    toughness: 2
                }
            );
            assert_eq!(g.colors(tam), 18);
            let action = Action::AnnounceActivate {
                card: tam,
                ability: 0,
                counter_payment: Counters::default(),
            };
            reject(&mut g, action);
            g.state.objects[tam as usize].summoning_sick = false;
            activate(&mut g, tam, tam);
            assert!(g.object_view(tam).tapped);
            settle(&mut g);
            assert_eq!(g.colors(tam), 31);
            assert!(!g.keywords(tam).contains(Keywords::HEXPROOF_OWN_COLORS));
            g.state.objects[tam as usize].tapped = false;
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: tam,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(foe),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            pay(&mut g);
            g.move_object(tam, Zone::Exile);
            settle(&mut g);
            assert_eq!(g.colors(bear), 31);
            assert!(!g.keywords(bear).contains(Keywords::HEXPROOF_OWN_COLORS));
            assert!(g.expire_color_effects());
            assert_eq!(g.colors(bear), 16);
        }
    }
}
#[test]
fn ecl_student_tam_dynamic_own_color_hexproof_filters_opponent_sources_only() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let tam = named(&mut g, p, "Tam, Mindful First-Year", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let sources = ["Pacifism", "Opt", "Duress", "Shock", "Snakeskin Veil"]
            .map(|name| named(&mut g, p.other(), name, Zone::Hand));
        assert!(g.keywords(bear).contains(Keywords::HEXPROOF_OWN_COLORS));
        assert!(!g.keywords(tam).contains(Keywords::HEXPROOF_OWN_COLORS));
        assert!(!g.keywords(foreign).contains(Keywords::HEXPROOF_OWN_COLORS));
        for colors in [0, 1, 2, 4, 8, 16, 31] {
            g.set_color_effects(&[bear], colors, AnimationDuration::Permanent);
            for source in sources {
                let reference = g.object_ref(source);
                let source_colors = g.state.objects[source as usize].card.colors();
                assert_eq!(
                    g.cast_target_matches(
                        TargetRule::Creature,
                        p.other(),
                        reference,
                        Target::Card(bear)
                    ),
                    colors & source_colors == 0
                );
                assert!(g.cast_target_matches(
                    TargetRule::Creature,
                    p,
                    reference,
                    Target::Card(bear)
                ));
                assert!(g.cast_target_matches(
                    TargetRule::Creature,
                    p.other(),
                    reference,
                    Target::Card(tam)
                ));
            }
        }
        checked(&mut g, Action::Pass);
    }
}
#[test]
fn ecl_student_tam_paid_response_makes_previously_legal_shock_target_illegal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let tam = named(&mut g, p, "Tam, Mindful First-Year", Zone::Battlefield);
        g.state.objects[tam as usize].summoning_sick = false;
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let shock = named(&mut g, p.other(), "Shock", Zone::Hand);
        g.state.priority = p.other();
        cast(&mut g, shock, None, &[Target::Card(bear)]);
        checked(&mut g, Action::Pass);
        activate(&mut g, tam, bear);
        settle(&mut g);
        assert_eq!(g.colors(bear), 31);
        assert!(g.on_battlefield(bear));
        assert_eq!(g.object_view(bear).damage, 0);
        assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
    }
}
#[test]
fn ecl_student_tam_source_and_recipient_ability_removal_and_control_changes_use_timeline() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let tam = named(&mut g, p, "Tam, Mindful First-Year", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let stop = named(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, stop, None, &[Target::Card(tam)]);
        settle(&mut g);
        assert!(!g.keywords(bear).contains(Keywords::HEXPROOF_OWN_COLORS));
        g.move_object(stop, Zone::Graveyard);
        assert!(g.keywords(bear).contains(Keywords::HEXPROOF_OWN_COLORS));
        let witness = named(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, witness, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert!(!g.keywords(bear).contains(Keywords::HEXPROOF_OWN_COLORS));
        g.move_object(tam, Zone::Exile);
        g.move_object(tam, Zone::Battlefield);
        assert!(g.keywords(bear).contains(Keywords::HEXPROOF_OWN_COLORS));
        let offering = named(&mut g, p, "Harmless Offering", Zone::Hand);
        cast(
            &mut g,
            offering,
            None,
            &[Target::Player(p.other()), Target::Card(tam)],
        );
        settle(&mut g);
        assert!(!g.keywords(bear).contains(Keywords::HEXPROOF_OWN_COLORS));
        assert!(g.keywords(foreign).contains(Keywords::HEXPROOF_OWN_COLORS));
        assert!(!g.keywords(tam).contains(Keywords::HEXPROOF_OWN_COLORS));
    }
}
#[test]
fn ecl_student_tam_departed_ability_source_uses_frozen_colors_without_rebinding_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let tam = named(&mut g, p, "Tam, Mindful First-Year", Zone::Battlefield);
        g.state.objects[tam as usize].summoning_sick = false;
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let firebrand = named(&mut g, p.other(), "Fanatical Firebrand", Zone::Battlefield);
        let old = g.object_ref(firebrand);
        g.state.priority = p.other();
        activate(&mut g, firebrand, bear);
        assert_eq!(g.object_view(firebrand).zone, Zone::Graveyard);
        g.move_object(firebrand, Zone::Battlefield);
        g.set_color_effects(&[firebrand], 0, AnimationDuration::Permanent);
        assert_ne!(g.object_ref(firebrand), old);
        checked(&mut g, Action::Pass);
        activate(&mut g, tam, bear);
        settle(&mut g);
        assert_eq!(g.colors(firebrand), 0);
        assert_eq!(g.colors(bear), 31);
        assert_eq!(g.object_view(bear).damage, 0);
    }
}
