use super::*;

fn settle_scry_choices(g: &mut Game) -> usize {
    let mut choices = 0;
    for _ in 0..96 {
        g.prepare_triggers();
        let actions = g.legal_actions();
        if let Some(action) = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ArrangeLibrary {
                        destination: LibraryDestination::Top,
                        ..
                    }
                )
            })
            .cloned()
        {
            act(g, action);
            choices += 1;
        } else if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return choices;
        } else if let Some(action) = actions
            .into_iter()
            .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
        {
            act(g, action);
        } else {
            panic!("unexpected scry history choice {:?}", g.state.pending);
        }
    }
    panic!("scry history resolution did not finish")
}

#[test]
fn proctor_simultaneous_friendly_entries_surveille_once_each_after_source_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let proctor = put(&mut g, p, "Proctor of Potential", Zone::Battlefield);
        let a = put(&mut g, p, "Bear Cub", Zone::Hand);
        let b = put(&mut g, p, "Hill Giant", Zone::Hand);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        let observers = g.death_trigger_sources();
        let events = g.move_objects_raw(&[a, b, enemy], Zone::Battlefield);
        g.finish_zone_events(&events, &observers);
        g.move_object(proctor, Zone::Graveyard);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            assert_eq!(settle_scry_choices(&mut g), 2);
            for viewer in [p, p.other()] {
                assert!(g.observe(viewer).players[p.index()].scried_or_surveilled_this_turn);
                assert!(
                    !g.observe(viewer).players[p.other().index()].scried_or_surveilled_this_turn
                );
            }
        }
    }
}

#[test]
fn surveillance_phantasm_actually_attacks_untapped_after_scry() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Surveillance Phantasm", Zone::Battlefield);
        g.state.objects[source as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        reject(&mut g, Action::Attack { card: source });
        g.state.step = Step::Main1;
        g.state.pending = Pending::Priority;
        let scry = put(&mut g, p, "Plunge into Winter", Zone::Hand);
        cast(&mut g, scry, None);
        assert_eq!(settle_scry_choices(&mut g), 1);
        assert!(g.keywords(source).contains(Keywords::DEFENDER));
        assert!(g.keywords(source).contains(Keywords::FLYING));
        assert!(g.keywords(source).contains(Keywords::VIGILANCE));
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: source });
        act(&mut g, Action::FinishAttackers);
        assert!(g.state.attackers.iter().any(|a| a.card == source));
        assert!(!g.object_view(source).tapped);
    }
}

#[test]
fn futurescribe_checks_scry_history_at_resolution_and_applies_counter_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Desperate Futurescribe", Zone::Battlefield);
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        g.begin_step(Step::BeginCombat);
        g.prepare_triggers();
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        assert!(!g.observe(p).players[p.index()].scried_or_surveilled_this_turn);
        let scry = put(&mut g, p, "Plunge into Winter", Zone::Hand);
        cast(&mut g, scry, None);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            assert_eq!(settle_scry_choices(&mut g), 1);
            assert_eq!(g.counter_count(target, CounterKind::PlusOne), 2);
            g.finish_cleanup();
            assert_eq!(g.counter_count(target, CounterKind::PlusOne), 2);
        }
    }
}

#[test]
fn proft_cannot_pay_unfunded_optional_cost_and_does_not_gain_counter_or_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let proft = put(&mut g, p, "Proft, Consulting Detective", Zone::Battlefield);
        let scry = put(&mut g, p, "Plunge into Winter", Zone::Hand);
        let hand = g.hand_count(p);
        cast(&mut g, scry, None);
        for _ in 0..48 {
            let actions = g.legal_actions();
            if let Some(action) = actions
                .iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::ArrangeLibrary {
                            destination: LibraryDestination::Top,
                            ..
                        }
                    )
                })
                .cloned()
            {
                act(&mut g, action);
            } else if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
                break;
            } else {
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
        }
        assert!(matches!(g.state.pending, Pending::ResolutionMana { .. }));
        g.state.players[p.index()].mana = [0; 6];
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::PayResolutionMana { .. }))
        );
        super::super::contracts::branch_contract(g.clone());
        act(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert_eq!(g.counter_count(proft, CounterKind::PlusOne), 0);
        // Plunge drew one card; the declined Proft ability draws none.
        assert_eq!(g.hand_count(p), hand);
    }
}
