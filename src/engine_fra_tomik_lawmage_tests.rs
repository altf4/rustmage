use super::*;

#[test]
fn tomik_restricts_each_planeswalker_independently_and_tracks_control() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let tomik = put(
            &mut g,
            p.other(),
            "Tomik, Orzhov Lawmage",
            Zone::Battlefield,
        );
        let one = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let two = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let a = put(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        let b = put(&mut g, p.other(), "Ajani Unrelenting", Zone::Battlefield);
        g.state.objects[a as usize].counters.loyalty = 5;
        g.state.objects[b as usize].counters.loyalty = 5;
        let ar = g.object_ref(a);
        let br = g.object_ref(b);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(
            &mut g,
            Action::AttackPlaneswalker {
                card: one,
                defender: ar,
            },
        );
        let forbidden = Action::AttackPlaneswalker {
            card: two,
            defender: ar,
        };
        reject(&mut g, forbidden.clone());
        assert!(g.legal_actions().contains(&Action::AttackPlaneswalker {
            card: two,
            defender: br
        }));
        assert!(g.legal_actions().contains(&Action::Attack { card: two }));
        let checkpoint = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for _ in 0..2 {
            g.change_controller(g.object_ref(tomik), p);
            assert!(g.legal_actions().contains(&forbidden));
            g.restore(&checkpoint).unwrap();
            assert!(!g.legal_actions().contains(&forbidden));
        }
    }
}

#[test]
fn tomik_counter_target_rechecked_and_tap_cost_committed() {
    for p in [Player::P0, Player::P1] {
        for remove in [false, true] {
            let mut g = scenario(p);
            let tomik = put(&mut g, p, "Tomik, Orzhov Lawmage", Zone::Battlefield);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let without = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            g.add_counter(target, CounterKind::PlusOne, 1);
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: tomik,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(without),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            pay(&mut g);
            assert!(g.object_view(tomik).tapped);
            if remove {
                g.state.objects[target as usize].counters.plus_one = 0;
            }
            let saved = g.snapshot();
            for _ in 0..2 {
                settle(&mut g);
                assert_eq!(g.keywords(target).contains(Keywords::FLYING), !remove);
                g.restore(&saved).unwrap();
            }
        }
    }
}

#[test]
fn tomik_competing_mandatory_attackers_maximize_requirements() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(
            &mut g,
            p.other(),
            "Tomik, Orzhov Lawmage",
            Zone::Battlefield,
        );
        put(&mut g, p.other(), "Storm, Windrider", Zone::Battlefield);
        let walker = put(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.state.objects[walker as usize].counters.loyalty = 5;
        let a = put(&mut g, p, "Flamewake Phoenix", Zone::Battlefield);
        let b = put(&mut g, p, "Flamewake Phoenix", Zone::Battlefield);
        let defender = g.object_ref(walker);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        assert!(!g.legal_actions().contains(&Action::FinishAttackers));
        act(&mut g, Action::AttackPlaneswalker { card: a, defender });
        assert!(
            !g.legal_actions()
                .contains(&Action::AttackPlaneswalker { card: b, defender })
        );
        assert!(g.legal_actions().contains(&Action::FinishAttackers));
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::FinishAttackers);
            assert_eq!(g.state.attackers.len(), 1);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn tomik_recipient_and_provider_ability_removal_follow_layer_timestamps() {
    for p in [Player::P0, Player::P1] {
        for remove_recipient in [false, true] {
            let mut g = scenario(p);
            let tomik = put(
                &mut g,
                p.other(),
                "Tomik, Orzhov Lawmage",
                Zone::Battlefield,
            );
            let walker = put(
                &mut g,
                p.other(),
                "Ajani, Caller of the Pride",
                Zone::Battlefield,
            );
            g.state.objects[walker as usize].counters.loyalty = 5;
            let recipient = if remove_recipient { walker } else { tomik };
            let aura = put(&mut g, p, "Fresh Start", Zone::Battlefield);
            // Low-level layer fixture: the recipient can be a planeswalker that
            // also has creature type in an ordinary game.
            g.state.objects[aura as usize].attached_to = Some(g.object_ref(recipient));
            assert!(!g.planeswalker_attack_limited(g.object_ref(walker)));
            g.move_object(aura, Zone::Graveyard);
            assert!(g.planeswalker_attack_limited(g.object_ref(walker)));
        }
    }
}

