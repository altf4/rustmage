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
    activate_slot(card, 0)
}

fn activate_slot(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
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
    assert!(!removed
        .object_view(hooligan)
        .keywords
        .contains(Keywords::TRAMPLE));

    settle(&mut game);
    assert_eq!(stats(&game, hooligan), (7, 3));
    assert!(game
        .object_view(hooligan)
        .keywords
        .contains(Keywords::TRAMPLE));
    finish_turn(&mut game);
    assert_eq!(stats(&game, hooligan), (3, 3));
    assert!(!game
        .object_view(hooligan)
        .keywords
        .contains(Keywords::TRAMPLE));
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
    assert!(game
        .state
        .attackers
        .iter()
        .any(|row| row.card == attacker && row.blocked));
}

fn bargain(game: &mut Game, offering: Option<CardId>) {
    let sacrifice = offering.map(|card| game.object_ref(card));
    act(game, Action::ChooseBargain { sacrifice });
}

#[test]
fn agathas_champion_fights_only_when_bargained_and_with_both_creatures_present() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let champion = candidate(&mut game, player, "Agatha's Champion");
    let bow = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let giant = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let bounce = put(&mut game, opponent, "Unsummon", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: champion });

    let mut unpaid = game.clone();
    bargain(&mut unpaid, None);
    pay(&mut unpaid);
    settle(&mut unpaid);
    assert_eq!(stats(&unpaid, champion), (4, 4));
    assert!(unpaid
        .object_view(champion)
        .keywords
        .contains(Keywords::TRAMPLE));
    assert_eq!(unpaid.object_view(bow).zone, Zone::Battlefield);
    assert_eq!(unpaid.object_view(giant).damage, 0);

    bargain(&mut game, Some(bow));
    let cost = game.mana_payment_domain().unwrap();
    assert_eq!(cost.required, [0, 0, 0, 0, 1, 0]);
    assert_eq!(cost.generic, 4);
    assert_eq!(game.object_view(bow).zone, Zone::Battlefield);
    pay(&mut game);
    assert_eq!(game.object_view(bow).zone, Zone::Graveyard);
    pass_twice(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    let mut no_target = game.clone();
    act(&mut no_target, Action::FinishTargets);
    settle(&mut no_target);
    assert_eq!(no_target.object_view(champion).damage, 0);
    assert_eq!(no_target.object_view(giant).damage, 0);

    target(&mut game, giant);
    for (bounced, survivor) in [(champion, giant), (giant, champion)] {
        let mut stale = game.clone();
        act(&mut stale, Action::Pass);
        cast(&mut stale, bounce, Some(Target::Card(bounced)));
        settle(&mut stale);
        assert_eq!(stale.object_view(bounced).zone, Zone::Hand);
        assert_eq!(stale.object_view(survivor).zone, Zone::Battlefield);
        assert_eq!(stale.object_view(survivor).damage, 0);
    }

    settle(&mut game);
    assert_eq!(game.object_view(giant).zone, Zone::Graveyard);
    let champ = game.object_view(champion);
    assert_eq!(champ.zone, Zone::Battlefield);
    assert_eq!(champ.damage, 3);
    assert!(champ.keywords.contains(Keywords::TRAMPLE));
}

