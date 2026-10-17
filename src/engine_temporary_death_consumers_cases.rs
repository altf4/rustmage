#[test]
fn consumers_presumed_returns_owner_then_suspects_exact_new_permanent() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        g.state.objects[bear as usize].controller = p;
        let old = g.object_ref(bear);
        paid(&mut g, p, "Presumed Dead", Some(Target::Card(bear)));
        assert_eq!(g.stats(bear).power, 4);
        paid(&mut g, p, "Stab", Some(Target::Card(bear)));
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert_ne!(g.object_ref(bear), old);
        assert_eq!(g.object_view(bear).controller, p.other());
        assert!(!g.object_view(bear).tapped);
        assert_eq!(g.stats(bear).power, 2);
        assert!(g.is_suspected(bear));
        assert!(g.keywords(bear).contains(Keywords::MENACE));
        assert!(g.keywords(bear).contains(Keywords::CANT_BLOCK));
        assert_eq!(g.observe(p).suspected, vec![g.object_ref(bear)]);
        assert_eq!(g.observe(p.other()).suspected, vec![g.object_ref(bear)]);
        let bytes = g.state_bytes();
        g.set_suspected(g.object_ref(bear), true);
        assert_eq!(g.state_bytes(), bytes);
        g.state.step = Step::Cleanup;
        g.advance_step();
        assert!(g.is_suspected(bear));
        paid(&mut g, p.other(), "Unsummon", Some(Target::Card(bear)));
        assert!(g.state.suspected.is_none());
        g.set_suspected(old, true);
        assert!(g.state.suspected.is_none());
    }
}

#[test]
fn consumers_galuf_uses_dying_power_and_late_optional_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let donor = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let recipient = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        paid(&mut g, p, "Galuf's Final Act", Some(Target::Card(donor)));
        assert_eq!(g.stats(donor).power, 3);
        // Killing the donor starts a separate late target announcement.
        let stab = named(&mut g, p, "Stab", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: stab });
        while g.state.announcing.is_some() {
            if g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(donor),
            }) {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(donor),
                    },
                );
            } else {
                choose(&mut g);
            }
        }
        drive(&mut g, Some(Target::Card(recipient)));
        assert_eq!(g.object_view(donor).zone, Zone::Graveyard);
        assert_eq!(g.state.objects[recipient as usize].counters.plus_one, 1);
        // Stab lowered dying power from three to one; use exact death LKI.
    }
}

#[test]
fn consumers_rabid_incremental_more_than_sixty_four_and_empty_selection() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bears: Vec<_> = (0..70)
            .map(|_| named(&mut g, p, "Grizzly Bears", Zone::Battlefield))
            .collect();
        let rabid = named(&mut g, p, "Rabid Attack", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: rabid });
        assert!(g.legal_actions().contains(&Action::FinishTargets));
        for &id in &bears {
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
            assert!(!g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(id)
            }));
        }
        drive(&mut g, None);
        assert!(bears.iter().all(|&id| g.stats(id).power == 3));
        assert_eq!(g.state.temporary_abilities.as_ref().unwrap().len(), 70);
        for _ in 0..70 {
            named(&mut g, p, "Forest", Zone::Library);
        }
        let drawn = g.state.players[p.index()].cards_drawn_this_turn;
        paid(&mut g, p, "Day of Judgment", None);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, drawn + 70);
        assert!(g.state.temporary_abilities.is_none());
        paid(&mut g, p, "Rabid Attack", None);
    }
}

#[test]
fn consumers_perigee_normal_and_warp_full_grant_tapped_return() {
    for p in [Player::P0, Player::P1] {
        for warp in [false, true] {
            let mut g = setup(p);
            let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let beckoner = named(&mut g, p, "Perigee Beckoner", Zone::Hand);
            act(
                &mut g,
                if warp {
                    Action::AnnounceWarp { card: beckoner }
                } else {
                    Action::AnnounceCast { card: beckoner }
                },
            );
            drive(&mut g, Some(Target::Card(bear)));
            assert_eq!(g.stats(bear).power, 4);
            assert_eq!(g.stats(beckoner).power, 4);
            let before = g.object_ref(bear);
            paid(&mut g, p, "Stab", Some(Target::Card(bear)));
            assert_ne!(g.object_ref(bear), before);
            assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
            assert!(g.object_view(bear).tapped);
            assert_eq!(g.stats(bear).power, 2);
        }
    }
}