#[test]
fn copied_tomik_grants_to_current_controller_and_disappears_with_copy() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let own = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Ajani Unrelenting", Zone::Battlefield);
        g.state.objects[own as usize].counters.loyalty = 5;
        g.state.objects[enemy as usize].counters.loyalty = 5;
        g.apply_temporary_copy(
            g.object_ref(source),
            CopyRecipe::new(Card::from_name("Tomik, Orzhov Lawmage").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert!(g.planeswalker_attack_limited(g.object_ref(own)));
        assert!(!g.planeswalker_attack_limited(g.object_ref(enemy)));
        let saved = g.snapshot();
        for _ in 0..2 {
            g.change_controller(g.object_ref(source), p.other());
            assert!(!g.planeswalker_attack_limited(g.object_ref(own)));
            assert!(g.planeswalker_attack_limited(g.object_ref(enemy)));
            g.finish_cleanup();
            assert!(!g.planeswalker_attack_limited(g.object_ref(enemy)));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn tomik_paid_mandatory_attack_does_not_hide_available_free_walker_slot() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(
            &mut g,
            p.other(),
            "Tomik, Orzhov Lawmage",
            Zone::Battlefield,
        );
        put(
            &mut g,
            p.other(),
            "Dáin, Lord of the Iron Hills",
            Zone::Battlefield,
        );
        put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        g.refresh_enduring_stories();
        assert_eq!(g.attack_controller_tax(p.other()), 1);
        let walker = put(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.state.objects[walker as usize].counters.loyalty = 5;
        let a = put(&mut g, p, "Flamewake Phoenix", Zone::Battlefield);
        let b = put(&mut g, p, "Flamewake Phoenix", Zone::Battlefield);
        g.state.players[p.index()].mana[5] = 1;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: a });
        assert!(!g.legal_actions().contains(&Action::FinishAttackers));
        let defender = g.object_ref(walker);
        act(&mut g, Action::AttackPlaneswalker { card: b, defender });
        assert!(g.legal_actions().contains(&Action::FinishAttackers));
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::FinishAttackers);
            act(
                &mut g,
                Action::PayAttackCost {
                    payment: [0, 0, 0, 0, 0, 1],
                },
            );
            assert_eq!(g.state.attackers.len(), 2);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn tomik_static_grant_has_distinct_public_identity_from_activated_grants() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Tomik, Orzhov Lawmage", Zone::Battlefield);
        let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 5;
        let view = g.object_view(walker);
        let grants = &view.abilities.as_ref().unwrap().grants;
        let grant = grants
            .iter()
            .find(|grant| grant.grantor == g.object_ref(source))
            .unwrap();
        assert_eq!(grant.ability, None);
        assert_eq!(
            grant.static_rule,
            Some(GrantedStaticRule::PlaneswalkerAttackLimit)
        );
        let encoded = serde_json::to_value(grant).unwrap();
        assert!(encoded.get("ability").is_none());
        assert_eq!(encoded["static_rule"], "PlaneswalkerAttackLimit");
        assert!(!view.grant_available(
            g.object_ref(source),
            g.state.objects[source as usize].card,
            0,
            None
        ));
    }
}

#[test]
fn tomik_matching_reassigns_earlier_attacker_for_asymmetric_choices() {
    // First attacker can use either walker; second can use only the first.
    // A greedy first-slot assignment gets one, while reassignment gets two.
    assert_eq!(
        combat_restrictions::maximum_attack_matching(&[vec![0, 1], vec![0]], 2),
        2
    );
    assert_eq!(
        combat_restrictions::maximum_attack_matching(&[vec![0], vec![0]], 2),
        1
    );
    assert_eq!(
        combat_restrictions::maximum_attack_matching(&[vec![], vec![1]], 2),
        1
    );
}