#[test]
fn back_for_seconds_routes_surviving_targets_with_optional_bargained_entry() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = candidate(&mut game, player, "Back for Seconds");
    let offering = put(&mut game, player, "A Tale for the Ages", Zone::Battlefield);
    let four = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let five = put(&mut game, player, "Beanstalk Wurm", Zone::Graveyard);
    let foreign = put(&mut game, opponent, "Bear Cub", Zone::Graveyard);
    let land = put(&mut game, player, "Forest", Zone::Graveyard);
    let ooze = put(&mut game, opponent, "Scavenging Ooze", Zone::Battlefield);
    act(&mut game, Action::AnnounceCast { card: spell });

    let mut unpaid = game.clone();
    bargain(&mut unpaid, None);
    act(
        &mut unpaid,
        Action::ChooseTarget {
            target: Target::Card(four),
        },
    );
    target(&mut unpaid, five);
    pay(&mut unpaid);
    settle(&mut unpaid);
    assert_eq!(unpaid.object_view(four).zone, Zone::Hand);
    assert_eq!(unpaid.object_view(five).zone, Zone::Hand);
    assert_eq!(unpaid.object_view(offering).zone, Zone::Battlefield);

    bargain(&mut game, Some(offering));
    for invalid in [land, foreign] {
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(invalid),
            },
        );
    }
    let mut no_targets = game.clone();
    act(&mut no_targets, Action::FinishTargets);
    pay(&mut no_targets);
    settle(&mut no_targets);
    assert_eq!(no_targets.object_view(four).zone, Zone::Graveyard);
    assert_eq!(no_targets.object_view(five).zone, Zone::Graveyard);
    assert_eq!(no_targets.object_view(offering).zone, Zone::Graveyard);

    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(four),
        },
    );
    target(&mut game, five);
    pay(&mut game);
    assert_eq!(game.object_view(offering).zone, Zone::Graveyard);

    let mut stale = game.clone();
    act(&mut stale, Action::Pass);
    act(&mut stale, activate(ooze));
    target(&mut stale, four);
    pay(&mut stale);
    settle(&mut stale);
    assert_eq!(stale.object_view(four).zone, Zone::Exile);
    assert_eq!(stale.object_view(five).zone, Zone::Hand);

    pass_twice(&mut game);
    let four = game.object_ref(four);
    let five = game.object_ref(five);
    let choice = game.observe(player).effect_choice.unwrap();
    assert!(choice.optional);
    assert_eq!(choice.candidates, [four]);
    reject(&mut game, Action::SelectGraveyardReturn { object: five });

    let mut declined = game.clone();
    act(&mut declined, Action::FinishGraveyardReturn);
    assert_eq!(declined.object_view(four.card).zone, Zone::Hand);
    assert_eq!(declined.object_view(five.card).zone, Zone::Hand);

    act(&mut game, Action::SelectGraveyardReturn { object: four });
    settle(&mut game);
    let revived = game.object_view(four.card);
    assert_eq!(revived.zone, Zone::Battlefield);
    assert_eq!(revived.controller, player);
    assert!(!revived.tapped);
    assert_ne!(revived.reference, four);
    assert_eq!(game.object_view(five.card).zone, Zone::Hand);
}

#[test]
fn brave_the_wilds_searches_on_either_route_and_animates_its_paid_target() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = candidate(&mut game, player, "Brave the Wilds");
    let offering = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let foreign = put(&mut game, opponent, "Island", Zone::Battlefield);
    let nonland = put(&mut game, player, "Bear Cub", Zone::Library);
    let found = put(&mut game, player, "Mountain", Zone::Library);

    let mut unpaid = game.clone();
    act(&mut unpaid, Action::AnnounceCast { card: spell });
    let sacrifice = Some(unpaid.object_ref(offering));
    reject(&mut unpaid, Action::ChooseBargain { sacrifice });
    bargain(&mut unpaid, None);
    pay(&mut unpaid);
    assert!(unpaid.state.stack.last().unwrap().targets.is_empty());
    pass_twice(&mut unpaid);
    let choice = unpaid.observe(player).search_choice.unwrap();
    assert_eq!(choice.filter, SearchFilter::BasicLand);
    assert!(choice.reveal);
    assert!(unpaid.observe(opponent).search_choice.is_none());
    reject(&mut unpaid, Action::ChooseSearch { card: nonland });

    let mut fail_to_find = unpaid.clone();
    let rng = fail_to_find.state.rng;
    act(&mut fail_to_find, Action::FinishSearch);
    assert_ne!(fail_to_find.state.rng, rng);
    assert_eq!(fail_to_find.object_view(found).zone, Zone::Library);

    act(&mut unpaid, Action::ChooseSearch { card: found });
    act(&mut unpaid, Action::FinishSearch);
    assert_eq!(unpaid.object_view(found).zone, Zone::Hand);
    assert!(unpaid
        .observe(opponent)
        .known_hand
        .contains(&unpaid.object_ref(found)));
    assert_eq!(unpaid.object_view(offering).zone, Zone::Battlefield);

    let forest = put(&mut game, player, "Forest", Zone::Hand);
    let fetch = put(&mut game, player, "Evolving Wilds", Zone::Battlefield);
    act(&mut game, Action::PlayLand { card: forest });
    act(&mut game, Action::AnnounceCast { card: spell });
    bargain(&mut game, Some(offering));
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(foreign),
        },
    );
    reject(&mut game, Action::FinishTargets);

    let mut stale = game.clone();
    target(&mut stale, fetch);
    pay(&mut stale);
    act(&mut stale, activate(fetch));
    pay(&mut stale);
    pass_twice(&mut stale);
    act(&mut stale, Action::FinishSearch);
    let rng = stale.state.rng;
    settle(&mut stale);
    assert_eq!(stale.object_view(fetch).zone, Zone::Graveyard);
    assert_eq!(stale.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(stale.state.rng, rng);
    assert_eq!(stale.object_view(found).zone, Zone::Library);

    target(&mut game, forest);
    pay(&mut game);
    assert_eq!(game.object_view(offering).zone, Zone::Graveyard);
    pass_twice(&mut game);
    let land = game.object_view(forest);
    assert_eq!(land.kinds & 3, 3);
    assert!(land.has_subtype("Forest"));
    assert!(land.has_subtype("Elemental"));
    assert!(land.keywords.contains(Keywords::HASTE));
    assert_eq!(stats(&game, forest), (3, 3));
    act(&mut game, Action::ChooseSearch { card: found });
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.object_view(found).zone, Zone::Hand);
    assert!(game
        .observe(opponent)
        .known_hand
        .contains(&game.object_ref(found)));

    let mut mana = game.clone();
    let green = mana.state.players[player.index()].mana[4];
    act(&mut mana, Action::TapMana { card: forest });
    assert_eq!(mana.state.players[player.index()].mana[4], green + 1);

    reach_attackers(&mut game);
    act(&mut game, Action::Attack { card: forest });
    finish_turn(&mut game);
    assert_eq!(stats(&game, forest), (3, 3));
    assert!(game.object_view(forest).keywords.contains(Keywords::HASTE));
}

