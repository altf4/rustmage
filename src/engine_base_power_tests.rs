use super::*;

fn attack(g: &mut Game, ids: &[CardId]) {
    let p = g.state.active;
    for &id in ids {
        g.state.objects[id as usize].summoning_sick = false;
    }
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.priority = p;
    for &card in ids {
        checked(g, Action::Attack { card });
    }
    checked(g, Action::FinishAttackers);
}
fn order(g: &mut Game) {
    g.prepare_triggers();
    while let Some(action) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PutTrigger { .. }))
    {
        checked(g, action);
    }
}
fn finish_damage(g: &mut Game) {
    for _ in 0..30 {
        if g.state.step == Step::CombatDamage {
            order(g);
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Pass | Action::FinishBlockers))
            .unwrap_or_else(|| {
                panic!(
                    "cannot advance combat: {:?} {:?}",
                    g.state.step, g.state.pending
                )
            });
        checked(g, action);
    }
    panic!("combat did not advance");
}

#[test]
fn power_excess_paid_sovereign_captures_resolution_recipients_after_its_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [80; 6];
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let mentor = named(&mut g, p, "Burrowguard Mentor", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p, "Sovereign Okinec Ahau", Zone::Hand);
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        let growth = named(&mut g, p, "Giant Growth", Zone::Hand);
        cast(&mut g, growth, None, &[Target::Card(bear)]);
        settle(&mut g);
        let wings = named(&mut g, p, "Water Wings", Zone::Hand);
        cast(&mut g, wings, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(
            (
                g.base_stats_without_hand(bear, None).power,
                g.stats(bear).power
            ),
            (4, 7)
        );
        attack(&mut g, &[source]);
        order(&mut g);
        let growth = named(&mut g, p, "Giant Growth", Zone::Hand);
        cast(&mut g, growth, None, &[Target::Card(mentor)]);
        // Resolve only the response; the independent attack packet stays stacked.
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(source)]);
        settle(&mut g);
        assert_eq!(g.state.objects[source as usize].zone, Zone::Hand);
        assert_eq!(g.state.objects[bear as usize].counters.plus_one, 3);
        assert_eq!(g.state.objects[mentor as usize].counters.plus_one, 3);
        assert_eq!(g.state.objects[enemy as usize].counters.plus_one, 0);
        assert_eq!(g.base_stats_without_hand(mentor, None).power, 2);
        assert_eq!(g.stats(mentor).power, 8);
        cleanup_checked(&mut g);
        assert_eq!(g.stats(bear).power, 5);
        assert_eq!(g.stats(mentor).power, 5);
    }
}

#[test]
fn power_excess_paid_sovereign_ward_requires_actual_two_mana_or_counters_the_spell() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let q = p.other();
            let source = named(&mut g, p, "Sovereign Okinec Ahau", Zone::Hand);
            cast(&mut g, source, None, &[]);
            settle(&mut g);
            g.state.players[q.index()].mana = [12; 6];
            g.state.priority = q;
            let shock = named(&mut g, q, "Shock", Zone::Hand);
            cast(&mut g, shock, None, &[Target::Card(source)]);
            settle_choices(&mut g, None, true, pay);
            assert_eq!(
                g.state.objects[source as usize].damage,
                if pay { 2 } else { 0 }
            );
            assert_eq!(
                g.state.players[q.index()]
                    .mana
                    .iter()
                    .map(|&v| u64::from(v))
                    .sum::<u64>(),
                if pay { 69 } else { 71 }
            );
        }
    }
}

