use super::*;

#[test]
fn tethermage_counts_the_actor_not_the_planeswalkers_controller_and_uses_one_trigger_per_event() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let theirs = put(
            &mut g,
            p.other(),
            "Liliana, Dreadhorde General",
            Zone::Battlefield,
        );
        let ours = put(&mut g, p, "Liliana, Dreadhorde General", Zone::Battlefield);
        g.add_counter(theirs, CounterKind::Loyalty, 5);
        g.add_counter(ours, CounterKind::Loyalty, 5);
        let mage = put(&mut g, p, "Inspired Tethermage", Zone::Battlefield);
        g.effect_counter_by(
            theirs,
            CounterKind::Loyalty,
            3,
            p,
            replacements::ResumeMode::Pending,
        );
        g.effect_counter_by(
            ours,
            CounterKind::Loyalty,
            2,
            p.other(),
            replacements::ResumeMode::Pending,
        );
        g.prepare_triggers();
        let before = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.counter_count(mage, CounterKind::PlusOne), 1);
        g.restore(&before).unwrap();
        settle(&mut g);
        assert_eq!(g.counter_count(mage, CounterKind::PlusOne), 1);
    }
}

#[test]
fn witchstalker_reflexive_follows_replaced_mill_and_can_target_old_graveyard_land() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Forest", Zone::Graveyard);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let spell = put(&mut g, p, "Primal Witchstalker", Zone::Hand);
        cast(&mut g, spell, None);
        let mut chosen = false;
        for _ in 0..64 {
            g.prepare_triggers();
            let actions = g.legal_actions();
            let action = Action::ChooseTarget {
                target: Target::Card(land),
            };
            if actions.contains(&action) {
                act(&mut g, action);
                chosen = true;
                break;
            }
            if let Some(action) = actions
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            {
                act(&mut g, action);
            } else {
                break;
            }
        }
        assert!(
            chosen,
            "four cards exiled by replacement still count as milled"
        );
        let before = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        assert!(g.object_view(land).tapped);
        g.restore(&before).unwrap();
        settle(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        assert!(g.object_view(land).tapped);
    }
}

#[test]
fn sphinx_copies_death_time_copiable_values_for_its_last_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let sphinx = put(
            &mut g,
            p.other(),
            "Sphinx of False Conclusions",
            Zone::Battlefield,
        );
        let reference = g.object_ref(sphinx);
        g.change_controller(reference, p);
        let recipe = CopyRecipe::new(g.state.objects[sphinx as usize].card)
            .with_exceptions(CopyExceptions::GOLEM);
        g.apply_temporary_copy(reference, recipe, CopyDeadline::EndOfTurn);
        assert_eq!(stats(&g, sphinx), (3, 3));
        g.move_object(sphinx, Zone::Graveyard);
        g.prepare_triggers();
        let before = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        settle(&mut g);
        let copies: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && o.is_token()
                    && o.controller == p
                    && o.card.name() == "Sphinx of False Conclusions")
                    .then_some(id as CardId)
            })
            .collect();
        assert_eq!(copies.len(), 1);
        assert_eq!(stats(&g, copies[0]), (3, 3));
        assert!(g.object_view(copies[0]).card.is_creature());
        assert!(g.object_view(copies[0]).kinds & 16 != 0);
        let after = g.snapshot();
        g.restore(&before).unwrap();
        settle(&mut g);
        assert_eq!(g.snapshot(), after);
    }
}

#[test]
fn sphinx_flash_casts_during_opponents_turn_and_exile_replacement_prevents_death_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p.other());
        g.state.priority = p;
        g.state.step = Step::BeginCombat;
        let sphinx = put(&mut g, p, "Sphinx of False Conclusions", Zone::Hand);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 2];
        act(&mut g, Action::AnnounceCast { card: sphinx });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(sphinx).zone, Zone::Battlefield);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        g.move_object(sphinx, Zone::Graveyard);
        g.prepare_triggers();
        let before = g.snapshot();
        settle(&mut g);
        assert_eq!(g.object_view(sphinx).zone, Zone::Exile);
        assert!(!g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
            && o.is_token()
            && o.card.name() == "Sphinx of False Conclusions"));
        g.restore(&before).unwrap();
        settle(&mut g);
        assert_eq!(g.object_view(sphinx).zone, Zone::Exile);
    }
}
