//! Paid rules scenarios for authored Wilds of Eldraine cards A–D.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn candidate(game: &mut Game, player: Player, name: &str) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), Zone::Hand)
}

fn target(game: &mut Game, card: CardId) {
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(card),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn pass_twice(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn reach_attackers(game: &mut Game) {
    for _ in 0..16 {
        if game.state.pending == Pending::Attackers {
            return;
        }
        act(game, Action::Pass);
    }
    panic!("attackers did not begin");
}

fn finish_turn(game: &mut Game) {
    let turn = game.state.turn;
    for _ in 0..32 {
        if game.state.turn != turn {
            return;
        }
        let action = match &game.state.pending {
            Pending::Attackers => Action::FinishAttackers,
            Pending::Blockers => Action::FinishBlockers,
            Pending::Priority => Action::Pass,
            pending => panic!("unexpected turn choice {pending:?}"),
        };
        act(game, action);
    }
    panic!("turn did not finish");
}

#[test]
fn break_the_spell_draws_for_qualifying_destruction() {
    let player = Player::P0;
    for (controller, redirect, shield, destination, draws) in [
        (player, false, false, Zone::Graveyard, 1),
        (player.other(), false, false, Zone::Graveyard, 0),
        (player, true, false, Zone::Exile, 1),
        (player, false, true, Zone::Battlefield, 0),
    ] {
        let mut game = scenario(player);
        let enchantment = put(
            &mut game,
            controller,
            "A Tale for the Ages",
            Zone::Battlefield,
        );
        let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let spell = candidate(&mut game, player, "Break the Spell");
        if redirect {
            put(
                &mut game,
                player.other(),
                "Rest in Peace",
                Zone::Battlefield,
            );
        }
        if shield {
            game.add_counter(enchantment, CounterKind::Shield, 1);
        }
        act(&mut game, Action::AnnounceCast { card: spell });
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        target(&mut game, enchantment);
        pay(&mut game);

        if controller == player && !redirect && !shield {
            let mut stale = game.clone();
            act(&mut stale, Action::Pass);
            let removal = put(&mut stale, player.other(), "Disenchant", Zone::Hand);
            cast(&mut stale, removal, Some(Target::Card(enchantment)));
            settle(&mut stale);
            assert_eq!(stale.object_view(enchantment).zone, Zone::Graveyard);
            assert_eq!(stale.state.players[player.index()].cards_drawn_this_turn, 0);
        }

        settle(&mut game);
        assert_eq!(game.object_view(enchantment).zone, destination);
        assert_eq!(game.counter_count(enchantment, CounterKind::Shield), 0);
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            draws
        );
    }
}

#[test]
fn break_the_spell_draws_for_an_opponents_role_token() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let courtier = put(&mut game, opponent, "Cursed Courtier", Zone::Hand);
    let spell = candidate(&mut game, player, "Break the Spell");
    cast(&mut game, courtier, None);
    settle(&mut game);
    let bearer = game.object_ref(courtier);
    let role = game
        .observe(opponent)
        .objects
        .into_iter()
        .find(|object| object.is_token() && object.attached_to == Some(bearer))
        .unwrap();
    assert_eq!(role.controller, opponent);
    assert_eq!(stats(&game, courtier), (1, 1));
    act(&mut game, Action::Pass);
    cast(&mut game, spell, Some(Target::Card(role.id)));
    settle(&mut game);
    assert_eq!(stats(&game, courtier), (3, 3));
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn charging_hooligan_counts_creatures_attacking_when_its_trigger_resolves() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let hooligan = candidate(&mut game, player, "Charging Hooligan");
    let boots = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let rat = put(&mut game, player, "Burglar Rat", Zone::Battlefield);
    let brigade = put(&mut game, player, "Shock Brigade", Zone::Battlefield);
    let mountain = put(&mut game, opponent, "Mountain", Zone::Battlefield);
    let shock = put(&mut game, opponent, "Shock", Zone::Hand);
    cast(&mut game, hooligan, None);
    settle(&mut game);
    assert_eq!(stats(&game, hooligan), (3, 3));
    act(&mut game, activate(boots));
    target(&mut game, hooligan);
    pay(&mut game);
    settle(&mut game);

    reach_attackers(&mut game);
    for card in [hooligan, rat, brigade] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    let trigger = game
        .state
        .ordering_triggers
        .iter()
        .find(|item| item.source.unwrap().id == hooligan)
        .unwrap()
        .id;
    act(&mut game, Action::PutTrigger { id: trigger });
    pass_twice(&mut game);
    assert_eq!(game.state.attackers.len(), 4);
    assert_eq!(game.state.stack.last().unwrap().id, trigger);

    let mut removed = game.clone();
    act(&mut removed, Action::Pass);
    act(&mut removed, Action::TapMana { card: mountain });
    cast(&mut removed, shock, Some(Target::Card(rat)));
    settle(&mut removed);
    assert_eq!(removed.object_view(rat).zone, Zone::Graveyard);
    assert_eq!(stats(&removed, hooligan), (6, 3));
    assert!(
        !removed
            .object_view(hooligan)
            .keywords
            .contains(Keywords::TRAMPLE)
    );

    settle(&mut game);
    assert_eq!(stats(&game, hooligan), (7, 3));
    assert!(
        game.object_view(hooligan)
            .keywords
            .contains(Keywords::TRAMPLE)
    );
    finish_turn(&mut game);
    assert_eq!(stats(&game, hooligan), (3, 3));
    assert!(
        !game
            .object_view(hooligan)
            .keywords
            .contains(Keywords::TRAMPLE)
    );
}

