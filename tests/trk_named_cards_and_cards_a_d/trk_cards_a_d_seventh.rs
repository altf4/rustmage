//! Star Trek A–D attack payment and entry-mode scenarios.
use crate::support::*;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};

fn owned(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}
fn drain(game: &mut Game) {
    for _ in 0..50 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("resolution budget");
}
fn reach_mode(game: &mut Game, mode: u16) {
    for _ in 0..32 {
        if game.legal_actions().contains(&Action::ChooseMode { mode }) {
            act(game, Action::ChooseMode { mode });
            return;
        }
        choose(game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("modal choice unavailable");
}

#[test]
fn automated_warfare_attack_draws_only_after_another_permanent_is_sacrificed() {
    for paid in [false, true] {
        let mut game = scenario(Player::P0);
        let source = owned(
            &mut game,
            Player::P0,
            "Automated Warfare System",
            Zone::Battlefield,
        );
        let artifact = put(&mut game, Player::P0, "Tricorder", Zone::Battlefield);
        let top = put(&mut game, Player::P0, "Forest", Zone::Library);
        game.state.objects[source as usize].summoning_sick = false;
        assert!(game.object_view(source).keywords.contains(Keywords::FLYING));
        game.state.step = Step::Attackers;
        game.state.pending = Pending::Attackers;
        act(&mut game, Action::Attack { card: source });
        act(&mut game, Action::FinishAttackers);
        for _ in 0..32 {
            if matches!(game.state.pending, Pending::EffectChoice { .. }) {
                break;
            }
            choose(&mut game, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert!(matches!(game.state.pending, Pending::EffectChoice { .. }));
        if paid {
            let artifact_ref = game.object_ref(artifact);
            act(
                &mut game,
                Action::Sacrifice {
                    object: artifact_ref,
                },
            );
        } else {
            act(&mut game, Action::DeclineEffectPayment);
        }
        drain(&mut game);
        assert_eq!(
            game.object_view(artifact).zone,
            if paid {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(
            game.object_view(top).zone,
            if paid { Zone::Hand } else { Zone::Library }
        );
    }
}

#[test]
fn raffi_modes_exile_optional_grave_card_or_make_clue_with_life_change() {
    for mode in [0, 1] {
        let mut game = scenario(Player::P0);
        let grave = put(&mut game, Player::P1, "Grizzly Bears", Zone::Graveyard);
        let source = owned(&mut game, Player::P0, "Commander Raffi Musiker", Zone::Hand);
        cast(&mut game, source, None);
        reach_mode(&mut game, mode);
        if mode == 0 {
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(grave),
                },
            );
            if game.legal_actions().contains(&Action::FinishTargets) {
                act(&mut game, Action::FinishTargets);
            }
        }
        drain(&mut game);
        assert_eq!(game.state.players[0].life, if mode == 0 { 22 } else { 18 });
        assert_eq!(
            game.object_view(grave).zone,
            if mode == 0 {
                Zone::Exile
            } else {
                Zone::Graveyard
            }
        );
        let clues = game
            .state
            .objects
            .iter()
            .filter(|o| {
                o.zone == Zone::Battlefield && o.controller == Player::P0 && o.card == Card(5296)
            })
            .count();
        assert_eq!(clues, if mode == 1 { 1 } else { 0 });
    }
}
