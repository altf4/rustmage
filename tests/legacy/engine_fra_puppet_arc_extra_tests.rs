use super::*;

#[test]
fn puppet_cast_requires_green_and_generic_and_fizzles_on_target_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let artifact = put(&mut g, p, "Hedron Archive", Zone::Battlefield);
        // Archive otherwise supplies {C}{C}, funding the generic cost even
        // when the initial mana pool contains only green.
        g.state.objects[artifact as usize].tapped = true;
        let invalid = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let aura = put(&mut g, p, "Puppet Crafting", Zone::Hand);
        for mana in [[0, 0, 0, 0, 0, 2], [0, 0, 0, 0, 1, 0]] {
            g.state.players[p.index()].mana = mana;
            reject(&mut g, Action::AnnounceCast { card: aura });
            assert_eq!(g.object_view(aura).zone, Zone::Hand);
        }
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::AnnounceCast { card: aura });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(invalid),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(artifact),
                },
            );
            pay(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            g.move_object(artifact, Zone::Exile);
            settle(&mut g);
            assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
            assert_eq!(g.object_view(artifact).zone, Zone::Exile);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn puppet_enchants_artifact_token_and_goes_to_graveyard_when_token_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.create_tokens(p, Card::test_index(5246), 1, false); // Food
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && o.card == Card::test_index(5246))
                .then_some(id as CardId)
            })
            .unwrap();
        let aura = put(&mut g, p, "Puppet Crafting", Zone::Hand);
        cast(&mut g, aura, Some(Target::Card(token)));
        settle(&mut g);
        assert_eq!(g.object_view(aura).attached_to, Some(g.object_ref(token)));
        assert_eq!(stats(&g, token), (5, 5));
        assert!(g.has_subtype(token, "Food"));
        assert!(g.has_subtype(token, "Construct"));
        let saved = g.snapshot();
        for _ in 0..2 {
            g.move_object(token, Zone::Exile);
            g.state_based_actions();
            assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
            assert!(!g.on_battlefield(token));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn arc_short_first_library_still_completes_other_players_draw_before_sba() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Variable Chaser // Arc of Fortune", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
        act(&mut g, Action::AnnounceCast { card: source });
        pay(&mut g);
        settle(&mut g);
        let permanent = g.object_ref(source);
        let prepared = g
            .state
            .preparations
            .as_ref()
            .unwrap()
            .iter()
            .find(|r| r.permanent == permanent)
            .unwrap()
            .spell_copy;
        for card in g.state.players[p.index()].library.clone() {
            g.move_object(card, Zone::Outside);
        }
        put(&mut g, p, "Forest", Zone::Library);
        put(&mut g, p, "Island", Zone::Hand);
        put(&mut g, p.other(), "Forest", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
        act(
            &mut g,
            Action::AnnounceExile {
                card: prepared.card,
            },
        );
        pay(&mut g);
        for _ in 0..64 {
            if matches!(g.state.pending, Pending::EffectOptional { .. }) {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        act(&mut g, Action::ChooseOptional { accept: true });
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::ChooseOptional { accept: true });
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
            assert!(g.state.players[p.index()].attempted_empty_draw);
            assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 7);
            assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
            assert_eq!(
                g.state.players[p.other().index()].cards_discarded_this_turn,
                1
            );
            assert_eq!(g.outcome(), Some(Outcome::Winner(p.other())));
            g.restore(&saved).unwrap();
        }
    }
}
