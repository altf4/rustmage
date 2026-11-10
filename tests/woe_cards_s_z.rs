mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn candidate(game: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, owner, Card::from_name(name).unwrap(), zone)
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn begin_attackers(game: &mut Game) {
    pass_pair(game);
    assert_eq!(game.state.step, Step::BeginCombat);
    settle(game);
    pass_pair(game);
    assert_eq!(game.state.pending, Pending::Attackers);
}

fn declare_blocks(game: &mut Game, attackers: &[CardId], blocks: &[(CardId, CardId)]) {
    begin_attackers(game);
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
    pass_pair(game);
    assert_eq!(game.state.pending, Pending::Blockers);
    for &(blocker, attacker) in blocks {
        act(game, Action::Block { blocker, attacker });
    }
    act(game, Action::FinishBlockers);
    for _ in 0..blocks.len() {
        if !matches!(game.state.pending, Pending::OrderTriggers { .. }) {
            return;
        }
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
}

#[test]
fn skewer_slinger_hits_each_creature_it_blocks_or_is_blocked_by() {
    for player in [Player::P0, Player::P1] {
        let mut attacking = scenario(player);
        let slinger = candidate(&mut attacking, player, "Skewer Slinger", Zone::Battlefield);
        let first = put(
            &mut attacking,
            player.other(),
            "Bear Cub",
            Zone::Battlefield,
        );
        let second = put(
            &mut attacking,
            player.other(),
            "Bear Cub",
            Zone::Battlefield,
        );
        assert_eq!(stats(&attacking, slinger), (1, 3));
        assert!(attacking
            .object_view(slinger)
            .keywords
            .contains(Keywords::REACH));
        declare_blocks(
            &mut attacking,
            &[slinger],
            &[(first, slinger), (second, slinger)],
        );
        assert_eq!(attacking.state.stack.len(), 2);
        settle(&mut attacking);
        assert_eq!(attacking.object_view(first).damage, 1);
        assert_eq!(attacking.object_view(second).damage, 1);
        assert_eq!(attacking.object_view(slinger).damage, 0);

        let mut defending = scenario(player.other());
        let flyer = put(
            &mut defending,
            player.other(),
            "Sleep-Cursed Faerie",
            Zone::Battlefield,
        );
        let slinger = candidate(&mut defending, player, "Skewer Slinger", Zone::Battlefield);
        declare_blocks(&mut defending, &[flyer], &[(slinger, flyer)]);
        settle(&mut defending);
        assert_eq!(defending.object_view(flyer).damage, 1);
    }
}

#[test]
fn tattered_ratter_boosts_the_blocked_rat_once_per_declaration() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ratter = candidate(&mut game, player, "Tattered Ratter", Zone::Battlefield);
    let blocked = put(&mut game, player, "Voracious Vermin", Zone::Battlefield);
    let unblocked = put(&mut game, player, "Voracious Vermin", Zone::Battlefield);
    let first = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    assert_eq!(stats(&game, ratter), (2, 2));
    declare_blocks(
        &mut game,
        &[blocked, unblocked],
        &[(first, blocked), (second, blocked)],
    );
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert_eq!(stats(&game, blocked), (4, 1));
    assert_eq!(stats(&game, unblocked), (2, 1));
    assert_eq!(stats(&game, ratter), (2, 2));
}

#[test]
fn territorial_witchstalker_rechecks_its_combat_guard_and_keeps_defender() {
    let player = Player::P1;
    let mut parent = scenario(player);
    let wolf = candidate(
        &mut parent,
        player,
        "Territorial Witchstalker",
        Zone::Battlefield,
    );
    put(
        &mut parent,
        player.other(),
        "Hamlet Glutton",
        Zone::Battlefield,
    );

    let mut unqualified = parent.clone();
    begin_attackers(&mut unqualified);
    assert_eq!(stats(&unqualified, wolf), (2, 3));
    reject(&mut unqualified, Action::Attack { card: wolf });

    let creature = put(&mut parent, player, "Hamlet Glutton", Zone::Battlefield);
    let mut qualified = parent.clone();
    begin_attackers(&mut qualified);
    assert_eq!(stats(&qualified, wolf), (3, 3));
    assert!(qualified
        .object_view(wolf)
        .keywords
        .contains(Keywords::DEFENDER));
    assert!(qualified
        .legal_actions()
        .contains(&Action::Attack { card: wolf }));
    act(&mut qualified, Action::FinishAttackers);
    for _ in 0..16 {
        if qualified.state.active != player {
            break;
        }
        match qualified.state.pending {
            Pending::Blockers => act(&mut qualified, Action::FinishBlockers),
            _ => act(&mut qualified, Action::Pass),
        }
    }
    assert_eq!(qualified.state.active, player.other());
    assert_eq!(stats(&qualified, wolf), (2, 3));

    let murder = put(&mut parent, player.other(), "Murder", Zone::Hand);
    let swamps = [0; 3].map(|_| put(&mut parent, player.other(), "Swamp", Zone::Battlefield));
    pass_pair(&mut parent);
    assert_eq!(parent.state.stack.len(), 1);
    act(&mut parent, Action::Pass);
    for card in swamps {
        act(&mut parent, Action::TapMana { card });
    }
    cast(&mut parent, murder, Some(Target::Card(creature)));
    settle(&mut parent);
    assert_eq!(parent.object_view(creature).zone, Zone::Graveyard);
    assert_eq!(stats(&parent, wolf), (2, 3));
    pass_pair(&mut parent);
    reject(&mut parent, Action::Attack { card: wolf });
}