#[test]
fn agatha_uses_live_power_and_boosts_only_other_creatures() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let agatha = candidate(&mut game, player, "Agatha of the Vile Cauldron");
    let keykeeper = put(&mut game, player, "Sterling Keykeeper", Zone::Battlefield);
    let giant = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let growth = put(&mut game, player, "Giant Growth", Zone::Hand);
    let late = put(&mut game, player, "Bear Cub", Zone::Hand);
    cast(&mut game, agatha, None);
    settle(&mut game);
    assert_eq!(stats(&game, agatha), (1, 1));

    let mut original_power = game.clone();
    act(&mut original_power, activate(agatha));
    assert_eq!(original_power.mana_payment_domain().unwrap().generic, 3);

    cast(&mut game, growth, Some(Target::Card(agatha)));
    settle(&mut game);
    assert_eq!(stats(&game, agatha), (4, 4));
    let mut without_mana = game.clone();
    without_mana.state.players[player.index()].mana = [0; 6];
    reject(&mut without_mana, activate(keykeeper));
    act(&mut game, activate(keykeeper));
    target(&mut game, giant);
    let cost = game.mana_payment_domain().unwrap();
    assert_eq!(cost.generic, 1);
    assert_eq!(cost.required, [0; 6]);
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(giant).tapped);

    act(&mut game, activate(agatha));
    let cost = game.mana_payment_domain().unwrap();
    assert_eq!(cost.generic, 0);
    assert_eq!(cost.required, [0, 0, 0, 1, 1, 0]);
    pay(&mut game);
    settle(&mut game);
    let granted = Keywords::TRAMPLE.union(Keywords::HASTE);
    assert_eq!(stats(&game, agatha), (4, 4));
    assert_eq!(game.object_view(agatha).keywords, Keywords::NONE);
    assert_eq!(stats(&game, giant), (3, 3));
    assert_eq!(game.object_view(giant).keywords, Keywords::NONE);
    assert_eq!(stats(&game, keykeeper), (3, 3));
    assert!(game.object_view(keykeeper).keywords.contains(granted));

    cast(&mut game, late, None);
    settle(&mut game);
    assert_eq!(stats(&game, late), (2, 2));
    assert_eq!(game.object_view(late).keywords, Keywords::NONE);
    finish_turn(&mut game);
    assert_eq!(stats(&game, agatha), (1, 1));
    assert_eq!(stats(&game, keykeeper), (2, 2));
    assert_eq!(game.object_view(keykeeper).keywords, Keywords::NONE);
}

fn select_land(game: &Game, card: CardId) -> Action {
    Action::SelectLand {
        object: game.object_ref(card),
    }
}

