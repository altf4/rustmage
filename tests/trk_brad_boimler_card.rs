//! Brad Boimler's printed tap trigger and all-kind counter increase.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use rustmage_core::engine::replacements::{CounterPlacement, ResumeMode};
use support::*;

fn put_counters(game: &mut Game, card: CardId, amount: Counters) {
    let view = game.object_view(card);
    game.stage_counter_placements(
        vec![CounterPlacement {
            object: view.reference,
            placed_by: view.controller,
            card: view.card,
            owner: view.owner,
            controller: view.controller,
            protector: view.battle_protector,
            subtypes: view.granted_subtypes,
            kind: view.kinds,
            power: view.power,
            amount,
            effect: true,
            entry_index: None,
            private: false,
        }],
        ResumeMode::Pending,
    );
    game.advance_replacements(true);
    assert!(game.observe(game.state.active).replacement_choice.is_none());
}

#[test]
fn attack_installs_replacement_for_controlled_permanents_and_expires_at_cleanup() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let brad = put(&mut game, player, "Brad Boimler, Eager Ensign", Zone::Battlefield);
        let target = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut game, player.other(), "Grizzly Bears", Zone::Battlefield);
        assert_eq!(stats(&game, brad), (2, 2));
        assert!(game.keywords(brad).contains(Keywords::LIFELINK));

        game.state.step = Step::Attackers;
        game.state.pending = Pending::Attackers;
        assert!(game.legal_actions().contains(&Action::Attack { card: brad }));
        act(&mut game, Action::Attack { card: brad });
        act(&mut game, Action::FinishAttackers);
        for _ in 0..8 {
            if game.state.pending == Pending::Priority && game.state.stack.is_empty()
                && game.state.pending_triggers.is_empty() {
                break;
            }
            choose(&mut game, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
        }
        assert_eq!(game.observe(player).permanent_counter_increases.len(), 1);
        assert_eq!(game.observe(player.other()).permanent_counter_increases.len(), 1);

        let mut mixed = Counters::of(CounterKind::PlusOne, 2);
        mixed.add(CounterKind::Stun, 1);
        put_counters(&mut game, target, mixed);
        assert_eq!(game.counter_count(target, CounterKind::PlusOne), 3);
        assert_eq!(game.counter_count(target, CounterKind::Stun), 2);
        put_counters(&mut game, enemy, Counters::of(CounterKind::PlusOne, 1));
        assert_eq!(game.counter_count(enemy, CounterKind::PlusOne), 1);

        game.state.step = Step::End;
        game.state.pending = Pending::Priority;
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert!(game.observe(player).permanent_counter_increases.is_empty());
    }
}