#[test]
fn stonesplitter_bolt_deals_x_or_twice_x_with_paid_bargain() {
    let player = Player::P0;
    for (x, bargain, planeswalker) in [(0, false, false), (2, false, false), (2, true, true)] {
        let mut game = scenario(player);
        let spell = candidate(&mut game, player, "Stonesplitter Bolt", Zone::Hand);
        let payment = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let target_card = if planeswalker {
            let walker = put(
                &mut game,
                player.other(),
                "Elspeth, Storm Slayer",
                Zone::Battlefield,
            );
            game.add_counter(walker, CounterKind::Loyalty, 9);
            walker
        } else {
            put(
                &mut game,
                player.other(),
                "Gigantosaurus",
                Zone::Battlefield,
            )
        };
        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::ChooseX { value: x });
        let sacrifice = bargain.then(|| game.object_ref(payment));
        act(&mut game, Action::ChooseBargain { sacrifice });
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(player.other()),
            },
        );
        act(&mut game, target(target_card));
        let mana_before: u32 = game.state.players[player.index()].mana.iter().sum();
        pay(&mut game);
        let mana_after: u32 = game.state.players[player.index()].mana.iter().sum();
        assert_eq!(mana_before - mana_after, x + 1);
        assert_eq!(
            game.object_view(payment).zone,
            if bargain {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        settle(&mut game);
        if planeswalker {
            assert_eq!(game.counter_count(target_card, CounterKind::Loyalty), 5);
        } else {
            assert_eq!(game.object_view(target_card).damage, i128::from(x));
        }
    }
}

#[test]
fn storyteller_pixie_draws_for_a_committed_adventure_but_not_its_copy_or_other_faces() {
    let player = Player::P0;
    let mut game = scenario(player);
    let pixie = candidate(&mut game, player, "Storyteller Pixie", Zone::Hand);
    cast(&mut game, pixie, None);
    settle(&mut game);
    assert_eq!(stats(&game, pixie), (3, 3));
    assert!(game.object_view(pixie).keywords.contains(Keywords::FLYING));

    let imitator = put(&mut game, player, "Slick Imitator", Zone::Battlefield);
    game.state.players[player.index()].speed = 4;
    let first = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    game.state.objects[first as usize].tapped = true;
    game.state.objects[second as usize].tapped = true;
    let threadbind = put(&mut game, player, "Threadbind Clique", Zone::Hand);
    act(&mut game, Action::AnnounceAdventure { card: threadbind });
    act(&mut game, target(first));
    pay(&mut game);
    let source = game.object_ref(imitator);
    act(
        &mut game,
        Action::AnnounceSourceGrant { source, ability: 0 },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Spell(threadbind),
        },
    );
    pay(&mut game);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::SetStackTarget {
            index: 0,
            target: Target::Card(second),
        },
    );
    act(&mut game, Action::FinishStackTargets);
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Graveyard);
    assert_eq!(game.object_view(second).zone, Zone::Graveyard);
    assert_eq!(game.object_view(threadbind).zone, Zone::Exile);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);

    act(&mut game, Action::AnnounceExile { card: threadbind });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(threadbind).zone, Zone::Battlefield);
    let omen = put(&mut game, player, "Riling Dawnbreaker", Zone::Hand);
    act(&mut game, Action::AnnounceOmen { card: omen });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn succumb_to_the_cold_stuns_each_surviving_target_including_a_tapped_creature() {
    for removed in [false, true] {
        let player = Player::P1;
        let mut game = scenario(player);
        let spell = candidate(&mut game, player, "Succumb to the Cold", Zone::Hand);
        let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let first = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let second = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let third = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        game.state.objects[first as usize].tapped = true;
        act(&mut game, Action::AnnounceCast { card: spell });
        reject(&mut game, Action::FinishTargets);
        reject(&mut game, target(own));
        act(&mut game, target(first));
        reject(&mut game, target(first));
        let mut single = game.clone();
        act(&mut single, Action::FinishTargets);
        pay(&mut single);
        settle(&mut single);
        assert_eq!(single.counter_count(first, CounterKind::Stun), 1);
        assert_eq!(single.counter_count(second, CounterKind::Stun), 0);

        act(&mut game, target(second));
        reject(&mut game, target(third));
        pay(&mut game);
        if removed {
            act(&mut game, Action::Pass);
            let bounce = put(&mut game, player.other(), "Unsummon", Zone::Hand);
            cast(&mut game, bounce, Some(Target::Card(second)));
        }
        settle(&mut game);
        assert!(game.object_view(first).tapped);
        assert_eq!(game.counter_count(first, CounterKind::Stun), 1);
        assert_eq!(
            game.counter_count(second, CounterKind::Stun),
            u64::from(!removed)
        );
        if removed {
            assert_eq!(game.object_view(second).zone, Zone::Hand);
        } else {
            assert!(game.object_view(second).tapped);
        }
        assert!(!game.object_view(third).tapped);
    }
}