#[test]
fn blossoming_tortoise_recovers_lands_and_modifies_live_land_activations_and_stats() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let tortoise = candidate(&mut game, player, "Blossoming Tortoise");
    let agatha = candidate(&mut game, player, "Agatha of the Vile Cauldron");
    let boots = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let land = put(&mut game, player, "Soulstone Sanctuary", Zone::Battlefield);
    let opposing_land = put(
        &mut game,
        opponent,
        "Soulstone Sanctuary",
        Zone::Battlefield,
    );
    let old = put(&mut game, player, "Plains", Zone::Graveyard);
    let foreign = put(&mut game, opponent, "Island", Zone::Graveyard);
    let attack_land = put(&mut game, player, "Mountain", Zone::Library);
    let attack_spells = ["Shock", "Opt"].map(|name| put(&mut game, player, name, Zone::Library));
    let entry_land = put(&mut game, player, "Island", Zone::Library);
    let entry_spells = ["Opt", "Shock"].map(|name| put(&mut game, player, name, Zone::Library));
    let growth = put(&mut game, player, "Giant Growth", Zone::Hand);
    let bounce = put(&mut game, opponent, "Unsummon", Zone::Hand);

    cast(&mut game, tortoise, None);
    pass_twice(&mut game);
    assert_eq!(stats(&game, tortoise), (3, 3));
    pass_twice(&mut game);
    let choice = game.observe(player).effect_choice.unwrap();
    assert!(!choice.optional);
    assert_eq!(
        choice.candidates,
        [game.object_ref(old), game.object_ref(entry_land)]
    );
    for card in entry_spells {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    assert_eq!(game.object_view(attack_land).zone, Zone::Library);
    reject(&mut game, Action::FinishLandSelection);
    let foreign = select_land(&game, foreign);
    reject(&mut game, foreign);
    let choose = select_land(&game, entry_land);
    act(&mut game, choose);
    assert_eq!(game.object_view(entry_land).zone, Zone::Battlefield);
    assert!(game.object_view(entry_land).tapped);
    assert_eq!(game.object_view(old).zone, Zone::Graveyard);

    act(&mut game, activate_slot(land, 1));
    assert_eq!(game.mana_payment_domain().unwrap().generic, 3);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, land), (4, 4));
    assert_eq!(game.object_view(land).kinds & 3, 3);
    assert!(game
        .object_view(land)
        .keywords
        .contains(Keywords::VIGILANCE));

    let mut removed = game.clone();
    act(&mut removed, Action::Pass);
    cast(&mut removed, bounce, Some(Target::Card(tortoise)));
    settle(&mut removed);
    assert_eq!(stats(&removed, land), (3, 3));
    act(&mut removed, activate_slot(land, 1));
    assert_eq!(removed.mana_payment_domain().unwrap().generic, 4);

    let mut opposing = game.clone();
    act(&mut opposing, Action::Pass);
    act(&mut opposing, activate_slot(opposing_land, 1));
    assert_eq!(opposing.mana_payment_domain().unwrap().generic, 4);
    pay(&mut opposing);
    settle(&mut opposing);
    assert_eq!(stats(&opposing, opposing_land), (3, 3));

    let mut combined = game.clone();
    cast(&mut combined, agatha, None);
    settle(&mut combined);
    cast(&mut combined, growth, Some(Target::Card(agatha)));
    settle(&mut combined);
    let one = [0, 0, 0, 0, 0, 1];
    combined.state.players[player.index()].mana = one;
    act(&mut combined, activate_slot(land, 1));
    assert_eq!(
        combined
            .observe(player)
            .cost_choices
            .iter()
            .map(|row| row.cost.generic)
            .collect::<Vec<_>>(),
        [0, 1],
    );
    for (cost, payment) in [(0, [0; 6]), (1, one)] {
        let mut route = combined.clone();
        act(&mut route, Action::ChooseActivationCost { cost });
        assert_eq!(
            route.mana_payment_domain().unwrap().generic,
            u32::from(cost)
        );
        act(&mut route, Action::PayCosts { payment });
        settle(&mut route);
        assert_eq!(stats(&route, land), (4, 4));
        assert_eq!(
            route.state.players[player.index()].mana,
            if cost == 0 { one } else { [0; 6] }
        );
    }

    act(&mut game, activate(boots));
    target(&mut game, tortoise);
    pay(&mut game);
    settle(&mut game);
    reach_attackers(&mut game);
    act(&mut game, Action::Attack { card: tortoise });
    act(&mut game, Action::FinishAttackers);
    pass_twice(&mut game);
    assert_eq!(game.object_view(attack_land).zone, Zone::Graveyard);
    for card in attack_spells {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    let choose = select_land(&game, old);
    act(&mut game, choose);
    assert_eq!(game.object_view(old).zone, Zone::Battlefield);
    assert!(game.object_view(old).tapped);
    assert_eq!(game.object_view(attack_land).zone, Zone::Graveyard);
}

