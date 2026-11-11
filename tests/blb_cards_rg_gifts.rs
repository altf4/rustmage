//! Card-specific Gift outcomes, costs, independent triggers, and partial targets.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn announce_gift(game: &mut Game, player: Player, spell: CardId, promised: bool) {
    act(game, Action::AnnounceCast { card: spell });
    act(
        game,
        Action::ChooseGift {
            beneficiary: promised.then_some(player.other()),
        },
    );
}

fn gift_targets(game: &mut Game, targets: &[CardId]) {
    for &card in targets {
        act(game, target(card));
    }
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
    pay(game);
}

fn drawn(game: &Game, player: Player) -> u64 {
    game.state.players[player.index()].cards_drawn_this_turn
}

fn tokens_with_type(game: &Game, player: Player, subtype: &str) -> Vec<CardId> {
    game.controlled_token_ids(player)
        .filter(|&id| game.has_subtype(id, subtype))
        .collect()
}

fn drain(game: &mut Game, chosen: Option<CardId>) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::PutTrigger { .. } | Action::Pass)
                || chosen.is_some_and(|id| *action == target(id))
        });
    }
    panic!("Gift triggers did not settle");
}

#[test]
fn blooming_blast_can_damage_its_caster_and_howl_hits_its_target_and_other_creatures() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let victim = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let blast = put(&mut game, player, "Blooming Blast", Zone::Hand);
    announce_gift(&mut game, player, blast, true);
    gift_targets(&mut game, &[victim]);

    let mut stale = game.clone();
    stale.move_object(victim, Zone::Exile);
    settle(&mut stale);
    assert_eq!(stale.state.players[player.index()].life, 20);
    assert!(tokens_with_type(&stale, opponent, "Treasure").is_empty());

    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 17);
    assert_eq!(game.state.players[opponent.index()].life, 20);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(tokens_with_type(&game, opponent, "Treasure").len(), 1);

    let mut howling = scenario(player);
    let giant = put(&mut howling, opponent, "Hill Giant", Zone::Battlefield);
    let collateral = put(&mut howling, player, "Grizzly Bears", Zone::Battlefield);
    let howl = put(&mut howling, player, "Wildfire Howl", Zone::Hand);
    let mut declined = howling.clone();
    announce_gift(&mut declined, player, howl, false);
    gift_targets(&mut declined, &[]);
    settle(&mut declined);
    assert_eq!(drawn(&declined, opponent), 0);
    assert_eq!(declined.object_view(giant).zone, Zone::Battlefield);
    assert_eq!(declined.object_view(collateral).zone, Zone::Graveyard);

    announce_gift(&mut howling, player, howl, true);
    gift_targets(&mut howling, &[giant]);
    settle(&mut howling);
    assert_eq!(drawn(&howling, opponent), 1);
    assert_eq!(howling.object_view(giant).zone, Zone::Graveyard);
    assert_eq!(howling.object_view(collateral).zone, Zone::Graveyard);
}

#[test]
fn sazacaps_brew_requires_discard_and_can_still_draw_after_the_creature_departs() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let brew = put(&mut game, player, "Sazacap's Brew", Zone::Hand);
    reject(&mut game, Action::AnnounceCast { card: brew });
    let discarded = put(&mut game, player, "Forest", Zone::Hand);
    announce_gift(&mut game, player, brew, true);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    act(&mut game, target(creature));
    let discard = Action::ChooseAdditionalCost {
        object: game.object_ref(discarded),
    };
    for _ in 0..8 {
        if game.state.pending == Pending::Priority && game.object_view(brew).zone == Zone::Stack {
            break;
        }
        choose(&mut game, |action| {
            *action == discard
                || matches!(
                    action,
                    Action::FinishTargets | Action::PayCosts { .. } | Action::ChooseCastCost { .. }
                )
        });
    }
    assert_eq!(game.object_view(brew).zone, Zone::Stack);
    assert_eq!(game.object_view(discarded).zone, Zone::Graveyard);
    assert_eq!(drawn(&game, player), 0);

    let mut only_player = game.clone();
    only_player.move_object(creature, Zone::Exile);
    settle(&mut only_player);
    assert_eq!(drawn(&only_player, player), 2);
    assert_eq!(tokens_with_type(&only_player, opponent, "Fish").len(), 1);

    settle(&mut game);
    assert_eq!(drawn(&game, player), 2);
    assert_eq!(drawn(&game, opponent), 0);
    assert_eq!(stats(&game, creature), (4, 2));
    let [fish] = tokens_with_type(&game, opponent, "Fish")
        .try_into()
        .unwrap();
    assert!(game.object_view(fish).tapped);
}

