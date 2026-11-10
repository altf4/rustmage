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
    assert!(
        unpaid
            .object_view(champion)
            .keywords
            .contains(Keywords::TRAMPLE)
    );
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
    assert!(
        unpaid
            .observe(opponent)
            .known_hand
            .contains(&unpaid.object_ref(found))
    );
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
    assert!(
        game.observe(opponent)
            .known_hand
            .contains(&game.object_ref(found))
    );

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
