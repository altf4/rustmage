//! Official no-XMage-class Plan consumer: paid entry/scry and independent payoff.
use super::big_helpers::checked;
use super::*;

fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = put(g, p, name, Zone::Hand);
    super::big_helpers::cast(g, id, None, &[]);
    id
}
fn scry(g: &mut Game, p: Player, bottom: bool) {
    for _ in 0..24 {
        if g.observe(p).library_choice.is_some() {
            break;
        }
        checked(g, Action::Pass);
    }
    let choice = g.observe(p).library_choice.expect("private scry");
    assert!(g.observe(p.other()).library_choice.is_none());
    assert_eq!(choice.remaining.len(), 1);
    let card = choice.remaining[0].id;
    let wrong = Action::ArrangeLibrary {
        card: CardId::MAX,
        destination: LibraryDestination::Top,
    };
    reject(g, wrong);
    checked(
        g,
        Action::ArrangeLibrary {
            card,
            destination: if bottom {
                LibraryDestination::Bottom
            } else {
                LibraryDestination::Top
            },
        },
    );
}
fn create(g: &mut Game, p: Player, bottom: bool) -> CardId {
    let creature = paid(g, p, "Bear Cub");
    scry(g, p, bottom);
    creature
}
#[test]
fn msh_political_triumph_paid_four_creatures_ordered_private_scry_and_whole_payoff() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let plan = paid(&mut g, p, "Political Triumph");
        settle(&mut g);
        assert_eq!(g.counter_count(plan, CounterKind::Plan), 0);
        // The entry trigger excludes opponent creatures and our noncreatures.
        g.state.active = p.other();
        g.state.priority = p.other();
        let foreign = paid(&mut g, p.other(), "Bear Cub");
        settle(&mut g);
        assert_eq!(g.counter_count(plan, CounterKind::Plan), 0);
        g.state.active = p;
        g.state.priority = p;
        let artifact = paid(&mut g, p, "Hedron Archive");
        settle(&mut g);
        assert_eq!(g.counter_count(plan, CounterKind::Plan), 0);
        let mut creatures = Vec::new();
        for n in 1..=4 {
            let hand = g.hand_count(p);
            creatures.push(create(&mut g, p, n % 2 == 0));
            settle(&mut g);
            if n < 4 {
                assert_eq!(g.counter_count(plan, CounterKind::Plan), n);
                assert_eq!(g.hand_count(p), hand);
            } else {
                assert_eq!(g.object_view(plan).zone, Zone::Graveyard);
                assert_eq!(g.hand_count(p), hand + 1);
            }
        }
        assert_eq!(g.counter_count(foreign, CounterKind::PlusOne), 0);
        assert_eq!(g.counter_count(artifact, CounterKind::PlusOne), 0);
        for id in creatures {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        }
    }
}
#[test]
fn msh_political_triumph_queued_payoff_survives_departure_blink_control_or_counter_removal() {
    for p in [Player::P0, Player::P1] {
        for variant in 0..4 {
            let mut g = scenario(p);
            let plan = paid(&mut g, p, "Political Triumph");
            settle(&mut g);
            g.effect_counter(
                plan,
                CounterKind::Plan,
                3,
                replacements::ResumeMode::Pending,
            );
            let hand = g.hand_count(p);
            let creature = create(&mut g, p, false);
            assert_eq!(g.counter_count(plan, CounterKind::Plan), 4);
            let old = g.object_ref(plan);
            match variant {
                0 => {
                    g.move_object(plan, Zone::Exile);
                }
                1 => {
                    g.move_object(plan, Zone::Exile);
                    g.move_object(plan, Zone::Battlefield);
                    assert_ne!(g.object_ref(plan), old);
                }
                2 => {
                    g.state.objects[plan as usize].controller = p.other();
                }
                _ => {
                    g.pay_counters(plan, &Counters::of(CounterKind::Plan, 4));
                }
            }
            settle(&mut g);
            assert_eq!(g.hand_count(p), hand + 1);
            assert_eq!(g.hand_count(p.other()), 0);
            assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 1);
            assert_eq!(
                g.object_view(plan).zone,
                match variant {
                    0 => Zone::Exile,
                    1 | 2 => Zone::Battlefield,
                    _ => Zone::Graveyard,
                }
            );
        }
    }
}
#[test]
fn msh_political_triumph_counter_replacement_crossing_boosts_all_current_creatures() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        paid(&mut g, p, "Doubling Season");
        settle(&mut g);
        let plan = paid(&mut g, p, "Political Triumph");
        settle(&mut g);
        let first = create(&mut g, p, false);
        settle(&mut g);
        assert_eq!(g.counter_count(plan, CounterKind::Plan), 2);
        let second = create(&mut g, p, true);
        settle(&mut g);
        assert_eq!(g.object_view(plan).zone, Zone::Graveyard);
        assert_eq!(g.counter_count(first, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(second, CounterKind::PlusOne), 2);
    }
}
