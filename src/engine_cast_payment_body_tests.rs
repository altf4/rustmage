use super::tests::{scenario, spawn};
use super::*;
const NAMES: [&str; 20] = [
    "Aberrant Manawurm",
    "Blazing Bomb",
    "Cuboid Colony",
    "Deluge Virtuoso",
    "Exhibition Tidecaller",
    "Expressive Firedancer",
    "Freestrider Commando",
    "Hungry Graffalon",
    "Muse Seeker",
    "Prompto Argentum",
    "Rancorous Archaic",
    "Sahagin",
    "Shantotto, Tactician Magician",
    "Spectacular Skywhale",
    "Tackle Artist",
    "Textbook Tabulator",
    "Thunderdrum Soloist",
    "Ultros, Obnoxious Octopus",
    "Unravel",
    "Arcane Omens",
];
fn ready(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    g.state.players[p.index()].mana = [40; 6];
    g.state.players[p.other().index()].mana = [40; 6];
    g
}
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn act(g: &mut Game, a: Action) {
    let p = g.actor().unwrap();
    g.apply(p, a).unwrap();
}
fn choose(g: &mut Game, p: Player) {
    let actions = g.legal_actions();
    let chosen=actions.iter().find(|a|matches!(a,Action::ChooseTarget{target:Target::Player(q)}if *q==p.other()))
 .or_else(||actions.iter().find(|a|matches!(a,Action::ChooseTarget{target:Target::Card(id)} if g.state.objects[*id as usize].owner==p.other())))
 .or_else(||actions.iter().find(|a|matches!(a,Action::PutTrigger{..}|Action::ChooseTarget{..}|Action::FinishTargets|Action::PayCosts{..}|Action::Discard{..}|Action::ArrangeLibrary{destination:LibraryDestination::Top,..}|Action::Pass))).unwrap_or_else(||panic!("{:?} {actions:?}",g.state.pending)).clone();
    act(g, chosen);
}
fn settle(g: &mut Game, p: Player) {
    for _ in 0..300 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
        {
            return;
        }
        assert!(g.outcome().is_none());
        choose(g, p);
    }
    panic!("did not settle")
}
fn paid(g: &mut Game, p: Player, id: CardId, x: Option<u32>, payment: Option<Mana>) {
    act(g, Action::AnnounceCast { card: id });
    if let Some(value) = x {
        act(g, Action::ChooseX { value });
    }
    while g.state.announcing.is_some() {
        if matches!(g.state.pending, Pending::PayCosts { .. })
            && let Some(payment) = payment
        {
            act(g, Action::PayCosts { payment });
            continue;
        }
        choose(g, p);
    }
    let item = g
        .state
        .stack
        .iter()
        .find(|s| s.kind == StackKind::Spell && s.card == id)
        .unwrap();
    let cast = item.cast.unwrap();
    assert_eq!(cast.origin, Zone::Hand);
    assert_eq!(cast.method, CastMethod::Normal);
    assert_eq!(
        cast.payment,
        payment.unwrap_or_else(|| {
            let c = cast.payment;
            assert!(c.into_iter().any(|n| n > 0));
            c
        })
    );
}
#[test]
fn all_twenty_full_bodies_actual_paid_casts_and_low_high_events_both_seats() {
    for p in [Player::P0, Player::P1] {
        for n in NAMES {
            if n == "Unravel" || n == "Arcane Omens" {
                continue;
            }
            let mut g = ready(p);
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let source = named(&mut g, p, n, Zone::Hand);
            paid(
                &mut g,
                p,
                source,
                None,
                (n == "Rancorous Archaic").then_some([1, 1, 1, 1, 1, 0]),
            );
            settle(&mut g, p);
            let baseline = g.stats(source);
            assert_eq!(g.state.objects[source as usize].zone, Zone::Battlefield);
            if n == "Rancorous Archaic" {
                assert_eq!(g.state.objects[source as usize].counters.plus_one, 5);
                assert_eq!(
                    baseline,
                    Stats {
                        power: 7,
                        toughness: 7
                    }
                );
            }
            if n == "Freestrider Commando" {
                assert_eq!(g.state.objects[source as usize].counters.plus_one, 0);
            }
            let shock = named(&mut g, p, "Shock", Zone::Hand);
            paid(&mut g, p, shock, None, None);
            settle(&mut g, p);
            let spring = named(&mut g, p, "Mind Spring", Zone::Hand);
            paid(&mut g, p, spring, Some(3), None);
            for packet in &g.state.pending_triggers {
                if packet.card == source {
                    let e = packet.cast_event.as_ref().expect("frozen qualifying cast");
                    assert_eq!(e.card, Card::from_name("Mind Spring").unwrap());
                    assert_eq!(e.controller, p);
                    assert_eq!(e.object.card, spring);
                    assert_eq!(e.payment.into_iter().map(u64::from).sum::<u64>(), 5);
                }
            }
            settle(&mut g, p);
            let view = g.object_view(source);
            match n {
                "Aberrant Manawurm" => assert_eq!((view.power, view.toughness), (8, 5)),
                "Blazing Bomb" => {
                    assert_eq!(view.counters.plus_one, 1);
                    g.state.objects[source as usize].summoning_sick = false;
                    let a = g
                        .legal_actions()
                        .into_iter()
                        .find(|a| matches!(a,Action::AnnounceActivate{card,..}if *card==source))
                        .unwrap();
                    act(&mut g, a);
                    while g.state.announcing.is_some() {
                        choose(&mut g, p)
                    }
                    assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
                    assert_eq!(g.state.stack.last().unwrap().source_facts.power, 2);
                    settle(&mut g, p);
                    assert_eq!(g.state.objects[enemy as usize].zone, Zone::Graveyard);
                }
                "Cuboid Colony" => assert_eq!((view.power, view.toughness), (2, 2)),
                "Deluge Virtuoso" => {
                    assert_eq!((view.power, view.toughness), (5, 5));
                    assert_eq!(g.state.objects[enemy as usize].counters.stun, 1);
                }
                "Exhibition Tidecaller" => {
                    assert_eq!(g.state.players[p.other().index()].library.len(), 7)
                }
                "Expressive Firedancer" => {
                    assert_eq!((view.power, view.toughness), (4, 4));
                    assert!(view.keywords.contains(Keywords::DOUBLE_STRIKE));
                }
                "Freestrider Commando" => assert_eq!((view.power, view.toughness), (3, 3)),
                "Hungry Graffalon" => assert_eq!((view.power, view.toughness), (4, 5)),
                "Muse Seeker" => {
                    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 5);
                    assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
                }
                "Prompto Argentum" => assert!(g.state.objects.iter().any(|o| o.owner == p
                    && o.zone == Zone::Battlefield
                    && o.card.is_token()
                    && o.card.has_subtype("Treasure"))),
                "Rancorous Archaic" => assert_eq!((view.power, view.toughness), (7, 7)),
                "Sahagin" => {
                    assert_eq!(view.counters.plus_one, 1);
                    assert!(view.keywords.contains(Keywords::CANT_BE_BLOCKED));
                }
                "Shantotto, Tactician Magician" => {
                    assert_eq!((view.power, view.toughness), (6, 4));
                    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 4);
                }
                "Spectacular Skywhale" => assert_eq!((view.power, view.toughness), (7, 7)),
                "Tackle Artist" => assert_eq!((view.power, view.toughness), (7, 6)),
                "Textbook Tabulator" => assert_eq!((view.power, view.toughness), (2, 5)),
                "Thunderdrum Soloist" => assert_eq!(g.state.players[p.other().index()].life, 14),
                "Ultros, Obnoxious Octopus" => {
                    assert_eq!(view.counters.plus_one, 0);
                    assert_eq!(g.state.objects[enemy as usize].counters.stun, 1);
                    let more = named(&mut g, p, "Mind Spring", Zone::Hand);
                    paid(&mut g, p, more, Some(6), None);
                    settle(&mut g, p);
                    assert_eq!(g.state.objects[source as usize].counters.plus_one, 8);
                    assert_eq!(g.state.objects[enemy as usize].counters.stun, 2);
                }
                _ => unreachable!(),
            }
        }
    }
}
#[test]
fn converge_arcane_omens_keeps_actual_color_choice_and_private_mandatory_discard() {
    for p in [Player::P0, Player::P1] {
        for payment in [[0, 0, 1, 0, 0, 4], [1, 1, 1, 1, 1, 0]] {
            let mut g = ready(p);
            for _ in 0..7 {
                named(&mut g, p.other(), "Forest", Zone::Hand);
            }
            let card = named(&mut g, p, "Arcane Omens", Zone::Hand);
            paid(&mut g, p, card, None, Some(payment));
            let count = (0..5).filter(|&i| payment[i] > 0).count();
            while g.state.stack.iter().any(|s| s.card == card) {
                choose(&mut g, p)
            }
            assert!(matches!(g.state.pending, Pending::EffectDiscard { .. }));
            assert_eq!(g.actor(), Some(p.other()));
            let snapshot = g.snapshot();
            let parent = g.clone();
            let mut child = parent.clone();
            drop(parent);
            settle(&mut child, p);
            settle(&mut g, p);
            assert_eq!(g.snapshot(), child.snapshot());
            assert_eq!(
                g.state.players[p.other().index()].cards_discarded_this_turn,
                count as u64
            );
            g.restore(&snapshot).unwrap();
            settle(&mut g, p);
            assert_eq!(g.snapshot(), child.snapshot());
        }
    }
}
#[test]
fn unravel_uses_actual_zero_plot_cast_and_countered_victim_before_departure() {
    for p in [Player::P0, Player::P1] {
        let q = p.other();
        let mut g = ready(q);
        let djinn = named(&mut g, q, "Djinn of Fool's Fall", Zone::Hand);
        act(&mut g, Action::AnnouncePlot { card: djinn });
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayPlot { .. }))
            .unwrap();
        act(&mut g, a);
        assert_eq!(g.state.players[q.index()].spells_cast_this_turn, 0);
        g.state.turn += 2;
        act(&mut g, Action::AnnouncePlotted { card: djinn });
        act(&mut g, Action::PayCosts { payment: [0; 6] });
        assert_eq!(g.state.stack.last().unwrap().cast.unwrap().mana_spent(), 0);
        act(&mut g, Action::Pass);
        let spell = named(&mut g, p, "Unravel", Zone::Hand);
        paid(&mut g, p, spell, None, None);
        let before = g.state.players[p.index()].cards_drawn_this_turn;
        settle(&mut g, p);
        assert_eq!(g.state.objects[djinn as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before + 1);
    }
}
#[test]
fn increment_zero_payment_negative_power_or_toughness_and_both_intervening_checks() {
    for p in [Player::P0, Player::P1] {
        let mut g = ready(p);
        let source = named(&mut g, p, "Cuboid Colony", Zone::Battlefield);
        g.state.objects[source as usize].growth.power = -2;
        let djinn = named(&mut g, p, "Djinn of Fool's Fall", Zone::Hand);
        act(&mut g, Action::AnnouncePlot { card: djinn });
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayPlot { .. }))
            .unwrap();
        act(&mut g, a);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 0);
        g.state.turn += 2;
        act(&mut g, Action::AnnouncePlotted { card: djinn });
        act(&mut g, Action::PayCosts { payment: [0; 6] });
        settle(&mut g, p);
        assert_eq!(g.state.objects[source as usize].counters.plus_one, 1);
        assert_eq!(
            g.stats(source),
            Stats {
                power: 0,
                toughness: 2
            }
        );
        for reentry in [false, true] {
            let mut g = ready(p);
            let source = named(&mut g, p, "Hungry Graffalon", Zone::Battlefield);
            let spring = named(&mut g, p, "Mind Spring", Zone::Hand);
            paid(&mut g, p, spring, Some(2), None);
            assert!(
                g.state
                    .stack
                    .iter()
                    .chain(&g.state.pending_triggers)
                    .any(|s| s.card == source && s.cast_event.is_some())
            );
            if reentry {
                g.move_object(source, Zone::Hand);
                g.move_object(source, Zone::Battlefield);
            } else {
                g.state.objects[source as usize].counters.plus_one += 1;
            }
            settle(&mut g, p);
            assert_eq!(
                g.state.objects[source as usize].counters.plus_one,
                u64::from(!reentry)
            );
        }
    }
}
#[test]
fn queued_actual_payment_survives_countered_cause_and_source_reentry() {
    for p in [Player::P0, Player::P1] {
        let mut g = ready(p);
        let source = named(
            &mut g,
            p,
            "Shantotto, Tactician Magician",
            Zone::Battlefield,
        );
        let original = g.object_ref(source);
        let spring = named(&mut g, p, "Mind Spring", Zone::Hand);
        paid(&mut g, p, spring, Some(3), None);
        let victim = g
            .state
            .stack
            .iter()
            .find(|s| s.kind == StackKind::Spell && s.card == spring)
            .unwrap()
            .id;
        assert!(g.counter_stack_item(victim, false));
        g.move_object(source, Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        assert_ne!(g.object_ref(source), original);
        let packet = g
            .state
            .stack
            .iter()
            .chain(&g.state.pending_triggers)
            .find(|s| s.card == source)
            .unwrap();
        assert_eq!(packet.object, original);
        assert_eq!(
            packet
                .cast_event
                .as_ref()
                .unwrap()
                .payment
                .into_iter()
                .map(u64::from)
                .sum::<u64>(),
            5
        );
        let parent = g.clone();
        let mut child = parent.clone();
        drop(parent);
        settle(&mut child, p);
        settle(&mut g, p);
        assert_eq!(g.snapshot(), child.snapshot());
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(
            g.stats(source),
            Stats {
                power: 0,
                toughness: 4
            }
        );
    }
}
#[test]
fn paid_and_free_plot_and_noncast_entry_counter_provenance_is_distinct() {
    for p in [Player::P0, Player::P1] {
        let mut g = ready(p);
        let source = named(&mut g, p, "Freestrider Commando", Zone::Hand);
        paid(&mut g, p, source, None, None);
        settle(&mut g, p);
        assert_eq!(g.object_view(source).counters.plus_one, 0);
        g.move_object(source, Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        assert_eq!(g.object_view(source).counters.plus_one, 2);
        g.move_object(source, Zone::Hand);
        act(&mut g, Action::AnnouncePlot { card: source });
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayPlot { .. }))
            .unwrap();
        act(&mut g, a);
        g.state.turn += 2;
        act(&mut g, Action::AnnouncePlotted { card: source });
        act(&mut g, Action::PayCosts { payment: [0; 6] });
        settle(&mut g, p);
        assert_eq!(g.object_view(source).counters.plus_one, 2);
        let rancorous = named(&mut g, p, "Rancorous Archaic", Zone::Hand);
        g.move_object(rancorous, Zone::Battlefield);
        assert_eq!(g.object_view(rancorous).counters.plus_one, 0);
    }
}
#[test]
fn unravel_draws_for_real_discount_even_when_countering_is_prevented() {
    for p in [Player::P0, Player::P1] {
        let q = p.other();
        let mut g = ready(q);
        named(&mut g, q, "Honest Rutstein", Zone::Battlefield);
        let bear = named(&mut g, q, "Gigantic Big Bear", Zone::Hand);
        paid(&mut g, q, bear, None, None);
        let packet = g.state.stack.last().unwrap();
        assert_eq!(packet.cast.unwrap().mana_spent(), 6);
        assert_eq!(packet.mana_value(), 7);
        act(&mut g, Action::Pass);
        let counter = named(&mut g, p, "Unravel", Zone::Hand);
        paid(&mut g, p, counter, None, None);
        settle(&mut g, p);
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
    }
}
#[test]
fn actual_payment_includes_kicker_and_warp_but_excludes_separate_ward() {
    for p in [Player::P0, Player::P1] {
        let mut g = ready(p);
        let spell = named(&mut g, p, "Burst Lightning", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseKicker { pay: true });
        while g.state.announcing.is_some() {
            choose(&mut g, p);
        }
        assert_eq!(
            g.state
                .stack
                .iter()
                .find(|s| s.card == spell)
                .unwrap()
                .cast
                .unwrap()
                .mana_spent(),
            5
        );
        let mut g = ready(p);
        let spell = named(&mut g, p, "Bygone Colossus", Zone::Hand);
        act(&mut g, Action::AnnounceWarp { card: spell });
        while g.state.announcing.is_some() {
            choose(&mut g, p);
        }
        let cast = g
            .state
            .stack
            .iter()
            .find(|s| s.card == spell)
            .unwrap()
            .cast
            .unwrap();
        assert_eq!(cast.method, CastMethod::Warp);
        assert_eq!(cast.mana_spent(), 3);
        let mut g = ready(p);
        let target = named(&mut g, p.other(), "Hulking Raptor", Zone::Battlefield);
        let spell = named(&mut g, p, "Shock", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        while g.state.announcing.is_some() {
            let a = Action::ChooseTarget {
                target: Target::Card(target),
            };
            if g.legal_actions().contains(&a) {
                act(&mut g, a)
            } else {
                choose(&mut g, p)
            }
        }
        let payment = g
            .state
            .stack
            .iter()
            .find(|s| s.card == spell)
            .unwrap()
            .cast
            .unwrap()
            .payment;
        assert_eq!(payment.into_iter().sum::<u32>(), 1);
        for _ in 0..20 {
            if g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::PayResolutionMana { .. }))
            {
                break;
            }
            choose(&mut g, p);
        }
        act(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 0, 2],
            },
        );
        assert_eq!(
            g.state
                .stack
                .iter()
                .find(|s| s.card == spell)
                .unwrap()
                .cast
                .unwrap()
                .payment,
            payment
        );
        settle(&mut g, p);
        assert_eq!(g.object_view(target).damage, 2);
    }
}

