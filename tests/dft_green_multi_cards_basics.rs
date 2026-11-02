//! Actual card outcomes for the green vanilla, cycling and saddle family.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn drain(g: &mut Game) {
    for _ in 0..64 {
        if g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.pending == Pending::Priority
        {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishTargets
                    | Action::FinishStackTargets
            )
        });
    }
    panic!("bounded card resolution");
}
fn target_ready(g: &mut Game, target: Target) {
    for _ in 0..32 {
        if g.legal_actions().contains(&Action::ChooseTarget { target }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("target decision not reached");
}
fn cycle(g: &mut Game, card: CardId, x: Option<u32>) {
    act(
        g,
        Action::AnnounceActivate {
            card,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if let Some(value) = x {
        act(g, Action::ChooseX { value });
    }
    pay(g);
}
#[test]
fn vanilla_creatures_have_printed_stats_costs_types_and_no_rules_abilities() {
    for (name, expected, mana, subtype) in [
        ("Jibbirik Omnivore", (3, 2), 2, "Beast"),
        ("Terrian, World Tyrant", (9, 7), 5, "Ooze"),
    ] {
        let mut g = scenario(Player::P0);
        let card = put(&mut g, Player::P0, name, Zone::Hand);
        let before: u32 = g.state.players[0].mana.iter().sum();
        cast(&mut g, card, None);
        drain(&mut g);
        assert_eq!(stats(&g, card), expected, "{name}");
        assert_eq!(g.state.objects[card as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.players[0].mana.iter().sum::<u32>(), before - mana);
        assert!(g.has_subtype(card, subtype));
        assert!(
            !g.object_has_rules_abilities(card),
            "{name} is a valid no-abilities card"
        );
    }
}
#[test]
fn migrating_ketradon_entry_gains_four_but_cycling_only_draws_and_discards() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let card = put(&mut g, p, "Migrating Ketradon", Zone::Hand);
        let before = g.state.players[p.index()].life;
        cast(&mut g, card, None);
        drain(&mut g);
        assert_eq!(stats(&g, card), (6, 6));
        assert!(g.keywords(card).contains(Keywords::REACH));
        assert_eq!(g.state.players[p.index()].life, before + 4);
        let hand = put(&mut g, p, "Migrating Ketradon", Zone::Hand);
        cycle(&mut g, hand, None);
        drain(&mut g);
        assert_eq!(g.state.objects[hand as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(g.state.players[p.index()].life, before + 4);
    }
}
#[test]
fn agonasaur_cycles_to_an_uncrewed_vehicle_with_permanent_counters_and_temporary_keywords() {
    let mut g = scenario(Player::P0);
    let vehicle = put(
        &mut g,
        Player::P1,
        "Cultivator's Caravan",
        Zone::Battlefield,
    );
    let rex = put(&mut g, Player::P0, "Agonasaur Rex", Zone::Hand);
    cycle(&mut g, rex, None);
    target_ready(&mut g, Target::Card(vehicle));
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(vehicle),
        },
    );
    drain(&mut g);
    assert!(!g.is_creature(vehicle));
    assert_eq!(g.counter_count(vehicle, CounterKind::PlusOne), 2);
    assert!(
        g.keywords(vehicle)
            .contains(Keywords::TRAMPLE.union(Keywords::INDESTRUCTIBLE))
    );
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    let cast_rex = put(&mut g, Player::P0, "Agonasaur Rex", Zone::Hand);
    g.state.players[0].mana = [8; 6];
    cast(&mut g, cast_rex, None);
    drain(&mut g);
    assert_eq!(stats(&g, cast_rex), (8, 8));
    assert!(g.keywords(cast_rex).contains(Keywords::TRAMPLE));
    g.finish_cleanup();
    assert_eq!(g.counter_count(vehicle, CounterKind::PlusOne), 2);
    assert!(!g.keywords(vehicle).contains(Keywords::INDESTRUCTIBLE));
}
#[test]
fn cycling_optional_triggers_can_decline_and_webstrike_uses_exact_chosen_x() {
    for name in ["Agonasaur Rex", "Webstrike Elite"] {
        let mut g = scenario(Player::P0);
        let card = put(&mut g, Player::P0, name, Zone::Hand);
        cycle(&mut g, card, (name == "Webstrike Elite").then_some(0));
        drain(&mut g);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
        assert_eq!(g.state.objects[card as usize].zone, Zone::Graveyard);
    }
    let mut g = scenario(Player::P1);
    let boots = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    let caravan = put(
        &mut g,
        Player::P0,
        "Cultivator's Caravan",
        Zone::Battlefield,
    );
    let bear = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let card = put(&mut g, Player::P1, "Webstrike Elite", Zone::Hand);
    cycle(&mut g, card, Some(2));
    target_ready(&mut g, Target::Card(boots));
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(caravan),
        },
    );
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(boots),
        },
    );
    drain(&mut g);
    assert_eq!(g.state.objects[boots as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.objects[caravan as usize].zone, Zone::Battlefield);
    let body = put(&mut g, Player::P1, "Webstrike Elite", Zone::Hand);
    cast(&mut g, body, None);
    drain(&mut g);
    assert_eq!(stats(&g, body), (3, 3));
    assert!(g.keywords(body).contains(Keywords::REACH));
}
fn token_count(g: &Game) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.is_token())
        .count()
}
#[test]
fn printed_saddle_thresholds_trigger_only_saddled_attacks_and_buffs_expire() {
    for (name, base, boosted, keyword, threshold) in [
        ("Alacrian Jaguar", (4, 4), (6, 6), Keywords::VIGILANCE, 1),
        ("Venomsac Lagac", (2, 1), (2, 4), Keywords::DEATHTOUCH, 2),
        ("Autarch Mammoth", (5, 5), (5, 5), Keywords::NONE, 5),
    ] {
        for saddled in [false, true] {
            let mut g = scenario(Player::P0);
            let mount = put(&mut g, Player::P0, name, Zone::Hand);
            cast(&mut g, mount, None);
            drain(&mut g);
            assert_eq!(stats(&g, mount), base, "{name}");
            assert!(g.keywords(mount).contains(keyword));
            let before = token_count(&g);
            assert_eq!(before, usize::from(name == "Autarch Mammoth"));
            if saddled {
                let rider_names: &[&str] = match threshold {
                    1 => &["Llanowar Elves"],
                    2 => &["Llanowar Elves", "Llanowar Elves"],
                    5 => &["Hill Giant", "Grizzly Bears"],
                    _ => unreachable!(),
                };
                let riders: Vec<_> = rider_names
                    .iter()
                    .map(|name| put(&mut g, Player::P0, name, Zone::Battlefield))
                    .collect();
                act(
                    &mut g,
                    Action::AnnounceActivate {
                        card: mount,
                        ability: 0,
                        counter_payment: Counters::default(),
                    },
                );
                if matches!(g.state.pending, Pending::PayCosts { .. }) {
                    pay(&mut g);
                }
                let own = g.object_ref(mount);
                reject(&mut g, Action::ChooseSaddler { object: own });
                reject(&mut g, Action::FinishSaddle);
                for (i, rider) in riders.iter().copied().enumerate() {
                    let object = g.object_ref(rider);
                    act(&mut g, Action::ChooseSaddler { object });
                    if i + 1 < riders.len() {
                        reject(&mut g, Action::FinishSaddle);
                    }
                }
                act(&mut g, Action::FinishSaddle);
                drain(&mut g);
                assert!(
                    riders.iter().all(|id| g.state.objects[*id as usize].tapped),
                    "Saddle {threshold}"
                );
            }
            g.state.objects[mount as usize].summoning_sick = false;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            act(&mut g, Action::Attack { card: mount });
            act(&mut g, Action::FinishAttackers);
            drain(&mut g);
            assert_eq!(
                stats(&g, mount),
                if saddled { boosted } else { base },
                "{name} saddled={saddled}"
            );
            assert_eq!(
                token_count(&g),
                before + usize::from(saddled && name == "Autarch Mammoth")
            );
            if name == "Alacrian Jaguar" {
                assert!(!g.state.objects[mount as usize].tapped);
            }
            for (id, o) in g.state.objects.iter().enumerate() {
                if o.zone == Zone::Battlefield && o.is_token() {
                    assert_eq!(stats(&g, id as CardId), (3, 3));
                    assert!(g.has_subtype(id as CardId, "Elephant"));
                }
            }
            g.finish_cleanup();
            assert_eq!(stats(&g, mount), base);
        }
    }
}