#[test]
fn consumers_person_interest_full_etb_and_lammasu_clear_then_optional_death() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let person = paid(&mut g, p, "Person of Interest", None);
        assert!(g.is_suspected(person));
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield
                    && o.card.info().type_line.contains("Detective"))
                .count(),
            1
        );
        let lammasu = paid(&mut g, p, "Absolving Lammasu", None);
        assert!(!g.is_suspected(person));
        let life = g.state.players[p.index()].life;
        paid(&mut g, p, "Murder", Some(Target::Card(lammasu)));
        assert_eq!(g.state.players[p.index()].life, life + 3);
    }
}

#[test]
fn consumers_it_doesnt_add_up_owner_return_and_persistent_public_designation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
        paid(&mut g, p, "It Doesn't Add Up", Some(Target::Card(bear)));
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert_eq!(g.object_view(bear).controller, p);
        assert!(g.is_suspected(bear));
        let snapshot = g.snapshot();
        let mut child = {
            let parent = g.clone();
            parent.clone()
        };
        child.set_suspected(child.object_ref(bear), false);
        assert!(!child.is_suspected(bear));
        assert!(g.is_suspected(bear));
        child.restore(&snapshot).unwrap();
        assert!(child.is_suspected(bear));
        assert_eq!(g.state_bytes(), child.state_bytes());
    }
}

#[test]
fn consumers_reasonable_doubt_counter_payment_and_independent_suspect_group() {
    for p in [Player::P0, Player::P1] {
        for decline in [false, true] {
            let mut g = setup(p.other());
            let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let growth = named(&mut g, p.other(), "Giant Growth", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: growth });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            while g.state.announcing.is_some() {
                choose(&mut g);
            }
            priority(&mut g, p);
            let doubt = named(&mut g, p, "Reasonable Doubt", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: doubt });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Spell(growth),
                },
            );
            let mut creature_chosen = false;
            while g.state.announcing.is_some() {
                if !creature_chosen
                    && g.legal_actions().contains(&Action::ChooseTarget {
                        target: Target::Card(bear),
                    })
                {
                    act(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(bear),
                        },
                    );
                    creature_chosen = true;
                } else {
                    choose(&mut g);
                }
            }
            assert!(creature_chosen);
            let mut payment_seen = false;
            for _ in 0..300 {
                if g.state.pending == Pending::Priority
                    && g.state.stack.is_empty()
                    && g.state.resolving.is_none()
                {
                    break;
                }
                if g.legal_actions().contains(&Action::DeclineResolutionMana) {
                    payment_seen = true;
                    if decline {
                        act(&mut g, Action::DeclineResolutionMana);
                    } else {
                        let payment = g
                            .legal_actions()
                            .into_iter()
                            .find(|a| matches!(a, Action::PayResolutionMana { .. }))
                            .unwrap();
                        act(&mut g, payment);
                    }
                } else {
                    choose(&mut g);
                }
            }
            assert!(payment_seen);
            assert!(g.is_suspected(bear));
            assert_eq!(g.stats(bear).power, if decline { 2 } else { 5 });
            assert_eq!(g.object_view(doubt).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn consumers_convenient_target_full_aura_removal_does_not_unsuspect_and_grave_activation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let aura = paid(&mut g, p, "Convenient Target", Some(Target::Card(bear)));
        assert!(g.is_suspected(bear));
        assert_eq!(g.stats(bear).power, 3);
        paid(&mut g, p, "Disenchant", Some(Target::Card(aura)));
        assert!(g.is_suspected(bear));
        assert_eq!(g.stats(bear).power, 2);
        let activation = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::AnnounceActivate { card, .. } if *card == aura))
            .unwrap();
        act(&mut g, activation);
        drive(&mut g, None);
        assert_eq!(g.object_view(aura).zone, Zone::Hand);
        assert!(g.is_suspected(bear));
    }
}