#[test]
fn tomik_optional_attacker_cannot_displace_mandatory_attacker_from_only_free_slot() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(
            &mut g,
            p.other(),
            "Tomik, Orzhov Lawmage",
            Zone::Battlefield,
        );
        put(&mut g, p.other(), "Storm, Windrider", Zone::Battlefield);
        let walker = put(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.state.objects[walker as usize].counters.loyalty = 5;
        let optional = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let mandatory = put(&mut g, p, "Flamewake Phoenix", Zone::Battlefield);
        let defender = g.object_ref(walker);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(
            &mut g,
            Action::AttackPlaneswalker {
                card: optional,
                defender,
            },
        );
        assert!(!g.legal_actions().contains(&Action::FinishAttackers));
        let saved = g.snapshot();
        assert!(g.legal_actions().contains(&Action::CancelAttackDeclaration));
        act(&mut g, Action::CancelAttackDeclaration);
        assert!(!g.object_view(optional).tapped);
        g.restore(&saved).unwrap();
        act(&mut g, Action::CancelAttackDeclaration);
        act(
            &mut g,
            Action::AttackPlaneswalker {
                card: mandatory,
                defender,
            },
        );
        assert!(g.legal_actions().contains(&Action::FinishAttackers));
    }
}

#[test]
fn tomik_multiple_grants_survive_one_provider_being_suppressed_or_leaving() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let first = put(&mut g, p, "Tomik, Orzhov Lawmage", Zone::Battlefield);
        let second = put(&mut g, p, "Tomik, Orzhov Lawmage", Zone::Battlefield);
        let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 5;
        let aura = put(&mut g, p.other(), "Fresh Start", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(first));
        assert!(g.planeswalker_attack_limited(g.object_ref(walker)));
        let saved = g.snapshot();
        for _ in 0..2 {
            g.move_object(second, Zone::Graveyard);
            assert!(!g.planeswalker_attack_limited(g.object_ref(walker)));
            g.move_object(aura, Zone::Graveyard);
            assert!(g.planeswalker_attack_limited(g.object_ref(walker)));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn tomik_failed_tax_payment_and_abort_restore_entire_declaration() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(
            &mut g,
            p.other(),
            "Tomik, Orzhov Lawmage",
            Zone::Battlefield,
        );
        put(
            &mut g,
            p.other(),
            "Dáin, Lord of the Iron Hills",
            Zone::Battlefield,
        );
        put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        g.refresh_enduring_stories();
        let walker = put(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.state.objects[walker as usize].counters.loyalty = 5;
        let a = put(&mut g, p, "Flamewake Phoenix", Zone::Battlefield);
        let b = put(&mut g, p, "Flamewake Phoenix", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: a });
        let defender = g.object_ref(walker);
        act(&mut g, Action::AttackPlaneswalker { card: b, defender });
        let saved = g.snapshot();
        act(&mut g, Action::FinishAttackers);
        act(&mut g, Action::TapMana { card: land });
        reject(&mut g, Action::PayAttackCost { payment: [0; 6] });
        act(&mut g, Action::AbortAttackPayment);
        if matches!(g.state.pending, Pending::ManaRollback { .. }) {
            act(&mut g, Action::ChooseManaRollback { keep: false });
        }
        assert_eq!(g.snapshot(), saved);
    }
}

#[test]
fn tomik_activation_fizzles_after_target_leaves_and_reenters_with_counter() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let tomik = put(&mut g, p, "Tomik, Orzhov Lawmage", Zone::Battlefield);
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.add_counter(target, CounterKind::PlusOne, 1);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: tomik,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        let before = g.object_ref(target);
        g.move_object(target, Zone::Exile);
        g.move_object(target, Zone::Battlefield);
        g.add_counter(target, CounterKind::PlusOne, 1);
        assert_ne!(before, g.object_ref(target));
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert!(!g.keywords(target).contains(Keywords::FLYING));
            g.restore(&saved).unwrap();
        }
    }
}