#[test]
fn root_review_textbook_actual_paid_entry_retains_two_card_private_surveillance() {
    for p in [Player::P0, Player::P1] {
        let mut g = ready(p);
        let source = named(&mut g, p, "Textbook Tabulator", Zone::Hand);
        let before = g.state.players[p.index()].library.len();
        paid(&mut g, p, source, None, None);
        for _ in 0..100 {
            if g.observe(p).library_choice.is_some() {
                break;
            }
            choose(&mut g, p);
        }
        let view = g
            .observe(p)
            .library_choice
            .expect("actual ETB must surveil two");
        assert!(view.surveil);
        assert_eq!(view.remaining.len(), 2);
        assert!(g.observe(p.other()).library_choice.is_none());
        let [first, second] = [view.remaining[0].id, view.remaining[1].id];
        let saved = g.snapshot();
        let revision = g.revision();
        let a = Action::ArrangeLibrary {
            card: first,
            destination: LibraryDestination::Graveyard,
        };
        assert!(g.apply(p.other(), a.clone()).is_err());
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), revision);
        let mut child = {
            let parent = g.clone();
            let child = parent.clone();
            drop(parent);
            child
        };
        act(&mut child, a.clone());
        act(&mut g, a.clone());
        assert_eq!(g.snapshot(), child.snapshot());
        g.restore(&saved).unwrap();
        act(&mut g, a);
        assert_eq!(g.snapshot(), child.snapshot());
        let a = Action::ArrangeLibrary {
            card: second,
            destination: LibraryDestination::Top,
        };
        act(&mut g, a.clone());
        act(&mut child, a);
        settle(&mut g, p);
        settle(&mut child, p);
        assert_eq!(g.snapshot(), child.snapshot());
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
        assert_eq!(g.state.players[p.index()].library.last(), Some(&second));
        assert_eq!(g.state.objects[first as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[source as usize].zone, Zone::Battlefield);
    }
}