fn pay_cauldron_exile(game: &mut Game, cauldron: CardId, donor: CardId) {
    act(game, activate(cauldron));
    target(game, donor);
    pay(game);
    assert!(game.object_view(cauldron).tapped);
}

#[test]
fn agathas_soul_cauldron_grants_linked_abilities_and_funds_the_exact_recipient() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let cauldron = candidate(&mut game, player, "Agatha's Soul Cauldron");
    let recipient = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let ineligible = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foreign = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let ooze = put(&mut game, opponent, "Scavenging Ooze", Zone::Graveyard);
    let elf = put(&mut game, player, "Llanowar Elves", Zone::Graveyard);
    let noncreature = put(&mut game, player, "Opt", Zone::Graveyard);
    let exile_target = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let mountain = put(&mut game, player, "Mountain", Zone::Battlefield);
    let removal = put(&mut game, opponent, "Disenchant", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    cast(&mut game, cauldron, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);

    let mut no_reflexive = game.clone();
    pay_cauldron_exile(&mut no_reflexive, cauldron, noncreature);
    settle(&mut no_reflexive);
    assert_eq!(no_reflexive.object_view(noncreature).zone, Zone::Exile);
    assert_eq!(
        no_reflexive.counter_count(recipient, CounterKind::PlusOne),
        0
    );
    assert!(no_reflexive.observe(player).borrowed_abilities.is_empty());

    let mut mana_ability = game.clone();
    pay_cauldron_exile(&mut mana_ability, cauldron, elf);
    pass_twice(&mut mana_ability);
    assert_eq!(mana_ability.object_view(elf).zone, Zone::Exile);
    target(&mut mana_ability, recipient);
    settle(&mut mana_ability);
    let exact_elf = mana_ability.object_ref(elf);
    let rows = mana_ability.observe(player).borrowed_abilities;
    assert_eq!(rows.len(), 1);
    let row = rows[0];
    assert_eq!(row.source, mana_ability.object_ref(recipient));
    assert_eq!(row.grantor, Some(mana_ability.object_ref(cauldron)));
    assert_eq!(row.exiled, exact_elf);
    let green = [0, 0, 0, 0, 1, 0];
    act(
        &mut mana_ability,
        Action::AnnounceBorrowedMana {
            card: recipient,
            exiled: exact_elf,
            ability: row.ability,
            counter_payment: Counters::default(),
            production: green,
        },
    );
    pay(&mut mana_ability);
    assert!(mana_ability.object_view(recipient).tapped);
    assert_eq!(mana_ability.state.players[player.index()].mana, green);
    assert!(mana_ability.state.stack.is_empty());

    pay_cauldron_exile(&mut game, cauldron, ooze);
    pass_twice(&mut game);
    assert_eq!(game.object_view(ooze).zone, Zone::Exile);
    assert_eq!(game.counter_count(recipient, CounterKind::PlusOne), 0);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(foreign),
        },
    );
    target(&mut game, recipient);
    settle(&mut game);
    assert_eq!(game.counter_count(recipient, CounterKind::PlusOne), 1);
    let exact_ooze = game.object_ref(ooze);
    let borrow = |card| Action::AnnounceBorrowedAbility {
        card,
        exiled: exact_ooze,
        ability: 0,
        counter_payment: Counters::default(),
    };
    reject(&mut game, borrow(ineligible));
    act(&mut game, borrow(recipient));
    target(&mut game, exile_target);
    assert_eq!(game.state.pending, Pending::PayCosts { player });
    assert!(game.mana_payment_domain().is_none());
    act(&mut game, Action::TapMana { card: mountain });
    let cost = game.mana_payment_domain().unwrap();
    assert_eq!(cost.generic, 1);
    assert_eq!(cost.required, [0; 6]);
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 1, 0, 0],
        },
    );
    let packet = game.state.stack.last().unwrap();
    assert_eq!(
        packet.source_identity(),
        Some((
            game.object_ref(recipient),
            Card::from_name("Scavenging Ooze").unwrap()
        ))
    );

    act(&mut game, Action::Pass);
    cast(&mut game, removal, Some(Target::Card(cauldron)));
    settle(&mut game);
    assert_eq!(game.object_view(cauldron).zone, Zone::Graveyard);
    assert_eq!(game.object_view(exile_target).zone, Zone::Exile);
    assert_eq!(game.counter_count(recipient, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(ooze, CounterKind::PlusOne), 0);
    assert_eq!(game.state.players[player.index()].life, 21);
    assert!(game.observe(player).borrowed_abilities.is_empty());
    reject(&mut game, borrow(recipient));
}

