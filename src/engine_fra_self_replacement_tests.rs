//! A synthetic resolving-effect fixture: on its controller's turn the effect
//! replaces its own poison event with life gain. This tests the existing typed
//! conditional-program representation; no Standard card is claimed as source.
use super::*;

fn self_replacing_effect(g: &mut Game, player: Player, source: CardId) {
    let item = g.activation_item(player, source, 0);
    g.state.resolving = Some(Resolution {
        chosen_creature_type: None,
        target_counter_removal: None,
        linked_copy: None,
        cascade: None,
        target_card_facts: None,
        sacrificed_power: None,
        immediate_blink_return: None,
        connive: None,
        optional_wheel: None,
        proliferate: None,
        explore: None,
        evidence: None,
        item,
        mana_payment: None,
        forage: None,
        target_library: None,
        program: &[
            Instruction::If {
                guard: StepGuard::YourTurn,
                skip: 2,
            },
            Instruction::GainLife(PlayerSet::Controller, 2),
            Instruction::Skip(1),
            Instruction::AddPoison(PlayerSet::Controller, 1),
        ],
        pc: 0,
        target_group: 0,
        moved_count: None,
        remembered_draws: None,
        discard_after: None,
        discards: [0; 2],
        selected_discards: [vec![], vec![]],
        milled: vec![],
        milled_selection: None,
        library: None,
        selection: None,
        return_attachment: None,
        search: None,
        search_result: None,
        optional_searches: None,
        choice: None,
    });
    g.run_program();
}

#[test]
fn conditional_effect_self_replacement_precedes_poison_prohibition() {
    let _fixture = crate::cards::test_counter_providers::activate();
    for player in [Player::P0, Player::P1] {
        for own_turn in [false, true] {
            let mut g = scenario(if own_turn { player } else { player.other() });
            let source = put(
                &mut g,
                player,
                "Doc Samson, Super Psychiatrist",
                Zone::Battlefield,
            );
            put(&mut g, player, "Bear Cub", Zone::Battlefield); // poison prohibition
            put(&mut g, player, "Hill Giant", Zone::Battlefield); // ordinary doubler
            let before = g.snapshot();
            let life = g.state.players[player.index()].life;
            for _ in 0..2 {
                self_replacing_effect(&mut g, player, source);
                assert_eq!(
                    g.state.players[player.index()].life,
                    life + if own_turn { 2 } else { 0 }
                );
                assert_eq!(g.state.players[player.index()].poison_counters, 0);
                assert!(!matches!(g.state.pending, Pending::Replacement { .. }));
                g.restore(&before).unwrap();
            }
        }
    }
}
