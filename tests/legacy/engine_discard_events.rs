use super::*;
use super::*;
pub use rustmage_core::engine::discard_events::*;
#[cfg(test)]
mod balin_result_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{named, setup};
    #[test]
    fn discard_context_counts_only_owned_elements() {
        let mut context = DiscardContext {
            sources: Vec::new(),
            cycled: None,
            cycling_x: None,
            shuffles: Vec::new(),
            after: None,
        };
        assert_eq!(context.owned_bytes(), 0);
        context.shuffles.push(ObjectRef {
            card: 0,
            generation: 0,
        });
        assert_eq!(context.owned_bytes(), std::mem::size_of::<ObjectRef>());
    }

    #[test]
    fn captured_view_counts_abilities_and_power_up_uses() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Forest", Zone::Battlefield);
        let mut view = game.object_view(id);
        view.abilities = None;
        view.power_up_uses.clear();
        let before = view.owned_bytes();
        view.abilities = Some(Box::new(AbilityAvailabilityView {
            printed: true,
            grants: vec![GrantedAbilityView {
                grantor: view.reference,
                granting_card: view.card,
                ability: Some(0),
                static_rule: None,
                grantor_controller: Player::P0,
                timestamp: 0,
            }],
        }));
        view.power_up_uses.push(PowerUpUseView {
            definition: view.card,
            ability: 0,
            borrowed_slot: None,
            origin: None,
            count: 1,
        });
        assert_eq!(
            view.owned_bytes(),
            before
                + std::mem::size_of::<AbilityAvailabilityView>()
                + std::mem::size_of::<GrantedAbilityView>()
                + std::mem::size_of::<PowerUpUseView>()
        );
        let context = DiscardContext {
            sources: vec![view],
            cycled: None,
            cycling_x: None,
            shuffles: Vec::new(),
            after: None,
        };
        assert_eq!(
            context.owned_bytes(),
            std::mem::size_of::<ObjectView>() + context.sources[0].owned_bytes()
        );
    }

    fn root(p: Player) -> Game {
        let mut g = setup(p);
        named(&mut g, p, "Balin, Loremaster", Zone::Battlefield);
        let dwarf = named(&mut g, p, "Dwarven Mauler", Zone::Hand);
        g.move_object(dwarf, Zone::Battlefield);
        g.give_priority();
        g.prepare_triggers();
        for _ in 0..32 {
            if g.legal_actions()
                .contains(&Action::ChooseOptional { accept: true })
            {
                return g;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                .unwrap();
            g.apply(g.actor().unwrap(), a).unwrap();
        }
        panic!("result fixture did not reach Balin optional");
    }
    #[test]
    fn hob_balin_result_empty_batch_overwrites_previous_result() {
        for p in [Player::P0, Player::P1] {
            let mut g = root(p);
            g.state.resolving.as_mut().unwrap().moved_count = Some(99);
            g.apply(p, Action::ChooseOptional { accept: true }).unwrap();
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            assert_eq!(g.state.players[p.other().index()].life, 20);
        }
    }
    #[test]
    fn hob_balin_result_linked_entry_pause_counts_completed_not_skipped_moves_and_restores() {
        // Synthetic replacement destination isolates the generic discard-context
        // contract. Balin itself cannot cause the opponent-only Wilt-Leaf entry.
        for p in [Player::P0, Player::P1] {
            let mut g = root(p);
            let adaptive = named(&mut g, p, "Adaptive Automaton", Zone::Hand);
            let shock = named(&mut g, p, "Shock", Zone::Hand);
            let prevented = named(&mut g, p, "Forest", Zone::Hand);
            let r = g.state.resolving.as_mut().unwrap();
            r.program = &[
                Instruction::DrawMoved,
                Instruction::DamagePlayersScaled(PlayerSet::Opponent, AmountValue::MovedCount),
            ];
            r.pc = 0;
            r.moved_count = Some(99);
            g.state.pending = Pending::Priority;
            let mut skip = g.entry_move(prevented, Zone::Graveyard, false);
            skip.skipped = true;
            g.stage_discard_entries(
                vec![
                    g.entry_move(adaptive, Zone::Battlefield, false),
                    g.entry_move(shock, Zone::Graveyard, false),
                    skip,
                ],
                DiscardContext {
                    sources: Vec::new(),
                    cycled: None,
                    cycling_x: None,
                    shuffles: Vec::new(),
                    after: Some(DiscardAfter::RememberActual),
                },
            );
            assert!(g.advance_entries());
            assert!(
                g.state
                    .entry_batch
                    .as_ref()
                    .unwrap()
                    .discard_context
                    .is_some()
            );
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            let saved = g.snapshot();
            let mut raw = g.clone();
            let mut indexed = g.clone();
            drop(g);
            let actions = raw.legal_actions();
            let index = actions
                .iter()
                .position(|a| matches!(a, Action::ChooseEntryValue { .. }))
                .unwrap();
            let action = actions[index].clone();
            let revision = indexed.revision();
            indexed.apply_index(p, revision, index).unwrap();
            raw.apply(p, action.clone()).unwrap();
            assert_eq!(raw.snapshot(), indexed.snapshot());
            assert_eq!(raw.state.players[p.index()].cards_discarded_this_turn, 2);
            assert_eq!(raw.state.players[p.index()].cards_drawn_this_turn, 2);
            assert_eq!(raw.state.players[p.other().index()].life, 18);
            assert_eq!(raw.object_view(prevented).zone, Zone::Hand);
            indexed.restore(&saved).unwrap();
            indexed.apply(p, action).unwrap();
            assert_eq!(raw.snapshot(), indexed.snapshot());
        }
    }
}
#[cfg(test)]
#[path = "engine_silvan_discard_tests.rs"]
mod silvan_discard_tests;
