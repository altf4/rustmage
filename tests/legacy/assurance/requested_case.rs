//! Case of the Uneaten Feast and exact-incarnation Case state.
use super::*;

#[test]
fn case_solves_only_after_five_life_at_own_end_step_and_resets_on_departure() {
    for p in [Player::P0, Player::P1] {
        for amount in [4, 5] {
            let mut g = scenario(p);
            let case = put(&mut g, p, "Case of the Uneaten Feast", Zone::Battlefield);
            let reference = g.object_ref(case);
            g.gain_life(p, amount);
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::AnnounceActivate { card, .. } if *card == case))
            );
            g.state.active = p.other();
            g.begin_step(Step::End);
            g.give_priority();
            g.prepare_triggers();
            big_helpers::settle(&mut g);
            assert!(!g.case_solved(reference));
            g.state.active = p;
            g.begin_step(Step::End);
            g.give_priority();
            g.prepare_triggers();
            let snap = g.snapshot();
            let mut branch = g.clone();
            big_helpers::settle(&mut g);
            assert_eq!(g.case_solved(reference), amount == 5);
            branch.restore(&snap).unwrap();
            big_helpers::settle(&mut branch);
            assert_eq!(g.snapshot(), branch.snapshot());
            g.move_object(case, Zone::Exile);
            g.move_object(case, Zone::Battlefield);
            assert!(!g.case_solved(g.object_ref(case)));
        }
    }
}

#[test]
fn case_sacrifice_grants_only_current_creature_cards_until_turn_end() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let case = put(&mut g, p, "Case of the Uneaten Feast", Zone::Battlefield);
        let creature = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        let land = put(&mut g, p, "Forest", Zone::Graveyard);
        let opponent = put(&mut g, p.other(), "Llanowar Elves", Zone::Graveyard);
        g.solve_case(g.object_ref(case));
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: case,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        assert_eq!(g.object_view(case).zone, Zone::Graveyard);
        big_helpers::settle(&mut g);
        assert!(g.has_graveyard_cast(p, creature, GraveyardCastKind::Cast));
        assert!(!g.has_graveyard_cast(p, land, GraveyardCastKind::Cast));
        assert!(!g.has_graveyard_cast(p, opponent, GraveyardCastKind::Cast));
        let late = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        assert!(!g.has_graveyard_cast(p, late, GraveyardCastKind::Cast));
        g.move_object(creature, Zone::Exile);
        g.move_object(creature, Zone::Graveyard);
        assert!(!g.has_graveyard_cast(p, creature, GraveyardCastKind::Cast));
    }
}

#[test]
fn case_triggers_for_controlled_creature_entries_even_after_solving() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let case = put(&mut g, p, "Case of the Uneaten Feast", Zone::Battlefield);
        g.solve_case(g.object_ref(case));
        let own = put(&mut g, p, "Llanowar Elves", Zone::Hand);
        let enemy = put(&mut g, p.other(), "Llanowar Elves", Zone::Hand);
        let life = g.state.players[p.index()].life;
        g.move_object(enemy, Zone::Battlefield);
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life);
        g.move_object(own, Zone::Battlefield);
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 1);
    }
}

#[test]
fn case_graveyard_permission_expires_at_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let case = put(&mut g, p, "Case of the Uneaten Feast", Zone::Battlefield);
        let creature = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        g.solve_case(g.object_ref(case));
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: case,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        big_helpers::settle(&mut g);
        assert!(g.has_graveyard_cast(p, creature, GraveyardCastKind::Cast));
        g.expire_graveyard_cast_permissions();
        assert!(!g.has_graveyard_cast(p, creature, GraveyardCastKind::Cast));
    }
}

