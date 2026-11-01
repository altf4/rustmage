use super::*;

#[test]
fn kwia_created_lotus_funds_an_actual_spell_and_replays() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Kwia Vigorbloom", Zone::Battlefield);
        g.gain_life(p, 1);
        g.prepare_triggers();
        settle(&mut g);
        let lotus = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.is_token()
                    && o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.card == Card::test_index(5453))
                .then_some(id as CardId)
            })
            .unwrap();
        let spell = put(&mut g, p, "Lightning Strike", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        let action = g.legal_actions().into_iter().find(|a| matches!(a, Action::ActivateMana { card, production, .. } if *card == lotus && production[3] == 3)).unwrap();
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            act(&mut g, action.clone());
            assert_ne!(g.object_view(lotus).zone, Zone::Battlefield);
            assert_eq!(g.state.players[p.index()].mana[3], 3);
            cast(&mut g, spell, Some(Target::Player(p.other())));
            assert_eq!(g.state.players[p.index()].mana[3], 1);
            settle(&mut g);
        }
    }
}

#[test]
fn kraken_created_forest_funds_green_spell_after_summoning_sickness() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Verdant Kraken", Zone::Battlefield);
        g.begin_step(Step::Upkeep);
        g.prepare_triggers();
        settle(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.is_token()
                    && o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.card == Card::test_index(5454))
                .then_some(id as CardId)
            })
            .unwrap();
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::TapMana { card, .. } if *card == token))
        );
        g.state.objects[token as usize].summoning_sick = false;
        let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Giant Growth", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::TapMana { card, .. } if *card == token))
            .unwrap();
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            act(&mut g, action.clone());
            assert_eq!(g.state.players[p.index()].mana[4], 1);
            cast(&mut g, spell, Some(Target::Card(creature)));
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            assert_eq!(stats(&g, creature), (5, 5));
        }
    }
}

#[test]
fn kiora_rechecks_that_its_target_is_still_attacking() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Kiora of Salt and Sand", Zone::Battlefield);
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.create_tokens(p, Card::JacePlaneswalkerToken, 1, false);
        let jace = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.is_token()
                    && o.zone == Zone::Battlefield
                    && o.card == Card::JacePlaneswalkerToken)
                    .then_some(id as CardId)
            })
            .unwrap();
        g.state.objects[jace as usize].counters.loyalty = 5;
        act(
            &mut g,
            Action::AnnounceActivate {
                card: jace,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        let target = Action::ChooseTarget {
            target: Target::Card(attacker),
        };
        for _ in 0..48 {
            if g.legal_actions().contains(&target) {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        act(&mut g, target);
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        let reference = g.object_ref(attacker);
        g.remove_from_combat(attacker, reference);
        assert!(g.object_view(attacker).tapped);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert!(g.object_view(attacker).tapped);
        assert!(!g.keywords(attacker).contains(Keywords::CANT_BE_BLOCKED));
        g.restore(&saved).unwrap();
        settle(&mut g);
        assert!(g.object_view(attacker).tapped);
    }
}

#[test]
fn copying_opponents_loyalty_ability_does_not_count_as_your_activation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p.other());
        put(&mut g, p, "Kiora of Salt and Sand", Zone::Battlefield);
        g.create_tokens(p.other(), Card::JacePlaneswalkerToken, 1, false);
        let jace = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.is_token()
                    && o.zone == Zone::Battlefield
                    && o.card == Card::JacePlaneswalkerToken)
                    .then_some(id as CardId)
            })
            .unwrap();
        g.state.objects[jace as usize].counters.loyalty = 5;
        act(
            &mut g,
            Action::AnnounceActivate {
                card: jace,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        let mut copy = g.state.stack.last().unwrap().clone();
        copy.id = g.next_stack_id();
        copy.controller = p;
        copy.is_copy = true;
        g.push_stack_item(copy);
        assert!(g.observe(p).players[p.other().index()].loyalty_ability_activated_this_turn);
        assert!(!g.observe(p.other()).players[p.index()].loyalty_ability_activated_this_turn);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        g.restore(&saved).unwrap();
        assert!(!g.observe(p.other()).players[p.index()].loyalty_ability_activated_this_turn);
    }
}
