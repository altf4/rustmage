//! Independent Sanctum Lurker grant and empower interaction tests.
use super::*;

fn lurker_source(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Bear Cub", Zone::Battlefield);
    g.state.objects[id as usize].card = Card::from_name("Sanctum Lurker").unwrap();
    g.register_battlefield_sources(id);
    id
}
fn blank(g: &mut Game, p: Player, walker: CardId) {
    let mut effect = g.activation_item(p, walker, 0);
    effect.targets = vec![BoundTarget {
        group: 0,
        target: Target::Card(walker),
        object: Some(g.object_ref(walker)),
    }];
    g.remove_target_abilities_until_end_of_turn(&effect);
}
fn grant(g: &Game, walker: CardId, provider: CardId) -> Action {
    Action::AnnouncePlaneswalkerGrant {
        source: g.object_ref(walker),
        grantor: g.object_ref(provider),
        ability: 0,
    }
}

#[test]
fn sanctum_recipient_ability_removal_respects_grant_timestamps() {
    for p in [Player::P0, Player::P1] {
        for blank_first in [false, true] {
            let mut g = scenario(p);
            let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
            g.state.objects[walker as usize].counters.loyalty = 3;
            if blank_first {
                blank(&mut g, p, walker);
            }
            let provider = lurker_source(&mut g, p);
            if !blank_first {
                blank(&mut g, p, walker);
            }
            let action = grant(&g, walker, provider);
            let saved = g.snapshot();
            assert_eq!(g.legal_actions().contains(&action), blank_first);
            if blank_first {
                act(&mut g, action);
                pay(&mut g);
                settle(&mut g);
                assert_eq!(g.state.players[p.index()].life, 21);
                assert_eq!(g.state.players[p.other().index()].life, 19);
            }
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn copied_sanctum_provider_grants_real_loyalty_ability_and_survives_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let ref_provider = g.object_ref(provider);
        g.apply_temporary_copy(
            ref_provider,
            CopyRecipe::new(Card::from_name("Sanctum Lurker").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 3;
        let action = grant(&g, walker, provider);
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, action.clone());
            pay(&mut g);
            assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 5);
            g.move_object(provider, Zone::Exile);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, 21);
            assert_eq!(g.state.players[p.other().index()].life, 19);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn sanctum_empower_selects_one_of_multiple_jaces_and_doubles_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.create_tokens(p, Card::JacePlaneswalkerToken, 2, false);
        let tokens: Vec<CardId> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(i, o)| {
                (o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.card == Card::JacePlaneswalkerToken)
                    .then_some(i as CardId)
            })
            .collect();
        assert_eq!(tokens.len(), 2);
        for &id in &tokens {
            g.state.objects[id as usize].counters.loyalty = 1;
        }
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let lurker = put(&mut g, p, "Sanctum Lurker", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: lurker });
        pay(&mut g);
        for _ in 0..48 {
            if g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseJace { .. }))
            {
                break;
            }
            choose(&mut g, |a| {
                matches!(
                    a,
                    Action::Pass | Action::PutTrigger { .. } | Action::ChooseReplacement { .. }
                )
            });
        }
        let first = g.object_ref(tokens[0]);
        let second = g.object_ref(tokens[1]);
        assert!(
            g.legal_actions()
                .contains(&Action::ChooseJace { object: first })
        );
        assert!(
            g.legal_actions()
                .contains(&Action::ChooseJace { object: second })
        );
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::ChooseJace { object: second });
            settle(&mut g);
            assert_eq!(g.counter_count(tokens[0], CounterKind::Loyalty), 1);
            assert_eq!(g.counter_count(tokens[1], CounterKind::Loyalty), 3);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn sanctum_empower_token_and_counter_doubling_keeps_unselected_zero_loyalty_token() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let lurker = put(&mut g, p, "Sanctum Lurker", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: lurker });
        pay(&mut g);
        for _ in 0..48 {
            if g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseJace { .. }))
            {
                break;
            }
            choose(&mut g, |a| {
                matches!(
                    a,
                    Action::Pass | Action::PutTrigger { .. } | Action::ChooseReplacement { .. }
                )
            });
        }
        let choices: Vec<_> = g
            .legal_actions()
            .into_iter()
            .filter_map(|a| match a {
                Action::ChooseJace { object } => Some(object),
                _ => None,
            })
            .collect();
        assert_eq!(choices.len(), 2);
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::ChooseJace { object: choices[0] });
            settle(&mut g);
            assert_eq!(g.counter_count(choices[0].card, CounterKind::Loyalty), 2);
            assert_eq!(g.counter_count(choices[1].card, CounterKind::Loyalty), 0);
            assert_eq!(g.object_view(choices[1].card).zone, Zone::Battlefield);
            g.restore(&saved).unwrap();
        }
    }
}