fn target_player(game: &mut Game, player: Player) {
    act(
        game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
}

#[test]
fn chancellor_copies_adventures_even_after_the_original_is_countered() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let chancellor = candidate(&mut game, player, "Chancellor of Tales");
    let mouse = put(&mut game, player, "Cheeky House-Mouse", Zone::Hand);
    let wurm = put(&mut game, player, "Beanstalk Wurm", Zone::Hand);
    let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let negate = put(&mut game, opponent, "Negate", Zone::Hand);
    cast(&mut game, chancellor, None);
    settle(&mut game);
    assert_eq!(stats(&game, chancellor), (2, 3));
    assert!(game
        .object_view(chancellor)
        .keywords
        .contains(Keywords::FLYING));

    let mut front = game.clone();
    cast(&mut front, mouse, None);
    assert_eq!(front.state.stack.len(), 1);
    settle(&mut front);
    assert_eq!(stats(&front, mouse), (2, 1));

    let mut targetless = game.clone();
    act(&mut targetless, Action::AnnounceAdventure { card: wurm });
    pay(&mut targetless);
    pass_twice(&mut targetless);
    act(&mut targetless, Action::ChooseOptional { accept: true });
    settle(&mut targetless);
    let facts = &targetless.state.players[player.index()];
    assert_eq!(facts.extra_land_plays_granted, 2);
    assert_eq!(facts.instant_sorcery_cast_this_turn, 1);
    assert_eq!(targetless.object_view(wurm).zone, Zone::Exile);

    act(&mut game, Action::AnnounceAdventure { card: mouse });
    target(&mut game, first);
    pay(&mut game);
    assert_eq!(game.state.stack.len(), 2);
    let mut declined = game.clone();
    pass_twice(&mut declined);
    act(&mut declined, Action::ChooseOptional { accept: false });
    settle(&mut declined);
    assert_eq!(stats(&declined, first), (3, 3));
    assert_eq!(stats(&declined, second), (2, 2));
    assert_eq!(declined.object_view(mouse).zone, Zone::Exile);

    act(&mut game, Action::Pass);
    cast(&mut game, negate, Some(Target::Spell(mouse)));
    pass_twice(&mut game);
    assert_eq!(game.object_view(mouse).zone, Zone::Graveyard);
    pass_twice(&mut game);
    act(&mut game, Action::ChooseOptional { accept: true });
    act(
        &mut game,
        Action::SetStackTarget {
            index: 0,
            target: Target::Card(second),
        },
    );
    act(&mut game, Action::FinishStackTargets);
    assert!(game.state.stack.last().unwrap().is_copy);
    settle(&mut game);
    assert_eq!(stats(&game, first), (2, 2));
    assert_eq!(stats(&game, second), (3, 3));
    assert_eq!(
        game.state.players[player.index()].instant_sorcery_cast_this_turn,
        1
    );
    assert_eq!(game.object_view(mouse).zone, Zone::Graveyard);
    assert!(!game
        .legal_actions()
        .iter()
        .any(|action| matches!(action, Action::AnnounceExile { .. })));
}