#[test]
fn cooped_up_restricts_combat_and_exiles_its_exact_attachment() {
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let creature = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
        let aura = candidate(&mut game, player, "Cooped Up");
        let disenchant = put(&mut game, opponent, "Disenchant", Zone::Hand);
        let blink = put(&mut game, opponent, "Personify", Zone::Hand);
        cast(&mut game, aura, Some(Target::Card(creature)));
        settle(&mut game);
        let enchanted = game.object_ref(creature);
        assert_eq!(game.object_view(aura).attached_to, Some(enchanted));

        let mut blocking = game.clone();
        reach_attackers(&mut blocking);
        act(&mut blocking, Action::Attack { card: attacker });
        act(&mut blocking, Action::FinishAttackers);
        pass_twice(&mut blocking);
        assert_eq!(blocking.state.pending, Pending::Blockers);
        reject(
            &mut blocking,
            Action::Block {
                blocker: creature,
                attacker,
            },
        );

        let mut attacking = game.clone();
        act(&mut attacking, Action::Pass);
        reject(&mut attacking, activate(aura));
        finish_turn(&mut attacking);
        reach_attackers(&mut attacking);
        assert_eq!(attacking.actor(), Some(opponent));
        reject(&mut attacking, Action::Attack { card: creature });

        act(&mut game, activate(aura));
        let payment = game.mana_payment_domain().unwrap();
        assert_eq!(payment.required, [1, 0, 0, 0, 0, 0]);
        assert_eq!(payment.generic, 2);
        pay(&mut game);
        assert!(game.state.stack.last().unwrap().targets.is_empty());

        let mut removed_aura = game.clone();
        act(&mut removed_aura, Action::Pass);
        cast(&mut removed_aura, disenchant, Some(Target::Card(aura)));
        settle(&mut removed_aura);
        assert_eq!(removed_aura.object_view(aura).zone, Zone::Graveyard);
        assert_eq!(removed_aura.object_view(creature).zone, Zone::Exile);

        let mut blinked = game.clone();
        act(&mut blinked, Action::Pass);
        cast(&mut blinked, blink, Some(Target::Card(creature)));
        settle(&mut blinked);
        assert_eq!(blinked.object_view(creature).zone, Zone::Battlefield);
        assert_ne!(blinked.object_ref(creature), enchanted);

        settle(&mut game);
        assert_eq!(game.object_view(creature).zone, Zone::Exile);
        assert_eq!(game.object_view(aura).zone, Zone::Graveyard);
    }
}

#[test]
fn squeak_by_uses_current_blocker_power_and_exiles_the_adventurer() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let mouse = candidate(&mut game, player, "Cheeky House-Mouse // Squeak By");
    let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let small = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let big = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let forest = put(&mut game, opponent, "Forest", Zone::Battlefield);
    let growth = put(&mut game, opponent, "Giant Growth", Zone::Hand);
    let unsummon = put(&mut game, opponent, "Unsummon", Zone::Hand);

    act(&mut game, Action::AnnounceAdventure { card: mouse });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(small),
        },
    );
    target(&mut game, attacker);
    let payment = game.mana_payment_domain().unwrap();
    assert_eq!(payment.required, [1, 0, 0, 0, 0, 0]);
    assert_eq!(payment.generic, 0);
    pay(&mut game);
    let spell = game.spell_characteristics().pop().unwrap();
    assert_eq!(spell.name, "Squeak By");
    assert_eq!(spell.face, CastFace::Adventure);
    assert_eq!(
        spell.physical_card,
        Card::from_name("Cheeky House-Mouse").unwrap()
    );

    let mut stale = game.clone();
    act(&mut stale, Action::Pass);
    cast(&mut stale, unsummon, Some(Target::Card(attacker)));
    settle(&mut stale);
    assert_eq!(stale.object_view(mouse).zone, Zone::Graveyard);
    assert!(stale.observe(player).blocker_power_limits.is_empty());
    reject(&mut stale, Action::AnnounceExile { card: mouse });

    settle(&mut game);
    assert_eq!(game.object_view(mouse).zone, Zone::Exile);
    assert_eq!(stats(&game, attacker), (3, 3));
    let view = game.observe(player);
    assert_eq!(view.blocker_power_limits.len(), 1);
    assert_eq!(
        view.blocker_power_limits[0].attacker,
        game.object_ref(attacker)
    );
    assert_eq!(view.blocker_power_limits[0].minimum, 3);
    reject(&mut game, Action::AnnounceAdventure { card: mouse });
    act(&mut game, Action::AnnounceExile { card: mouse });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(mouse).zone, Zone::Battlefield);
    assert_eq!(game.object_view(mouse).name, "Cheeky House-Mouse");
    assert_eq!(stats(&game, mouse), (2, 1));

    let mut cleanup = game.clone();
    finish_turn(&mut cleanup);
    assert_eq!(stats(&cleanup, attacker), (2, 2));
    assert!(cleanup.observe(player).blocker_power_limits.is_empty());

    reach_attackers(&mut game);
    act(&mut game, Action::Attack { card: attacker });
    act(&mut game, Action::FinishAttackers);
    pass_twice(&mut game);
    reject(
        &mut game,
        Action::Block {
            blocker: big,
            attacker,
        },
    );
    act(
        &mut game,
        Action::Block {
            blocker: small,
            attacker,
        },
    );
    act(&mut game, Action::FinishBlockers);
    act(&mut game, Action::Pass);
    act(&mut game, Action::TapMana { card: forest });
    cast(&mut game, growth, Some(Target::Card(small)));
    settle(&mut game);
    assert_eq!(stats(&game, small), (5, 5));
    assert!(game.state.blocks.contains(&(small, attacker)));
    assert!(
        game.state
            .attackers
            .iter()
            .any(|row| row.card == attacker && row.blocked)
    );
}