#[test]
fn consumers_person_departure_does_not_omit_the_independent_detective_tail() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let person = named(&mut g, p, "Person of Interest", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: person });
        for _ in 0..100 {
            if g.state.stack.iter().any(|s| {
                matches!(s.kind, StackKind::TriggeredAbility { .. })
                    && s.trigger_definition() == Card::from_name("Person of Interest").unwrap()
            }) {
                break;
            }
            choose(&mut g);
        }
        assert_eq!(g.object_view(person).zone, Zone::Battlefield);
        paid(&mut g, p, "Murder", Some(Target::Card(person)));
        assert_eq!(g.object_view(person).zone, Zone::Graveyard);
        assert!(g.state.suspected.is_none());
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield && o.card.has_subtype("Detective"))
                .count(),
            1
        );
    }
}

#[test]
fn consumers_lammasu_chosen_illegal_target_prevents_the_entire_life_and_suspect_ability() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let foe = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let lammasu = paid(&mut g, p, "Absolving Lammasu", None);
        let murder = named(&mut g, p, "Murder", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: murder });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(lammasu),
            },
        );
        let mut targeted = false;
        for _ in 0..100 {
            if g.state
                .stack
                .iter()
                .any(|s| s.trigger_definition() == Card::from_name("Absolving Lammasu").unwrap())
            {
                break;
            }
            if g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(foe),
            }) {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(foe),
                    },
                );
                targeted = true;
            } else {
                choose(&mut g);
            }
        }
        assert!(targeted);
        let life = g.state.players[p.index()].life;
        paid(&mut g, p, "Unsummon", Some(Target::Card(foe)));
        assert_eq!(g.state.players[p.index()].life, life);
        assert!(g.state.suspected.is_none());
    }
}

#[test]
fn consumers_partial_rabid_targets_and_repeated_grants_draw_for_dying_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let first = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        g.state.objects[first as usize].controller = p;
        let second = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = named(&mut g, p, "Rabid Attack", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        for id in [first, second] {
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        while g.state.announcing.is_some() {
            choose(&mut g);
        }
        paid(&mut g, p, "Unsummon", Some(Target::Card(second)));
        assert_eq!(g.object_view(second).zone, Zone::Hand);
        assert_eq!(g.stats(first).power, 3);
        assert_eq!(g.observe(p).temporary_abilities.len(), 1);
        paid(&mut g, p, "Rabid Attack", Some(Target::Card(first)));
        assert_eq!(g.observe(p).temporary_abilities.len(), 2);
        assert_eq!(g.stats(first).power, 4);
        for _ in 0..2 {
            named(&mut g, p, "Forest", Zone::Library);
        }
        let drawn = g.state.players[p.index()].cards_drawn_this_turn;
        paid(&mut g, p, "Murder", Some(Target::Card(first)));
        assert_eq!(g.object_view(first).zone, Zone::Graveyard);
        assert_eq!(g.object_view(first).owner, p.other());
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, drawn + 2);
        assert!(g.state.temporary_abilities.is_none());
    }
}

#[test]
fn consumers_real_lantern_removes_first_grave_binding_without_suspecting_new_zone() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        g.state.objects[bear as usize].controller = p;
        let grant = paid(&mut g, p, "Presumed Dead", Some(Target::Card(bear)));
        let lantern = paid(&mut g, p, "Soul-Guide Lantern", Some(Target::Card(grant)));
        let bolt = named(&mut g, p, "Lightning Bolt", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: bolt });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        for _ in 0..100 {
            if g.state
                .stack
                .iter()
                .any(|s| s.trigger_definition() == Card::from_name("Presumed Dead").unwrap())
            {
                break;
            }
            choose(&mut g);
        }
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        priority(&mut g, p);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: lantern,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        drive(&mut g, None);
        assert_eq!(g.object_view(bear).zone, Zone::Exile);
        assert!(g.state.suspected.is_none());
    }
}