#[test]
fn cruel_somnophage_counts_both_graveyards_and_mills_either_player() {
    let player = Player::P1;
    let opponent = player.other();
    let mut root = scenario(player);
    let creature = candidate(&mut root, player, "Cruel Somnophage // Can't Wake Up");
    let ooze = put(&mut root, player, "Scavenging Ooze", Zone::Battlefield);
    let foreign = put(&mut root, opponent, "Hill Giant", Zone::Graveyard);
    put(&mut root, player, "Bear Cub", Zone::Graveyard);
    put(&mut root, player, "Opt", Zone::Graveyard);
    let own_top = ["Forest", "Opt", "Bear Cub", "Llanowar Elves"]
        .map(|name| put(&mut root, player, name, Zone::Library));
    for card in root.state.players[opponent.index()].library.clone() {
        root.move_object(card, Zone::Outside);
    }
    let opposing_top =
        ["Forest", "Bear Cub"].map(|name| put(&mut root, opponent, name, Zone::Library));
    assert_eq!(stats(&root, creature), (2, 2));

    for (mill_player, milled, power) in [
        (player, own_top.as_slice(), 4),
        (opponent, opposing_top.as_slice(), 3),
    ] {
        let mut game = root.clone();
        act(&mut game, Action::AnnounceAdventure { card: creature });
        target_player(&mut game, mill_player);
        let cost = game.mana_payment_domain().unwrap();
        assert_eq!(cost.required, [0, 1, 0, 0, 0, 0]);
        assert_eq!(cost.generic, 1);
        pay(&mut game);
        assert_eq!(
            game.spell_characteristics().last().unwrap().name,
            "Can't Wake Up"
        );
        settle(&mut game);
        assert_eq!(game.object_view(creature).zone, Zone::Exile);
        for &card in milled {
            assert_eq!(game.object_view(card).zone, Zone::Graveyard);
        }
        assert_eq!(
            game.state.players[mill_player.index()].cards_library_to_graveyard_this_turn,
            milled.len() as u64
        );

        act(&mut game, Action::AnnounceExile { card: creature });
        let cost = game.mana_payment_domain().unwrap();
        assert_eq!(cost.required, [0, 0, 1, 0, 0, 0]);
        assert_eq!(cost.generic, 1);
        pay(&mut game);
        assert_eq!(stats(&game, creature), (power, power));
        settle(&mut game);
        assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
        assert_eq!(stats(&game, creature), (power, power));

        act(&mut game, activate(ooze));
        target(&mut game, foreign);
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(foreign).zone, Zone::Exile);
        assert_eq!(stats(&game, creature), (power - 1, power - 1));
    }
}

#[test]
fn decadent_dragon_steals_private_exact_cards_and_pays_with_attack_treasure() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let dragon = candidate(&mut game, player, "Decadent Dragon // Expensive Taste");
    let recruiter = put(&mut game, player, "Imodane's Recruiter", Zone::Hand);
    let island = put(&mut game, opponent, "Island", Zone::Battlefield);
    let bounce = put(&mut game, opponent, "Unsummon", Zone::Hand);
    let remaining = put(&mut game, opponent, "Opt", Zone::Library);
    let land = put(&mut game, opponent, "Forest", Zone::Library);
    let stolen = put(&mut game, opponent, "Shock", Zone::Library);
    act(&mut game, Action::AnnounceAdventure { card: dragon });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    target_player(&mut game, opponent);
    let cost = game.mana_payment_domain().unwrap();
    assert_eq!(cost.required, [0, 0, 1, 0, 0, 0]);
    assert_eq!(cost.generic, 2);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(dragon).zone, Zone::Exile);
    assert_eq!(game.object_view(remaining).zone, Zone::Library);
    for (card, name) in [(land, "Forest"), (stolen, "Shock")] {
        assert!(game.observe(player).objects.iter().any(|object| {
            object.id == card && object.zone == Zone::Exile && object.name == name
        }));
        assert!(!game
            .observe(opponent)
            .objects
            .iter()
            .any(|object| object.id == card));
    }

    let mut wrong_color = game.clone();
    wrong_color.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
    reject(&mut wrong_color, Action::AnnounceExile { card: stolen });

    let mut later = game.clone();
    finish_turn(&mut later);
    act(&mut later, Action::Pass);
    reject(&mut later, Action::PlayLand { card: land });
    finish_turn(&mut later);
    pass_twice(&mut later);
    pass_twice(&mut later);
    act(&mut later, Action::PlayLand { card: land });
    let played = later.object_view(land);
    assert_eq!(played.zone, Zone::Battlefield);
    assert_eq!(played.controller, player);
    assert_eq!(played.owner, opponent);

    act(&mut game, Action::AnnounceExile { card: dragon });
    let cost = game.mana_payment_domain().unwrap();
    assert_eq!(cost.required, [0, 0, 0, 2, 0, 0]);
    assert_eq!(cost.generic, 2);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, dragon), (4, 4));
    assert!(game
        .object_view(dragon)
        .keywords
        .contains(Keywords::FLYING.union(Keywords::TRAMPLE)));
    cast(&mut game, recruiter, None);
    settle(&mut game);
    reach_attackers(&mut game);
    act(&mut game, Action::Attack { card: dragon });
    act(&mut game, Action::FinishAttackers);
    act(&mut game, Action::Pass);
    act(&mut game, Action::TapMana { card: island });
    cast(&mut game, bounce, Some(Target::Card(dragon)));
    settle(&mut game);
    assert_eq!(game.object_view(dragon).zone, Zone::Hand);
    reject(&mut game, Action::PlayLand { card: land });
    let treasures = game
        .observe(player)
        .objects
        .into_iter()
        .filter(|object| {
            object.zone == Zone::Battlefield && object.is_token() && object.has_subtype("Treasure")
        })
        .collect::<Vec<_>>();
    assert_eq!(treasures.len(), 1);
    assert_eq!(treasures[0].controller, player);
    act(&mut game, Action::AnnounceExile { card: stolen });
    target_player(&mut game, opponent);
    let red = [0, 0, 0, 1, 0, 0];
    act(
        &mut game,
        Action::ActivateMana {
            card: treasures[0].id,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: red,
        },
    );
    assert_eq!(game.mana_payment_domain().unwrap().required, red);
    act(&mut game, Action::PayCosts { payment: red });
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 18);
    assert_eq!(game.object_view(stolen).zone, Zone::Graveyard);
    assert_eq!(game.object_view(stolen).owner, opponent);
}

