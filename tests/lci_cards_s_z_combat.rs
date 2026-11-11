//! Printed attack and untap rules through the exact Ixalan definitions.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn member(game: &mut Game, player: Player, name: &str) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), Zone::Hand)
}

fn enter_attackers(game: &mut Game) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
}

fn advance_until(game: &mut Game, action: &Action) {
    for _ in 0..16 {
        if game.legal_actions().contains(action) {
            return;
        }
        choose(game, |choice| {
            matches!(choice, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("never offered {action:?}");
}

#[test]
fn shipwreck_sentry_observes_own_artifact_entry_even_after_it_leaves() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let sentry = member(&mut game, player, "Shipwreck Sentry");
        cast(&mut game, sentry, None);
        settle(&mut game);
        assert_eq!(stats(&game, sentry), (3, 3));
        game.state.objects[sentry as usize].summoning_sick = false;
        let foreign = put(&mut game, player.other(), "Short Bow", Zone::Hand);
        game.move_object(foreign, Zone::Battlefield);
        let mut no_entry = game.clone();
        enter_attackers(&mut no_entry);
        reject(&mut no_entry, Action::Attack { card: sentry });

        let artifact = put(&mut game, player, "Short Bow", Zone::Hand);
        cast(&mut game, artifact, None);
        settle(&mut game);
        game.move_object(artifact, Zone::Graveyard);
        assert!(game.keywords(sentry).contains(Keywords::DEFENDER));
        let mut next_turn = game.clone();
        next_turn.begin_turn();
        enter_attackers(&mut next_turn);
        reject(&mut next_turn, Action::Attack { card: sentry });

        enter_attackers(&mut game);
        act(&mut game, Action::Attack { card: sentry });
        act(&mut game, Action::FinishAttackers);
        assert!(
            game.state
                .attackers
                .iter()
                .any(|attack| attack.card == sentry)
        );
    }
}

#[test]
fn thousand_moons_infantry_untaps_on_the_opponents_step_and_respects_stun() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let infantry = member(&mut game, player, "Thousand Moons Infantry");
        cast(&mut game, infantry, None);
        settle(&mut game);
        assert_eq!(stats(&game, infantry), (2, 4));
        let ordinary = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        for object in [infantry, ordinary] {
            game.state.objects[object as usize].tapped = true;
        }
        game.state.step = Step::End;
        let mut stunned = game.clone();
        stunned.add_counter(infantry, CounterKind::Stun, 1);
        for branch in [&mut game, &mut stunned] {
            act(branch, Action::Pass);
            act(branch, Action::Pass);
            assert_eq!(branch.state.active, player.other());
            assert_eq!(branch.state.step, Step::Upkeep);
            assert!(branch.object_view(ordinary).tapped);
        }
        assert!(!game.object_view(infantry).tapped);
        assert!(stunned.object_view(infantry).tapped);
        assert_eq!(stunned.counter_count(infantry, CounterKind::Stun), 0);
    }
}

#[test]
fn thousand_moons_crackshot_selects_a_creature_only_after_the_exact_optional_payment() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let crackshot = member(&mut game, player, "Thousand Moons Crackshot");
        cast(&mut game, crackshot, None);
        settle(&mut game);
        assert_eq!(stats(&game, crackshot), (2, 2));
        let rival = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let artifact = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
        game.state.objects[crackshot as usize].summoning_sick = false;
        enter_attackers(&mut game);
        act(&mut game, Action::Attack { card: crackshot });
        act(&mut game, Action::FinishAttackers);
        let pay_cost = Action::PayResolutionMana {
            payment: [1, 0, 0, 0, 0, 2],
        };
        let choose_creature = Action::ChooseTarget {
            target: Target::Card(rival),
        };
        advance_until(&mut game, &pay_cost);
        assert!(matches!(game.state.pending, Pending::ResolutionMana { .. }));
        assert!(!game.legal_actions().contains(&choose_creature));
        reject(
            &mut game,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 0, 3],
            },
        );

        let mut declined = game.clone();
        act(&mut declined, Action::DeclineResolutionMana);
        settle(&mut declined);
        assert!(!declined.object_view(rival).tapped);

        act(&mut game, pay_cost);
        advance_until(&mut game, &choose_creature);
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        act(&mut game, choose_creature);
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        settle(&mut game);
        assert!(game.object_view(rival).tapped);
        assert!(!game.object_view(artifact).tapped);
    }
}
