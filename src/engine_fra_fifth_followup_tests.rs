use super::*;

#[test]
fn vraska_soul_ignores_opponents_spells_and_control_and_suppression_change_vigilance() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p.other());
        let vraska = put(&mut g, p, "Vraska, Soul of Stone", Zone::Battlefield);
        g.create_tokens(p, Card::test_index(5437), 1, false);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(i, o)| {
                (o.zone == Zone::Battlefield && o.is_token() && o.card == Card::test_index(5437))
                    .then_some(i as CardId)
            })
            .unwrap();
        assert!(g.keywords(token).contains(Keywords::VIGILANCE));
        let token_ref = g.object_ref(token);
        g.change_controller(token_ref, p.other());
        assert!(!g.keywords(token).contains(Keywords::VIGILANCE));
        g.change_controller(token_ref, p);
        assert!(g.keywords(token).contains(Keywords::VIGILANCE));
        let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(vraska));
        assert!(!g.printed_abilities_available(vraska));
        assert!(!g.keywords(token).contains(Keywords::VIGILANCE));
        g.move_object(aura, Zone::Graveyard);
        assert!(g.keywords(token).contains(Keywords::VIGILANCE));
        let shock = put(&mut g, p.other(), "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p)));
        settle(&mut g);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield
                    && o.is_token()
                    && o.card == Card::test_index(5437))
                .count(),
            1
        );
    }
}

#[test]
fn vraska_soul_cast_trigger_survives_source_and_obeys_token_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let vraska = put(&mut g, p, "Vraska, Soul of Stone", Zone::Battlefield);
        let creature = put(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, creature, None);
        settle(&mut g);
        assert!(!g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
            && o.is_token()
            && o.card == Card::test_index(5437)));
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p.other())));
        g.move_object(vraska, Zone::Graveyard);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            let tokens: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter_map(|(i, o)| {
                    (o.zone == Zone::Battlefield
                        && o.is_token()
                        && o.card == Card::test_index(5437)
                        && o.controller == p)
                        .then_some(i as CardId)
                })
                .collect();
            assert_eq!(tokens.len(), 2);
            for id in tokens {
                assert_eq!(g.object_view(id).owner, p);
                assert_eq!(g.object_view(id).colors, 0);
                assert_eq!(stats(&g, id), (1, 1));
                assert!(g.has_subtype(id, "Sculpture") && g.has_subtype(id, "Treasure"));
                assert!(!g.keywords(id).contains(Keywords::VIGILANCE));
            }
        }
    }
}

#[test]
fn tethermage_sacrifice_can_be_replaced_and_entry_trigger_survives_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        let mage = put(&mut g, p, "Tenured Tethermage", Zone::Hand);
        cast(&mut g, mage, None);
        for _ in 0..48 {
            g.prepare_triggers();
            if g.legal_actions().contains(&Action::DeclineEffectPayment) {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert!(g.legal_actions().contains(&Action::DeclineEffectPayment));
        g.move_object(mage, Zone::Graveyard);
        let object = g.object_ref(land);
        super::super::contracts::branch_contract(g.clone());
        act(&mut g, Action::Sacrifice { object });
        settle(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Exile);
        let tokens: Vec<_> = g
            .state
            .objects
            .iter()
            .filter(|o| {
                o.zone == Zone::Battlefield
                    && o.is_token()
                    && o.controller == p
                    && o.card == Card::test_index(5438)
            })
            .collect();
        assert_eq!(tokens.len(), 2);
        assert!(tokens.iter().all(|o| o.tapped && o.owner == p));
    }
}

#[test]
fn puppetbeast_rejects_unfunded_cost_and_sacrifice_can_go_to_exile() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let beast = put(&mut g, p, "Hungering Puppetbeast", Zone::Battlefield);
        let artifact = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let action = Action::AnnounceActivate {
            card: beast,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, action.clone());
        assert_eq!(g.object_view(artifact).zone, Zone::Battlefield);
        g.state.players[p.index()].mana[5] = 1;
        act(&mut g, action);
        let object = g.object_ref(artifact);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert_eq!(g.object_view(artifact).zone, Zone::Exile);
        for _ in 0..32 {
            if g.legal_actions()
                .contains(&Action::ChooseKeyword { index: 2 })
            {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        super::super::contracts::branch_contract(g.clone());
        act(&mut g, Action::ChooseKeyword { index: 2 });
        settle(&mut g);
        assert_eq!(g.counter_count(beast, CounterKind::PlusOne), 1);
        assert!(g.keywords(beast).contains(Keywords::HASTE));
    }
}

#[test]
fn plan_first_noncreature_cast_triggers_even_when_spell_is_countered() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Plan for All Outcomes", Zone::Battlefield);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        let negate = put(&mut g, p.other(), "Negate", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p.other())));
        g.prepare_triggers();
        act(&mut g, Action::Pass);
        cast(&mut g, negate, Some(Target::Spell(shock)));
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
            let jaces = fra_jaces(&g, p);
            assert_eq!(jaces.len(), 1);
            assert_eq!(g.counter_count(jaces[0], CounterKind::Loyalty), 1);
        }
    }
}