#[test]
fn case_permission_allows_normal_paid_creature_cast() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let case = put(&mut g, p, "Case of the Uneaten Feast", Zone::Battlefield);
        let creature = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        g.solve_case(g.object_ref(case));
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: case,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        big_helpers::settle(&mut g);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: creature })
        );
        big_helpers::cast(&mut g, creature, None, &[]);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
    }
}
#[test]
fn case_solved_designation_stays_with_the_permanent_through_copy_changes() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let case = put(&mut g, p, "Case of the Uneaten Feast", Zone::Battlefield);
        let other = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let original = g.object_ref(case);
        g.solve_case(original);
        let case_recipe = CopyRecipe::new(Card::from_name("Case of the Uneaten Feast").unwrap());
        g.apply_temporary_copy(g.object_ref(other), case_recipe, CopyDeadline::EndOfTurn);
        assert!(!g.case_solved(g.object_ref(other)));
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::AnnounceActivate { card, .. } if *card == other))
        );
        let bear_recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
        g.apply_temporary_copy(original, bear_recipe, CopyDeadline::EndOfTurn);
        assert!(g.case_solved(original));
        super::contracts::branch_contract(g.clone());
    }
}

#[test]
fn case_solve_trigger_keeps_controller_after_control_changes() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let case = put(&mut g, p, "Case of the Uneaten Feast", Zone::Battlefield);
        let reference = g.object_ref(case);
        g.gain_life(p, 5);
        g.begin_step(Step::End);
        g.give_priority();
        g.prepare_triggers();
        g.state.objects[case as usize].controller = p.other();
        big_helpers::settle(&mut g);
        assert!(g.case_solved(reference));
        g.state.priority = p.other();
        let action = Action::AnnounceActivate {
            card: case,
            ability: 0,
            counter_payment: Counters::default(),
        };
        assert!(g.legal_actions().contains(&action));
    }
}

#[test]
fn case_loses_and_recovers_its_solved_activation_when_abilities_are_removed() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let case = put(&mut g, p, "Case of the Uneaten Feast", Zone::Battlefield);
        let source = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let case_ref = g.object_ref(case);
        let source_ref = g.object_ref(source);
        g.solve_case(case_ref);
        let action = Action::AnnounceActivate {
            card: case,
            ability: 0,
            counter_payment: Counters::default(),
        };
        assert!(g.legal_actions().contains(&action));
        let timestamp = g.next_role_timestamp();
        g.state
            .roles
            .get_or_insert_with(Default::default)
            .source_linked_effects
            .push(SourceLinkedEffectView {
                source: source_ref,
                recipient: case_ref,
                controller: p.other(),
                kind: SourceLinkedEffectKind::RemoveAbilities,
                timestamp,
            });
        assert!(g.case_solved(case_ref));
        assert!(!g.legal_actions().contains(&action));
        g.move_object(source, Zone::Graveyard);
        assert!(g.case_solved(case_ref));
        assert!(g.legal_actions().contains(&action));
    }
}

#[test]
fn case_illegal_activation_and_sacrifice_restore_are_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let case = put(&mut g, p, "Case of the Uneaten Feast", Zone::Battlefield);
        let creature = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        let action = Action::AnnounceActivate {
            card: case,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, action.clone());
        g.solve_case(g.object_ref(case));
        super::contracts::branch_contract(g.clone());
        big_helpers::checked(&mut g, action);
        pay(&mut g);
        let before = g.snapshot();
        let mut branch = g.clone();
        big_helpers::settle(&mut g);
        assert!(g.has_graveyard_cast(p, creature, GraveyardCastKind::Cast));
        branch.restore(&before).unwrap();
        big_helpers::settle(&mut branch);
        assert_eq!(g.snapshot(), branch.snapshot());
    }
}

#[test]
fn case_sacrifice_redirected_to_exile_still_grants_permission() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let case = put(&mut g, p, "Case of the Uneaten Feast", Zone::Battlefield);
        let creature = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        put(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
        g.solve_case(g.object_ref(case));
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: case,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        assert_eq!(g.object_view(case).zone, Zone::Exile);
        big_helpers::settle(&mut g);
        assert!(g.has_graveyard_cast(p, creature, GraveyardCastKind::Cast));
    }
}