#[test]
fn power_excess_paid_kutzil_restricts_all_cast_origins_but_not_mana_or_source_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let q = p.other();
        g.state.players[q.index()].mana = [30; 6];
        let source = named(&mut g, p, "Kutzil, Malamet Exemplar", Zone::Hand);
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        let shock = named(&mut g, q, "Shock", Zone::Hand);
        let twice = named(&mut g, q, "Think Twice", Zone::Graveyard);
        let forest = named(&mut g, q, "Forest", Zone::Battlefield);
        checked(&mut g, Action::Pass);
        assert_eq!(g.actor(), Some(q));
        for action in [
            Action::AnnounceCast { card: shock },
            Action::AnnounceFlashback { card: twice },
        ] {
            assert!(!g.legal_actions().contains(&action));
            reject(&mut g, action);
        }
        assert!(g.observe(q).casting_prohibited);
        assert!(!g.observe(p).casting_prohibited);
        checked(&mut g, Action::TapMana { card: forest });
        // Current controller, not immutable owner, determines the turn predicate.
        g.state.objects[source as usize].controller = q;
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: shock })
        );
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceFlashback { card: twice })
        );
        assert!(!g.observe(q).casting_prohibited);
        g.state.objects[source as usize].controller = p;
        g.state.priority = p;
        let own = named(&mut g, p, "Giant Growth", Zone::Hand);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: own })
        );
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(source)]);
        settle(&mut g);
        g.state.priority = q;
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: shock })
        );
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceFlashback { card: twice })
        );
        assert!(!g.observe(q).casting_prohibited);
    }
}

#[test]
fn power_excess_paid_kutzil_actual_combat_aggregates_only_qualified_creatures_and_survives_departure()
 {
    for p in [Player::P0, Player::P1] {
        for qualified in [false, true] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [80; 6];
            let source = named(&mut g, p, "Kutzil, Malamet Exemplar", Zone::Hand);
            cast(&mut g, source, None, &[]);
            settle(&mut g);
            let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            for id in [a, b] {
                let spell = named(
                    &mut g,
                    p,
                    if qualified {
                        "Giant Growth"
                    } else {
                        "Water Wings"
                    },
                    Zone::Hand,
                );
                cast(&mut g, spell, None, &[Target::Card(id)]);
                settle(&mut g);
                assert_eq!(g.object_view(id).power_above_base, qualified);
            }
            let before = g.state.players[p.index()].library.len();
            let island = named(&mut g, p, "Island", Zone::Battlefield);
            attack(&mut g, &[source, a, b]);
            finish_damage(&mut g);
            let packets = g
                .state
                .stack
                .iter()
                .filter(|s| {
                    s.card == source && matches!(s.kind, StackKind::TriggeredAbility { .. })
                })
                .count();
            assert_eq!(packets, usize::from(qualified));
            if qualified {
                checked(&mut g, Action::TapMana { card: island });
                let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
                cast(&mut g, bounce, None, &[Target::Card(source)]);
                settle(&mut g);
                assert_eq!(g.state.objects[source as usize].zone, Zone::Hand);
            }
            assert_eq!(
                g.state.players[p.index()].library.len(),
                before - usize::from(qualified)
            );
        }
    }
}

#[test]
fn power_excess_paid_voice_mobilizes_exactly_two_and_delayed_sacrifice_survives_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Voice of Victory", Zone::Hand);
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        assert!(g.observe(p.other()).casting_prohibited);
        attack(&mut g, &[source]);
        settle(&mut g);
        let tokens: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.zone == Zone::Battlefield && o.card.is_token())
            .map(|(i, _)| i as CardId)
            .collect();
        assert_eq!(tokens.len(), 2);
        for &id in &tokens {
            let v = g.object_view(id);
            assert_eq!(v.controller, p);
            assert!(v.tapped && v.attacking);
            assert_eq!((v.power, v.toughness), (1, 1));
            assert_eq!(v.token.unwrap().name, "Warrior Token");
        }
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(source)]);
        settle(&mut g);
        assert!(!g.observe(p.other()).casting_prohibited);
        g.state.step = Step::Main2;
        g.state.pending = Pending::Priority;
        g.state.priority = p;
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        settle(&mut g);
        for id in tokens {
            assert!(!g.on_battlefield(id));
        }
    }
}

#[test]
fn power_excess_signed_counter_capacity_failure_rolls_back_complete_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Sovereign Okinec Ahau", Zone::Hand);
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let item = g.activation_item(p, bear, 0);
        g.set_base_stats(&item, Subject::Source, Some(i128::MIN), None);
        g.state.objects[bear as usize].growth.power = i128::MAX;
        assert_eq!(g.stats(bear).power, -1);
        attack(&mut g, &[source]);
        order(&mut g);
        checked(&mut g, Action::Pass);
        let before = g.snapshot();
        let revision = g.revision();
        let actor = g.actor().unwrap();
        assert!(g.apply(actor, Action::Pass).is_err());
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), revision);
        assert_eq!(g.state.objects[bear as usize].counters.plus_one, 0);
    }
}
