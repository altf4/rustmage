//! Independent tests for Null Summoner's copied ability-pair origins.
use super::*;

fn resolve_null_link(g: &mut Game, p: Player, card: CardId) {
    for _ in 0..80 {
        if matches!(g.state.pending, Pending::HandCardChoice { .. }) {
            break;
        }
        if g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Player(p.other()),
        }) {
            act(
                g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
        } else {
            choose(g, |a| {
                matches!(
                    a,
                    Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets
                )
            });
        }
    }
    assert!(matches!(g.state.pending, Pending::HandCardChoice { .. }));
    let chosen = g.object_ref(card);
    act(
        g,
        Action::ChooseHandCard {
            object: Some(chosen),
        },
    );
    assert_eq!(g.object_view(card).zone, Zone::Exile);
}

#[test]
fn copied_null_trigger_retains_its_layer_origin_and_uncovering_restores_it() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let card = put(&mut g, p.other(), "Spectral Sailor", Zone::Hand);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source_ref = g.object_ref(source);
        let anchor = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let anchor_ref = g.object_ref(anchor);
        g.apply_temporary_copy(
            source_ref,
            CopyRecipe::new(Card::from_name("Null Summoner").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        let copied_origin = g.copy_ability_origin(source_ref).unwrap();
        // Directly queue the copied entry ability after a modeled cast entry;
        // this tests identity propagation, not entry-replacement discovery.
        let view = g.object_view(source);
        assert!(g.queue_trigger(&view, 0));
        assert!(
            g.state
                .pending_triggers
                .iter()
                .any(|item| item.source_ability_origin == Some(copied_origin))
        );
        g.prepare_triggers();
        resolve_null_link(&mut g, p, card);
        for _ in 0..7 {
            put(&mut g, p, "Forest", Zone::Graveyard);
        }
        assert!(g.can_play_exiled(p, card));
        let before = g.snapshot();
        for _ in 0..2 {
            g.apply_temporary_copy(
                source_ref,
                CopyRecipe::new(Card::from_name("Null Summoner").unwrap()),
                CopyDeadline::WhileSource { source: anchor_ref },
            );
            assert_ne!(g.copy_ability_origin(source_ref), Some(copied_origin));
            assert!(!g.can_play_exiled(p, card));
            g.move_object(anchor, Zone::Graveyard);
            assert_eq!(g.copy_ability_origin(source_ref), Some(copied_origin));
            assert!(g.can_play_exiled(p, card));
            g.restore(&before).unwrap();
        }
    }
}

#[test]
fn projected_sacrifice_expires_covering_copy_and_restores_linked_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let card = put(&mut g, p.other(), "Spectral Sailor", Zone::Hand);
        let source = put(&mut g, p, "Bear Cub", Zone::Hand);
        g.state.objects[source as usize].card = Card::from_name("Null Summoner").unwrap();
        act(&mut g, Action::AnnounceCast { card: source });
        pay(&mut g);
        resolve_null_link(&mut g, p, card);
        for _ in 0..7 {
            put(&mut g, p, "Forest", Zone::Graveyard);
        }
        let anchor = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let anchor_ref = g.object_ref(anchor);
        let ring = put(&mut g, p, "Lotus Ring", Zone::Battlefield);
        let ring_ref = g.object_ref(ring);
        assert!(g.attach_equipment(ring_ref, anchor_ref));
        g.state.objects[anchor as usize].summoning_sick = false;
        let source_ref = g.object_ref(source);
        g.apply_temporary_copy(
            source_ref,
            CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
            CopyDeadline::WhileSource { source: anchor_ref },
        );
        assert!(!g.can_play_exiled(p, card));
        g.state.players[p.index()].mana = [0; 6];
        let purpose = g.spell_mana_purpose_for_card(p, card, g.state.objects[card as usize].card);
        let goal = FundingGoal {
            actor: p,
            required: [0, 1, 0, 0, 0, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        // This probes the funding world directly: ordinary announcement is not
        // yet legal until the source's copy expires.
        assert!(!cast_origins::spends_any_type(&g, p, purpose));
        assert!(g.can_fund_for(goal.clone(), purpose));
        let saved = g.snapshot();
        let action = Action::TapGrantedMana {
            card: anchor,
            grantor: ring_ref,
            production: [0, 0, 0, 3, 0, 0],
        };
        for _ in 0..2 {
            act(&mut g, action.clone());
            assert_eq!(g.object_view(anchor).zone, Zone::Graveyard);
            assert!(g.can_play_exiled(p, card));
            assert!(cast_origins::spends_any_type(&g, p, purpose));
            g.restore(&saved).unwrap();
        }
    }
}
