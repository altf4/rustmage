//! Final mono-white/blue SOS cards and their reusable rules boundaries.
use super::*;

fn pause(g: &mut Game) {
    for _ in 0..16 {
        if g.state.pending != Pending::Priority {
            return;
        }
        big_helpers::checked(g, Action::Pass);
    }
    panic!("expected a resolution choice");
}
fn paid(g: &mut Game) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    big_helpers::checked(g, action);
}
fn cast_x(g: &mut Game, spell: CardId, x: u32, target: Target) {
    big_helpers::checked(g, Action::AnnounceCast { card: spell });
    big_helpers::checked(g, Action::ChooseX { value: x });
    big_helpers::checked(g, Action::ChooseTarget { target });
    paid(g);
}
fn tester_payment(g: &mut Game, x: u32, pay: bool) {
    pause(g);
    big_helpers::checked(g, Action::ChooseOptional { accept: true });
    assert!(matches!(g.state.pending, Pending::ResolutionX { .. }));
    big_helpers::checked(g, Action::ChooseX { value: x });
    assert!(matches!(g.state.pending, Pending::ResolutionMana { .. }));
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseTarget { .. }))
    );
    let action = if pay {
        g.legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayResolutionMana { .. }))
            .unwrap()
    } else {
        Action::DeclineResolutionMana
    };
    big_helpers::checked(g, action);
}

#[test]
fn sos_final_fractalize_replaces_creature_types_colors_and_base_before_modifiers() {
    for p in [Player::P0, Player::P1] {
        for x in [0, 2] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
            g.set_counters(target, Counters::of(CounterKind::PlusOne, 1));
            let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
            big_helpers::cast(&mut g, growth, None, &[Target::Card(target)]);
            big_helpers::settle(&mut g);
            let spell = put(&mut g, p, "Fractalize", Zone::Hand);
            cast_x(&mut g, spell, x, Target::Card(target));
            big_helpers::settle(&mut g);
            let view = g.object_view(target);
            assert_eq!(view.colors, 2 | 16);
            assert!(g.has_subtype(target, "Fractal"));
            assert!(!g.has_subtype(target, "Dragon"));
            assert!(g.keywords(target).contains(Keywords::FLYING));
            assert_eq!(stats(&g, target), (i128::from(x) + 5, i128::from(x) + 5));
            assert_eq!(
                g.observe(p)
                    .animations
                    .iter()
                    .filter(|r| r.object == g.object_ref(target))
                    .count(),
                1
            );
            g.finish_cleanup();
            assert_eq!(g.object_view(target).colors, 8);
            assert!(g.has_subtype(target, "Dragon"));
            assert!(!g.has_subtype(target, "Fractal"));
            assert_eq!(stats(&g, target), (6, 6));
        }
    }
}

#[test]
fn sos_final_fractalize_preserves_artifact_keywords_and_does_not_follow_blinks() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p, "Steel Hellkite", Zone::Battlefield);
        let old = g.object_ref(target);
        let spell = put(&mut g, p, "Fractalize", Zone::Hand);
        cast_x(&mut g, spell, 2, Target::Card(target));
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(target).kinds, 1 | 16);
        assert!(g.has_subtype(target, "Fractal"));
        assert!(!g.has_subtype(target, "Dragon"));
        assert_eq!(stats(&g, target), (3, 3));
        assert!(g.keywords(target).contains(Keywords::FLYING));
        let blink = put(&mut g, p, "Daydream", Zone::Hand);
        big_helpers::cast(&mut g, blink, None, &[Target::Card(target)]);
        big_helpers::settle(&mut g);
        assert_ne!(g.object_ref(target), old);
        assert_eq!(g.object_view(target).colors, 0);
        assert!(g.has_subtype(target, "Dragon"));
        assert!(!g.has_subtype(target, "Fractal"));
        assert_eq!(stats(&g, target), (6, 6));
        let spell = put(&mut g, p, "Fractalize", Zone::Hand);
        cast_x(&mut g, spell, 2, Target::Card(target));
        let bounce = put(&mut g, p, "Unsummon", Zone::Hand);
        big_helpers::cast(&mut g, bounce, None, &[Target::Card(target)]);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Hand);
        assert_eq!(g.object_view(target).colors, 0);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    }
}

#[test]
fn sos_final_tester_increment_optional_x_partial_counter_move_and_replacements() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Tester of the Tangential", Zone::Hand);
        big_helpers::cast(&mut g, source, None, &[]);
        big_helpers::settle(&mut g);
        let spell = put(&mut g, p, "Divination", Zone::Hand);
        big_helpers::cast(&mut g, spell, None, &[]);
        big_helpers::settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        assert_eq!(stats(&g, source), (2, 2));
        g.begin_step(Step::BeginCombat);
        pause(&mut g);
        big_helpers::checked(&mut g, Action::ChooseOptional { accept: false });
        big_helpers::settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        for (x, pay) in [(0, true), (1, false), (1, true), (3, true)] {
            let mut g = scenario(p);
            put(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
            let source = put(&mut g, p, "Tester of the Tangential", Zone::Battlefield);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.set_counters(source, Counters::of(CounterKind::PlusOne, 2));
            g.state.active = p.other();
            g.begin_step(Step::BeginCombat);
            assert!(g.state.pending_triggers.is_empty());
            g.state.active = p;
            g.state.priority = p;
            g.begin_step(Step::BeginCombat);
            tester_payment(&mut g, x, pay);
            if pay {
                pause(&mut g);
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(source),
                    },
                );
                big_helpers::checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
            }
            big_helpers::settle(&mut g);
            let moved = if pay { u64::from(x.min(2)) } else { 0 };
            assert_eq!(g.counter_count(source, CounterKind::PlusOne), 2 - moved);
            assert_eq!(g.counter_count(target, CounterKind::PlusOne), 2 * moved);
        }
    }
}

#[test]
fn sos_final_tester_reflexive_transfer_ignores_source_departure_lki_and_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "High Fae Trickster", Zone::Battlefield);
        let source = put(&mut g, p, "Tester of the Tangential", Zone::Battlefield);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.set_counters(source, Counters::of(CounterKind::PlusOne, 2));
        g.begin_step(Step::BeginCombat);
        tester_payment(&mut g, 2, true);
        pause(&mut g);
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        let old = g.object_ref(source);
        let blink = put(&mut g, p, "Daydream", Zone::Hand);
        big_helpers::cast(&mut g, blink, None, &[Target::Card(source)]);
        big_helpers::settle(&mut g);
        assert_ne!(g.object_ref(source), old);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(target, CounterKind::PlusOne), 0);
    }
}