#[test]
fn devouring_sugarmaw_creates_both_tokens_and_checks_its_own_upkeep() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let horror = candidate(&mut game, player, "Devouring Sugarmaw // Have for Dinner");
    let bow = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let season = put(&mut game, player, "Doubling Season", Zone::Battlefield);
    let foreign = put(&mut game, opponent, "Short Bow", Zone::Battlefield);

    act(&mut game, Action::AnnounceAdventure { card: horror });
    let cost = game.mana_payment_domain().unwrap();
    assert_eq!(cost.required, [1, 0, 0, 0, 0, 0]);
    assert_eq!(cost.generic, 1);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(horror).zone, Zone::Exile);
    let tokens = game
        .observe(player)
        .objects
        .into_iter()
        .filter(|object| {
            object.zone == Zone::Battlefield && object.is_token() && object.controller == player
        })
        .collect::<Vec<_>>();
    let humans = tokens
        .iter()
        .filter(|token| token.has_subtype("Human"))
        .collect::<Vec<_>>();
    let food = tokens
        .iter()
        .filter(|token| token.has_subtype("Food"))
        .count();
    assert_eq!(humans.len(), 2);
    assert_eq!(food, 2);
    assert_eq!(tokens.len(), 4);
    assert!(humans
        .iter()
        .all(|human| (human.power, human.toughness, human.colors) == (1, 1, 1)));
    let human = humans[0].reference;

    act(&mut game, Action::AnnounceExile { card: horror });
    let cost = game.mana_payment_domain().unwrap();
    assert_eq!(cost.required, [0, 0, 2, 0, 0, 0]);
    assert_eq!(cost.generic, 2);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, horror), (6, 6));
    assert!(game
        .object_view(horror)
        .keywords
        .contains(Keywords::MENACE.union(Keywords::TRAMPLE)));

    finish_turn(&mut game);
    assert_eq!(game.state.step, Step::Upkeep);
    assert_eq!(game.state.active, opponent);
    assert!(game.state.stack.is_empty());
    finish_turn(&mut game);
    assert_eq!(game.state.step, Step::Upkeep);
    assert_eq!(game.state.active, player);
    pass_twice(&mut game);
    for card in [bear, foreign] {
        let object = game.object_ref(card);
        reject(&mut game, Action::ChooseAdditionalCost { object });
    }
    for object in [game.object_ref(bow), game.object_ref(season), human] {
        let mut paid = game.clone();
        act(&mut paid, Action::ChooseAdditionalCost { object });
        act(&mut paid, Action::PayResolutionMana { payment: [0; 6] });
        assert!(!paid.object_view(horror).tapped);
        assert!(!paid
            .observe(player)
            .objects
            .iter()
            .any(|view| view.reference == object));
    }
    act(&mut game, Action::DeclineResolutionMana);
    assert!(game.object_view(horror).tapped);
    assert_eq!(game.object_view(bow).zone, Zone::Battlefield);
}