#[test]
fn valley_rally_buffs_only_current_creatures_and_longstalk_counters_before_fighting() {
    let player = Player::P0;
    let opponent = player.other();
    let mut rally = scenario(player);
    let chosen = put(&mut rally, player, "Grizzly Bears", Zone::Battlefield);
    let teammate = put(&mut rally, player, "Grizzly Bears", Zone::Battlefield);
    let rival = put(&mut rally, opponent, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut rally, player, "Valley Rally", Zone::Hand);
    announce_gift(&mut rally, player, spell, true);
    gift_targets(&mut rally, &[chosen]);
    let mut stale = rally.clone();
    stale.move_object(chosen, Zone::Exile);
    settle(&mut stale);
    assert_eq!(stats(&stale, teammate), (2, 2));
    assert!(tokens_with_type(&stale, opponent, "Food").is_empty());

    settle(&mut rally);
    let late = put(&mut rally, player, "Grizzly Bears", Zone::Battlefield);
    assert_eq!(stats(&rally, chosen), (4, 2));
    assert_eq!(stats(&rally, teammate), (4, 2));
    assert_eq!(stats(&rally, rival), (2, 2));
    assert_eq!(stats(&rally, late), (2, 2));
    assert!(rally.keywords(chosen).contains(Keywords::FIRST_STRIKE));
    assert!(!rally.keywords(teammate).contains(Keywords::FIRST_STRIKE));
    assert_eq!(tokens_with_type(&rally, opponent, "Food").len(), 1);

    let mut fight = scenario(player);
    let ours = put(&mut fight, player, "Hill Giant", Zone::Battlefield);
    let theirs = put(&mut fight, opponent, "Hill Giant", Zone::Battlefield);
    let brawl = put(&mut fight, player, "Longstalk Brawl", Zone::Hand);
    announce_gift(&mut fight, player, brawl, true);
    gift_targets(&mut fight, &[ours, theirs]);

    let mut only_ours = fight.clone();
    only_ours.move_object(theirs, Zone::Exile);
    settle(&mut only_ours);
    assert_eq!(stats(&only_ours, ours), (4, 4));
    assert_eq!(tokens_with_type(&only_ours, opponent, "Fish").len(), 1);

    settle(&mut fight);
    assert_eq!(fight.object_view(theirs).zone, Zone::Graveyard);
    assert_eq!(fight.object_view(ours).zone, Zone::Battlefield);
    assert_eq!(stats(&fight, ours), (4, 4));
    let [fish] = tokens_with_type(&fight, opponent, "Fish")
        .try_into()
        .unwrap();
    assert!(fight.object_view(fish).tapped);
}

#[test]
fn recycling_and_wear_down_keep_their_one_remaining_legal_target() {
    let player = Player::P0;
    let opponent = player.other();
    let mut recycle = scenario(player);
    let land = put(&mut recycle, player, "Forest", Zone::Graveyard);
    let creature = put(&mut recycle, player, "Grizzly Bears", Zone::Graveyard);
    let nonpermanent = put(&mut recycle, player, "Shock", Zone::Graveyard);
    let foreign = put(&mut recycle, opponent, "Forest", Zone::Graveyard);
    let spell = put(&mut recycle, player, "Peerless Recycling", Zone::Hand);
    announce_gift(&mut recycle, player, spell, true);
    reject(&mut recycle, target(nonpermanent));
    reject(&mut recycle, target(foreign));
    act(&mut recycle, target(land));
    reject(&mut recycle, target(land));
    gift_targets(&mut recycle, &[creature]);
    let mut both = recycle.clone();
    settle(&mut both);
    assert_eq!(drawn(&both, opponent), 1);
    assert_eq!(both.object_view(land).zone, Zone::Hand);
    assert_eq!(both.object_view(creature).zone, Zone::Hand);

    recycle.move_object(creature, Zone::Exile);
    settle(&mut recycle);
    assert_eq!(drawn(&recycle, opponent), 1);
    assert_eq!(recycle.object_view(land).zone, Zone::Hand);
    assert_eq!(recycle.object_view(creature).zone, Zone::Exile);

    let mut wear = scenario(player);
    let artifact = put(&mut wear, player, "Candy Trail", Zone::Battlefield);
    let enchantment = put(&mut wear, opponent, "Innkeeper's Talent", Zone::Battlefield);
    let spell = put(&mut wear, player, "Wear Down", Zone::Hand);
    let mut declined = wear.clone();
    announce_gift(&mut declined, player, spell, false);
    gift_targets(&mut declined, &[artifact]);
    settle(&mut declined);
    assert_eq!(drawn(&declined, opponent), 0);
    assert_eq!(declined.object_view(artifact).zone, Zone::Graveyard);
    assert_eq!(declined.object_view(enchantment).zone, Zone::Battlefield);

    announce_gift(&mut wear, player, spell, true);
    gift_targets(&mut wear, &[artifact, enchantment]);
    wear.move_object(enchantment, Zone::Exile);
    settle(&mut wear);
    assert_eq!(drawn(&wear, opponent), 1);
    assert_eq!(wear.object_view(artifact).zone, Zone::Graveyard);
    assert_eq!(wear.object_view(enchantment).zone, Zone::Exile);
}

#[test]
fn scrapshooter_gives_its_card_with_or_without_a_legal_destruction_target() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let own = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let opposing = put(&mut game, opponent, "Candy Trail", Zone::Battlefield);
    let shooter = put(&mut game, player, "Scrapshooter", Zone::Hand);

    let mut declined = game.clone();
    announce_gift(&mut declined, player, shooter, false);
    gift_targets(&mut declined, &[]);
    drain(&mut declined, None);
    assert_eq!(drawn(&declined, opponent), 0);
    assert_eq!(declined.object_view(opposing).zone, Zone::Battlefield);

    let mut untargeted = game.clone();
    untargeted.move_object(opposing, Zone::Exile);
    announce_gift(&mut untargeted, player, shooter, true);
    gift_targets(&mut untargeted, &[]);
    drain(&mut untargeted, None);
    assert_eq!(drawn(&untargeted, opponent), 1);
    assert_eq!(untargeted.object_view(own).zone, Zone::Battlefield);

    announce_gift(&mut game, player, shooter, true);
    gift_targets(&mut game, &[]);
    drain(&mut game, Some(opposing));
    assert_eq!(drawn(&game, opponent), 1);
    assert_eq!(game.object_view(opposing).zone, Zone::Graveyard);
    assert_eq!(game.object_view(own).zone, Zone::Battlefield);
    assert!(game.keywords(shooter).contains(Keywords::REACH));
}
