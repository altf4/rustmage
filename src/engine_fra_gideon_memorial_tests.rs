use super::*;

fn cadet(g: &mut Game, p: Player) -> CardId {
    let before = g.state.objects.len();
    g.create_tokens(p, Card::test_index(5298), 1, false);
    (before..g.state.objects.len())
        .find_map(|id| {
            let o = &g.state.objects[id];
            (o.zone == Zone::Battlefield
                && o.is_token()
                && o.controller == p
                && o.card == Card::test_index(5298))
            .then_some(id as CardId)
        })
        .unwrap()
}

#[test]
fn memorial_abilities_affect_only_current_controlled_creature_tokens() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let memorial = put(&mut g, p, "Gideon's Memorial", Zone::Battlefield);
        let own = cadet(&mut g, p);
        let enemy = cadet(&mut g, p.other());
        let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert_eq!(stats(&g, own), (3, 2));
        assert!(g.keywords(own).contains(Keywords::VIGILANCE));
        assert_eq!(stats(&g, enemy), (2, 2));
        assert_eq!(stats(&g, creature), (2, 2));
        assert!(!g.keywords(enemy).contains(Keywords::VIGILANCE));
        let object = g.object_ref(own);
        g.change_controller(object, p.other());
        assert_eq!(stats(&g, own), (2, 2));
        assert!(!g.keywords(own).contains(Keywords::VIGILANCE));
        g.change_controller(object, p);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        g.move_object(memorial, Zone::Graveyard);
        assert_eq!(stats(&g, own), (2, 2));
        assert!(!g.keywords(own).contains(Keywords::VIGILANCE));
        g.restore(&saved).unwrap();
        assert_eq!(stats(&g, own), (3, 2));
        assert!(g.keywords(own).contains(Keywords::VIGILANCE));
    }
}

#[test]
fn memorial_mana_is_only_spendable_on_planeswalker_spells() {
    for p in [Player::P0, Player::P1] {
        for walker in [false, true] {
            let mut g = scenario(p);
            let memorial = put(&mut g, p, "Gideon's Memorial", Zone::Battlefield);
            g.state.players[p.index()].mana = if walker {
                [1, 0, 0, 0, 0, 1]
            } else {
                [0, 0, 0, 0, 0, 1]
            };
            let mana = g.legal_actions().into_iter().find(|a| matches!(a,
                Action::ActivateMana { card, ability: 0, production: [1, 0, 0, 0, 0, 0], .. } if *card == memorial)).unwrap();
            let before = g.snapshot();
            act(&mut g, mana.clone());
            assert_eq!(
                g.mana_lots(p)
                    .iter()
                    .find(|lot| lot.quality.restriction == ManaRestrictionView::PlaneswalkerSpell)
                    .unwrap()
                    .amounts[0],
                1
            );
            let card = put(
                &mut g,
                p,
                if walker {
                    "Ajani, Caller of the Pride"
                } else {
                    "Ajani's Pridemate"
                },
                Zone::Hand,
            );
            if walker {
                cast(&mut g, card, None);
                settle(&mut g);
                assert_eq!(g.object_view(card).zone, Zone::Battlefield);
            } else {
                reject(&mut g, Action::AnnounceCast { card });
                assert_eq!(g.object_view(card).zone, Zone::Hand);
            }
            g.restore(&before).unwrap();
            act(&mut g, mana);
            assert!(
                g.mana_lots(p)
                    .iter()
                    .any(|lot| lot.quality.restriction == ManaRestrictionView::PlaneswalkerSpell)
            );
        }
    }
}

#[test]
fn memorial_discard_ability_targets_only_attacking_or_blocking_creatures() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p.other());
        let attacker = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let idle = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let memorial = put(&mut g, p, "Gideon's Memorial", Zone::Hand);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        g.state.priority = p;
        act(
            &mut g,
            Action::AnnounceActivate {
                card: memorial,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(idle),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(attacker),
            },
        );
        pay(&mut g);
        assert_eq!(g.object_view(memorial).zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(attacker).zone, Zone::Graveyard);
    }
}

#[test]
fn copied_memorial_produces_restricted_mana_that_survives_provider_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let recipe = CopyRecipe::new(Card::from_name("Gideon's Memorial").unwrap());
        g.apply_temporary_copy(g.object_ref(source), recipe, CopyDeadline::EndOfTurn);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        let action = g.legal_actions().into_iter().find(|a| matches!(a,
            Action::ActivateMana { card, ability: 0, production: [1, 0, 0, 0, 0, 0], .. } if *card == source)).unwrap();
        act(&mut g, action);
        g.move_object(source, Zone::Graveyard);
        assert!(
            g.mana_lots(p)
                .iter()
                .any(|lot| lot.quality.restriction == ManaRestrictionView::PlaneswalkerSpell)
        );
        let creature = put(&mut g, p, "Ajani's Pridemate", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: creature });
    }
}
