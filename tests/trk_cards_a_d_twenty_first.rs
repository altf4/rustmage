//! Antagonist Incorporator grants each artifact creature its own optional hit trigger.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

#[test]
fn antagonist_grants_self_and_borg_token_but_not_nonartifact_creature() {
    let mut game = scenario(Player::P0);
    let source = put(
        &mut game,
        Player::P0,
        "Antagonist Incorporator",
        Zone::Battlefield,
    );
    let mana_land = put(&mut game, Player::P0, "Forest", Zone::Battlefield);
    let ordinary = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    game.create_tokens(Player::P0, Card(6007), 1, false);
    assert!(!game.advance_replacements(false));
    game.advance_entries();
    let token = game
        .state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, object)| {
            (object.zone == Zone::Battlefield && object.card == Card(6007)).then_some(id as CardId)
        })
        .unwrap();
    for id in [source, ordinary, token] {
        game.state.objects[id as usize].summoning_sick = false;
    }
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for card in [source, ordinary, token] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);

    let mut payments = 0;
    for _ in 0..96 {
        if game.state.step == Step::EndCombat
            && game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            break;
        }
        let actions = game.legal_actions();
        if actions.contains(&Action::DeclineResolutionMana) {
            payments += 1;
            if payments == 1 {
                if !actions
                    .iter()
                    .any(|action| matches!(action, Action::PayResolutionMana { .. }))
                {
                    act(&mut game, Action::TapMana { card: mana_land });
                }
                choose(&mut game, |action| {
                    matches!(action, Action::PayResolutionMana { .. })
                });
            } else {
                act(&mut game, Action::DeclineResolutionMana);
            }
        } else {
            choose(&mut game, |action| {
                matches!(
                    action,
                    Action::Pass | Action::PutTrigger { .. } | Action::FinishBlockers
                ) || matches!(
                    action,
                    Action::AssignDamage {
                        target: Target::Player(Player::P1),
                        ..
                    }
                )
            });
        }
    }
    assert_eq!(
        payments, 2,
        "only the two artifact creatures grant triggers"
    );
    assert_eq!(
        game.state
            .objects
            .iter()
            .filter(|object| object.zone == Zone::Battlefield && object.card == Card(6007))
            .count(),
        2,
        "paying for one trigger creates exactly one additional Borg token"
    );
}
