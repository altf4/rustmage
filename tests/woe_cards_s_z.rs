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

fn resolve_with_target(game: &mut Game, card: CardId) {
    for _ in 0..24 {
        match game.state.pending {
            Pending::Targets { .. } => act(game, target(card)),
            Pending::OrderTriggers { .. } => {
                choose(game, |action| matches!(action, Action::PutTrigger { .. }));
            }
            Pending::Priority if game.state.stack.is_empty() => return,
            Pending::Priority => act(game, Action::Pass),
            _ => panic!("unexpected choice {:?}", game.state.pending),
        }
    }
    panic!("tap trigger resolution budget exhausted");
}

#[test]
fn sharae_and_solitary_sanctuary_count_actual_taps_in_different_ways() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        game.state.players[player.index()].mana = [16; 6];
        let sharae = candidate(&mut game, player, "Sharae of Numbing Depths", Zone::Hand);
        let sanctuary = candidate(&mut game, player, "Solitary Sanctuary", Zone::Hand);
        let already = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let first = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let second = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let later = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        game.state.objects[already as usize].tapped = true;
        cast(&mut game, sharae, None);
        resolve_with_target(&mut game, already);
        assert_eq!(stats(&game, sharae), (2, 3));
        assert_eq!(game.counter_count(already, CounterKind::Stun), 1);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
        cast(&mut game, sanctuary, None);
        resolve_with_target(&mut game, already);
        assert_eq!(game.counter_count(already, CounterKind::Stun), 2);
        assert_eq!(game.counter_count(sharae, CounterKind::PlusOne), 0);

        let spell = put(&mut game, player, "Succumb to the Cold", Zone::Hand);
        announce(
            &mut game,
            spell,
            &[Target::Card(first), Target::Card(second)],
        );
        resolve_with_target(&mut game, sharae);
        assert!(game.object_view(first).tapped && game.object_view(second).tapped);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
        assert_eq!(game.counter_count(sharae, CounterKind::PlusOne), 2);

        let repeat = put(&mut game, player, "Succumb to the Cold", Zone::Hand);
        announce(
            &mut game,
            repeat,
            &[Target::Card(later), Target::Card(first)],
        );
        resolve_with_target(&mut game, sharae);
        assert!(game.object_view(later).tapped);
        assert_eq!(game.counter_count(first, CounterKind::Stun), 2);
        assert_eq!(game.counter_count(sharae, CounterKind::PlusOne), 3);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    }
}

#[test]
fn snaremaster_sprite_pays_before_choosing_the_reflexive_target() {
    let player = Player::P1;
    let mut game = scenario(player);
    let sprite = candidate(&mut game, player, "Snaremaster Sprite", Zone::Hand);
    let bearer = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    cast(&mut game, sprite, None);
    pass_pair(&mut game);
    assert_eq!(stats(&game, sprite), (1, 1));
    assert!(game.object_view(sprite).keywords.contains(Keywords::FLYING));
    pass_pair(&mut game);
    reject(&mut game, target(bearer));
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    settle(&mut declined);
    assert!(!declined.object_view(bearer).tapped);
    assert_eq!(declined.counter_count(bearer, CounterKind::Stun), 0);

    let mana_before: u32 = game.state.players[player.index()].mana.iter().sum();
    choose(&mut game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    let mana_after: u32 = game.state.players[player.index()].mana.iter().sum();
    assert_eq!(mana_before - mana_after, 2);
    assert!(!game.object_view(bearer).tapped);
    reject(&mut game, target(sprite));
    act(&mut game, target(bearer));
    let mut response = game.clone();
    act(&mut response, Action::Pass);
    let bounce = put(&mut response, player.other(), "Unsummon", Zone::Hand);
    cast(&mut response, bounce, Some(Target::Card(bearer)));
    settle(&mut response);
    assert_eq!(response.object_view(bearer).zone, Zone::Hand);
    assert_eq!(response.counter_count(bearer, CounterKind::Stun), 0);

    settle(&mut game);
    assert!(game.object_view(bearer).tapped);
    assert_eq!(game.counter_count(bearer, CounterKind::Stun), 1);
}

fn complete_scry_one(game: &mut Game, player: Player) {
    let top = game.top_library_card(player).unwrap();
    pass_pair(game);
    assert_eq!(game.state.pending, Pending::EffectLibrary { player });
    assert!(game.observe(player).library_choice.is_some());
    assert!(game.observe(player.other()).library_choice.is_none());
    assert_eq!(
        game.legal_actions()
            .iter()
            .filter(|action| matches!(action, Action::ArrangeLibrary { .. }))
            .count(),
        2
    );
    act(
        game,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Bottom,
        },
    );
    assert_ne!(game.top_library_card(player), Some(top));
    settle(game);
}

#[test]
fn slumbering_keepguard_scries_for_enchantments_and_locks_its_paid_boost() {
    let player = Player::P0;
    let mut game = scenario(player);
    let keepguard = candidate(&mut game, player, "Slumbering Keepguard", Zone::Hand);
    put(
        &mut game,
        player.other(),
        "A Tale for the Ages",
        Zone::Battlefield,
    );
    let recipient = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    cast(&mut game, keepguard, None);
    settle(&mut game);
    assert_eq!(stats(&game, keepguard), (1, 1));
    let own = put(&mut game, player, "A Tale for the Ages", Zone::Hand);
    cast(&mut game, own, None);
    pass_pair(&mut game);
    complete_scry_one(&mut game, player);

    choose(
        &mut game,
        |action| matches!(action, Action::Activate { card, ability: 0, target: None, .. } if *card == keepguard),
    );
    let mut interrupted = game.clone();
    act(&mut interrupted, Action::Pass);
    let disenchant = put(&mut interrupted, player.other(), "Disenchant", Zone::Hand);
    cast(&mut interrupted, disenchant, Some(Target::Card(own)));
    settle(&mut interrupted);
    assert_eq!(stats(&interrupted, keepguard), (1, 1));

    settle(&mut game);
    assert_eq!(stats(&game, keepguard), (2, 2));
    let role = put(&mut game, player, "Royal Treatment", Zone::Hand);
    cast(&mut game, role, Some(Target::Card(recipient)));
    pass_pair(&mut game);
    complete_scry_one(&mut game, player);
    assert_eq!(stats(&game, keepguard), (2, 2));
}

#[test]
fn spider_food_creates_food_with_zero_or_one_legal_target() {
    let player = Player::P0;
    for (name, stale) in [
        (None, false),
        (Some("Short Bow"), false),
        (Some("A Tale for the Ages"), false),
        (Some("Barrow Naughty"), false),
        (Some("Barrow Naughty"), true),
    ] {
        let mut game = scenario(player);
        let spell = candidate(&mut game, player, "Spider Food", Zone::Hand);
        let ground = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let chosen = name.map(|name| put(&mut game, player.other(), name, Zone::Battlefield));
        let spare = name
            .is_none()
            .then(|| put(&mut game, player.other(), "Short Bow", Zone::Battlefield));
        act(&mut game, Action::AnnounceCast { card: spell });
        reject(&mut game, target(ground));
        if let Some(card) = chosen {
            act(&mut game, target(card));
        } else {
            act(&mut game, Action::FinishTargets);
        }
        pay(&mut game);
        if stale {
            act(&mut game, Action::Pass);
            let bounce = put(&mut game, player.other(), "Unsummon", Zone::Hand);
            cast(&mut game, bounce, chosen.map(Target::Card));
        }
        settle(&mut game);
        if let Some(card) = chosen {
            assert_eq!(
                game.object_view(card).zone,
                if stale { Zone::Hand } else { Zone::Graveyard }
            );
        }
        if let Some(card) = spare {
            assert_eq!(game.object_view(card).zone, Zone::Battlefield);
        }
        let food = game
            .observe(player)
            .objects
            .into_iter()
            .filter(|object| {
                object.zone == Zone::Battlefield
                    && object.controller == player
                    && object.is_token()
                    && object.has_subtype("Food")
            })
            .count();
        assert_eq!(food, usize::from(!stale));
    }
}

#[test]
fn spell_stutter_charges_the_spells_controller_for_live_faeries() {
    let player = Player::P1;
    let payer = player.other();
    let mut game = scenario(payer);
    let first = put(&mut game, player, "Barrow Naughty", Zone::Battlefield);
    put(&mut game, player, "Sleep-Cursed Faerie", Zone::Battlefield);
    put(&mut game, payer, "Barrow Naughty", Zone::Battlefield);
    let shock = put(&mut game, payer, "Shock", Zone::Hand);
    let spell = candidate(&mut game, player, "Spell Stutter", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(player)));
    act(&mut game, Action::Pass);
    cast(&mut game, spell, Some(Target::Spell(shock)));

    let mut reduced = game.clone();
    act(&mut reduced, Action::Pass);
    let bounce = put(&mut reduced, payer, "Unsummon", Zone::Hand);
    cast(&mut reduced, bounce, Some(Target::Card(first)));
    pass_pair(&mut reduced);
    assert_eq!(reduced.object_view(first).zone, Zone::Hand);
    pass_pair(&mut reduced);
    assert_eq!(reduced.actor(), Some(payer));
    act(
        &mut reduced,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 3],
        },
    );
    settle(&mut reduced);
    assert_eq!(reduced.state.players[player.index()].life, 18);

    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(payer));
    reject(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 3],
        },
    );
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    settle(&mut declined);
    assert_eq!(declined.object_view(shock).zone, Zone::Graveyard);
    assert_eq!(declined.state.players[player.index()].life, 20);
    act(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 4],
        },
    );
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 18);
}

#[test]
fn tangled_colony_cannot_block_and_creates_rats_equal_to_actual_damage() {
    let player = Player::P0;
    let opponent = player.other();
    let mut defense = scenario(opponent);
    let attacker = put(&mut defense, opponent, "Bear Cub", Zone::Battlefield);
    let colony = candidate(&mut defense, player, "Tangled Colony", Zone::Battlefield);
    let blocker = put(&mut defense, player, "Bear Cub", Zone::Battlefield);
    begin_attackers(&mut defense);
    act(&mut defense, Action::Attack { card: attacker });
    act(&mut defense, Action::FinishAttackers);
    pass_pair(&mut defense);
    reject(
        &mut defense,
        Action::Block {
            blocker: colony,
            attacker,
        },
    );
    act(&mut defense, Action::Block { blocker, attacker });

    let mut game = scenario(player);
    let colony = candidate(&mut game, player, "Tangled Colony", Zone::Hand);
    cast(&mut game, colony, None);
    settle(&mut game);
    assert_eq!(stats(&game, colony), (3, 2));
    let rats = |game: &Game| {
        game.observe(player)
            .objects
            .into_iter()
            .filter(|object| {
                object.zone == Zone::Battlefield
                    && object.controller == player
                    && object.is_token()
                    && object.has_subtype("Rat")
            })
            .collect::<Vec<_>>()
    };
    let mut no_damage = game.clone();
    let murder = put(&mut no_damage, player, "Murder", Zone::Hand);
    cast(&mut no_damage, murder, Some(Target::Card(colony)));
    settle(&mut no_damage);
    assert_eq!(no_damage.object_view(colony).zone, Zone::Graveyard);
    assert!(rats(&no_damage).is_empty());
    let strike = put(&mut game, player, "Lightning Strike", Zone::Hand);
    cast(&mut game, strike, Some(Target::Card(colony)));
    settle(&mut game);
    assert_eq!(game.object_view(colony).zone, Zone::Graveyard);
    let tokens = rats(&game);
    assert_eq!(tokens.len(), 3);
    for token in tokens {
        assert_eq!((token.power, token.toughness, token.colors), (1, 1, 4));
        assert!(token.keywords.contains(Keywords::CANT_BLOCK));
    }
}

#[test]
fn the_end_reduces_at_five_and_searches_the_exiled_permanents_controller() {
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let spell = candidate(&mut game, player, "The End", Zone::Hand);
        let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        game.state.objects[creature as usize].controller = opponent;
        let hand = put(&mut game, opponent, "Bear Cub", Zone::Hand);
        let grave = put(&mut game, opponent, "Bear Cub", Zone::Graveyard);
        let library = put(&mut game, opponent, "Bear Cub", Zone::Library);
        let caster_card = put(&mut game, player, "Bear Cub", Zone::Hand);
        game.state.players[player.index()].mana = [0, 0, 2, 0, 0, 0];
        game.state.players[player.index()].life = 6;
        reject(&mut game, Action::AnnounceCast { card: spell });

        let mut full = game.clone();
        full.state.players[player.index()].mana[5] = 2;
        let walker = put(
            &mut full,
            opponent,
            "Elspeth, Storm Slayer",
            Zone::Battlefield,
        );
        full.add_counter(walker, CounterKind::Loyalty, 4);
        let same_walker = put(&mut full, opponent, "Elspeth, Storm Slayer", Zone::Hand);
        cast(&mut full, spell, Some(Target::Card(walker)));
        let item = full.state.stack.last().unwrap();
        assert_eq!(item.mana_value(), 4);
        assert_eq!(item.cast.unwrap().mana_spent(), 4);
        pass_pair(&mut full);
        assert_eq!(full.object_view(walker).zone, Zone::Exile);
        act(&mut full, Action::FinishSearch);
        assert_eq!(full.object_view(same_walker).zone, Zone::Hand);
        assert_eq!(
            full.state.players[opponent.index()].cards_drawn_this_turn,
            0
        );

        game.state.players[player.index()].life = 5;
        cast(&mut game, spell, Some(Target::Card(creature)));
        let item = game.state.stack.last().unwrap();
        assert_eq!(item.mana_value(), 4);
        assert_eq!(item.cast.unwrap().mana_spent(), 2);
        pass_pair(&mut game);
        assert_eq!(game.object_view(creature).zone, Zone::Exile);
        assert!(game.observe(player).effect_choice.is_some());
        assert!(game.observe(opponent).effect_choice.is_none());
        let object = game.object_ref(caster_card);
        reject(&mut game, Action::SelectResolutionExile { object });
        for card in [hand, grave, library] {
            let object = game.object_ref(card);
            act(&mut game, Action::SelectResolutionExile { object });
        }
        act(&mut game, Action::FinishSearch);
        for card in [hand, grave, library] {
            assert_eq!(game.object_view(card).zone, Zone::Exile);
        }
        assert_eq!(game.object_view(caster_card).zone, Zone::Hand);
        assert_eq!(
            game.state.players[opponent.index()].cards_drawn_this_turn,
            1
        );
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    }
}

#[test]
fn spellscorn_coven_returns_physical_spells_and_discards_on_entry() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let coven = candidate(&mut game, player, "Spellscorn Coven", Zone::Hand);
    let shock = put(&mut game, opponent, "Shock", Zone::Hand);
    let discard = put(&mut game, opponent, "Forest", Zone::Hand);
    let imitator = put(&mut game, opponent, "Slick Imitator", Zone::Battlefield);
    game.state.players[opponent.index()].speed = 4;
    act(&mut game, Action::Pass);
    cast(&mut game, shock, Some(Target::Player(player)));

    let mut copied = game.clone();
    let source = copied.object_ref(imitator);
    act(
        &mut copied,
        Action::AnnounceSourceGrant { source, ability: 0 },
    );
    act(
        &mut copied,
        Action::ChooseTarget {
            target: Target::Spell(shock),
        },
    );
    pay(&mut copied);
    pass_pair(&mut copied);
    act(&mut copied, Action::FinishStackTargets);
    if copied.actor() != Some(player) {
        act(&mut copied, Action::Pass);
    }
    let copy_id = copied
        .state
        .stack
        .iter()
        .find(|item| item.is_copy)
        .unwrap()
        .id;
    act(&mut copied, Action::AnnounceAdventure { card: coven });
    act(
        &mut copied,
        Action::ChooseTarget {
            target: Target::Stack(copy_id),
        },
    );
    pay(&mut copied);
    settle(&mut copied);
    assert_eq!(copied.object_view(coven).zone, Zone::Exile);
    assert_eq!(copied.object_view(shock).zone, Zone::Graveyard);
    assert_eq!(copied.hand_count(opponent), 1);
    assert_eq!(copied.state.players[player.index()].life, 18);

    act(&mut game, Action::Pass);
    act(&mut game, Action::AnnounceAdventure { card: coven });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Spell(shock),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(shock).zone, Zone::Hand);
    assert_eq!(game.object_view(coven).zone, Zone::Exile);
    assert_eq!(game.state.players[player.index()].life, 20);

    act(&mut game, Action::AnnounceExile { card: coven });
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(stats(&game, coven), (2, 3));
    assert!(game.object_view(coven).keywords.contains(Keywords::FLYING));
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    let before = game.snapshot();
    assert_eq!(
        game.apply(player, Action::Discard { card: shock }),
        Err(Error::WrongActor)
    );
    assert_eq!(game.snapshot(), before);
    act(&mut game, Action::Discard { card: discard });
    settle(&mut game);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(game.object_view(shock).zone, Zone::Hand);
}

#[test]
fn swift_spiral_returns_a_nontoken_to_its_owner_and_twining_twins_has_ward() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let twins = candidate(&mut game, player, "Twining Twins", Zone::Hand);
    let original = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.state.objects[original as usize].controller = opponent;
    let rat_out = put(&mut game, player, "Rat Out", Zone::Hand);
    announce(&mut game, rat_out, &[]);
    settle(&mut game);
    let rat = game
        .observe(player)
        .objects
        .into_iter()
        .find(|object| {
            object.zone == Zone::Battlefield && object.is_token() && object.has_subtype("Rat")
        })
        .unwrap()
        .id;
    act(&mut game, Action::AnnounceAdventure { card: twins });
    reject(&mut game, target(rat));
    act(&mut game, target(original));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(original).zone, Zone::Exile);
    let exiled = game.object_ref(original);
    act(&mut game, Action::AnnounceExile { card: twins });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, twins), (4, 4));
    assert!(game
        .object_view(twins)
        .keywords
        .contains(Keywords::FLYING.union(Keywords::VIGILANCE)));

    act(&mut game, Action::Pass);
    let murder = put(&mut game, opponent, "Murder", Zone::Hand);
    cast(&mut game, murder, Some(Target::Card(twins)));
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    let mut unpaid = game.clone();
    act(&mut unpaid, Action::DeclineResolutionMana);
    settle(&mut unpaid);
    assert_eq!(unpaid.object_view(twins).zone, Zone::Battlefield);
    assert_eq!(unpaid.object_view(murder).zone, Zone::Graveyard);
    act(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 1],
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(twins).zone, Zone::Graveyard);
    assert_eq!(game.object_view(original).zone, Zone::Exile);

    for _ in 0..16 {
        if game.state.step == Step::End {
            break;
        }
        let action = match game.state.pending {
            Pending::Attackers => Action::FinishAttackers,
            Pending::Blockers => Action::FinishBlockers,
            _ => Action::Pass,
        };
        act(&mut game, action);
    }
    assert_eq!(game.state.step, Step::End);
    settle(&mut game);
    assert_eq!(game.object_view(original).zone, Zone::Battlefield);
    assert_eq!(game.object_view(original).controller, player);
    assert_ne!(game.object_ref(original), exiled);
}

fn sacrifice_bodyguard(game: &mut Game, card: CardId, controller: Player) {
    if game.actor() != Some(controller) {
        act(game, Action::Pass);
    }
    choose(game, |action| match action {
        Action::Activate {
            card: source,
            ability: 0,
            target: None,
            ..
        }
        | Action::AnnounceActivate {
            card: source,
            ability: 0,
            ..
        } => *source == card,
        _ => false,
    });
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
    assert_eq!(game.object_view(card).zone, Zone::Graveyard);
}

#[test]
fn werefox_bodyguard_can_flash_and_returns_its_exact_card_to_the_owner() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let guard = candidate(&mut game, player, "Werefox Bodyguard", Zone::Hand);
    let other_fox = candidate(&mut game, opponent, "Werefox Bodyguard", Zone::Battlefield);
    let bearer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.state.objects[bearer as usize].controller = opponent;
    act(&mut game, Action::Pass);
    cast(&mut game, guard, None);
    pass_pair(&mut game);
    assert_eq!(stats(&game, guard), (2, 2));
    assert!(game.object_view(guard).keywords.contains(Keywords::FLASH));
    reject(&mut game, target(other_fox));
    reject(&mut game, target(guard));

    let mut declined = game.clone();
    act(&mut declined, Action::FinishTargets);
    settle(&mut declined);
    assert_eq!(declined.object_view(bearer).controller, opponent);
    assert_eq!(declined.object_view(bearer).zone, Zone::Battlefield);

    act(&mut game, target(bearer));
    let mut early = game.clone();
    let original = early.object_ref(bearer);
    sacrifice_bodyguard(&mut early, guard, player);
    settle(&mut early);
    assert_eq!(early.object_ref(bearer), original);
    assert_eq!(early.object_view(bearer).controller, opponent);
    assert_eq!(early.state.players[player.index()].life, 22);

    settle(&mut game);
    assert_eq!(game.object_view(bearer).zone, Zone::Exile);
    let exiled = game.object_ref(bearer);
    sacrifice_bodyguard(&mut game, guard, player);
    settle(&mut game);
    assert_eq!(game.object_view(bearer).zone, Zone::Battlefield);
    assert_eq!(game.object_view(bearer).controller, player);
    assert_ne!(game.object_ref(bearer), exiled);
    assert_eq!(game.state.players[player.index()].life, 22);
}

#[test]
fn talion_chooses_as_it_enters_and_matches_opposing_casts_once() {
    let number = |value| Action::ChooseEntryValue {
        value: EntryChoiceValue::Number(value),
    };
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let talion = candidate(&mut game, player, "Talion, the Kindly Lord", Zone::Hand);
        cast(&mut game, talion, None);
        pass_pair(&mut game);
        assert_ne!(game.object_view(talion).zone, Zone::Battlefield);
        assert_eq!(
            game.legal_actions()
                .into_iter()
                .filter(|action| matches!(action, Action::ChooseEntryValue { .. }))
                .collect::<Vec<_>>(),
            (1..=10).map(number).collect::<Vec<_>>()
        );
        reject(&mut game, number(0));
        reject(&mut game, number(11));
        let mut two = game.clone();
        act(&mut two, number(2));
        act(&mut game, number(3));
        assert_eq!(stats(&game, talion), (3, 4));
        assert!(game.object_view(talion).keywords.contains(Keywords::FLYING));
        assert_eq!(
            game.object_view(talion).entry_choice,
            Some(EntryChoiceValue::Number(3))
        );

        let own = put(&mut two, player, "Bear Cub", Zone::Hand);
        cast(&mut two, own, None);
        settle(&mut two);
        assert_eq!(two.state.players[opponent.index()].life, 20);
        assert_eq!(two.state.players[player.index()].cards_drawn_this_turn, 0);
        two.state.active = opponent;
        two.state.priority = opponent;
        let matching_all_three = put(&mut two, opponent, "Bear Cub", Zone::Hand);
        cast(&mut two, matching_all_three, None);
        settle(&mut two);
        assert_eq!(two.state.players[opponent.index()].life, 18);
        assert_eq!(two.state.players[player.index()].cards_drawn_this_turn, 1);

        game.state.active = opponent;
        game.state.priority = opponent;
        for (name, matches) in [
            ("Bear Cub", false),
            ("Stingblade Assassin", true),
            ("Barrow Naughty", true),
            ("Quick Study", true),
        ] {
            let before = game.state.players[opponent.index()].life;
            let drawn = game.state.players[player.index()].cards_drawn_this_turn;
            let spell = put(&mut game, opponent, name, Zone::Hand);
            cast(&mut game, spell, None);
            settle(&mut game);
            assert_eq!(
                game.state.players[opponent.index()].life,
                before - if matches { 2 } else { 0 },
                "{name}"
            );
            assert_eq!(
                game.state.players[player.index()].cards_drawn_this_turn,
                drawn + u64::from(matches),
                "{name}"
            );
        }
    }
}

#[test]
fn talions_messenger_loots_once_and_targets_a_faerie_after_the_discard() {
    let player = Player::P1;
    let mut game = scenario(player);
    let messenger = candidate(&mut game, player, "Talion's Messenger", Zone::Hand);
    let discard = put(&mut game, player, "Island", Zone::Hand);
    let first = put(&mut game, player, "Barrow Naughty", Zone::Battlefield);
    let second = put(&mut game, player, "Faerie Dreamthief", Zone::Battlefield);
    let nonfaerie = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foreign = put(
        &mut game,
        player.other(),
        "Faerie Dreamthief",
        Zone::Battlefield,
    );
    cast(&mut game, messenger, None);
    settle(&mut game);
    assert_eq!(stats(&game, messenger), (1, 3));
    assert!(game
        .object_view(messenger)
        .keywords
        .contains(Keywords::FLYING));
    begin_attackers(&mut game);
    let mut ground = game.clone();
    act(&mut ground, Action::Attack { card: nonfaerie });
    act(&mut ground, Action::FinishAttackers);
    assert!(ground.state.stack.is_empty());
    assert_eq!(
        ground.state.players[player.index()].cards_drawn_this_turn,
        0
    );

    for card in [first, second] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    assert_eq!(game.state.stack.len(), 1);
    pass_pair(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(game.counter_count(messenger, CounterKind::PlusOne), 0);
    reject(&mut game, target(messenger));
    act(&mut game, Action::Discard { card: discard });
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    reject(&mut game, target(nonfaerie));
    reject(&mut game, target(foreign));
    act(&mut game, target(messenger));
    settle(&mut game);
    assert_eq!(game.counter_count(messenger, CounterKind::PlusOne), 1);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn goose_mother_enters_with_x_and_rounds_food_up_then_offers_a_paid_draw() {
    let player = Player::P0;
    for (x, expected_food) in [(0, 0), (2, 1), (3, 2)] {
        let mut game = scenario(player);
        let mother = candidate(&mut game, player, "The Goose Mother", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: mother });
        act(&mut game, Action::ChooseX { value: x });
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 1, 0, 0, 1, x],
            },
        );
        settle(&mut game);
        assert_eq!(stats(&game, mother), (2 + i128::from(x), 2 + i128::from(x)));
        assert_eq!(
            game.counter_count(mother, CounterKind::PlusOne),
            u64::from(x)
        );
        assert!(game.object_view(mother).keywords.contains(Keywords::FLYING));
        let food = game
            .observe(player)
            .objects
            .into_iter()
            .filter(|object| {
                object.zone == Zone::Battlefield
                    && object.controller == player
                    && object.is_token()
                    && object.has_subtype("Food")
            })
            .map(|object| game.object_ref(object.id))
            .collect::<Vec<_>>();
        assert_eq!(food.len(), expected_food);
        if x != 3 {
            continue;
        }

        game.state.objects[mother as usize].summoning_sick = false;
        begin_attackers(&mut game);
        act(&mut game, Action::Attack { card: mother });
        act(&mut game, Action::FinishAttackers);
        pass_pair(&mut game);
        let invalid = game.object_ref(mother);
        reject(&mut game, Action::Sacrifice { object: invalid });
        let mut decline = game.clone();
        act(&mut decline, Action::DeclineEffectPayment);
        settle(&mut decline);
        assert_eq!(
            decline.state.players[player.index()].cards_drawn_this_turn,
            0
        );
        assert_eq!(decline.object_view(food[0].card).zone, Zone::Battlefield);

        act(&mut game, Action::Sacrifice { object: food[0] });
        settle(&mut game);
        assert_ne!(game.object_view(food[0].card).zone, Zone::Battlefield);
        assert_eq!(game.object_view(food[1].card).zone, Zone::Battlefield);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    }
}

#[test]
fn wildwood_mentor_counts_each_token_and_boosts_with_its_departure_power() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let mentor = candidate(&mut game, player, "Wildwood Mentor", Zone::Hand);
    let dinner = put(&mut game, player, "Devouring Sugarmaw", Zone::Hand);
    let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let idle = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let growth = put(&mut game, player, "Giant Growth", Zone::Hand);
    let removal = put(&mut game, opponent, "Murder", Zone::Hand);
    let forest = put(&mut game, player, "Forest", Zone::Battlefield);
    let swamps = [0; 3].map(|_| put(&mut game, opponent, "Swamp", Zone::Battlefield));
    cast(&mut game, mentor, None);
    settle(&mut game);
    assert_eq!(stats(&game, mentor), (1, 1));
    act(&mut game, Action::AnnounceAdventure { card: dinner });
    pay(&mut game);
    pass_pair(&mut game);
    for _ in 0..2 {
        if matches!(game.state.pending, Pending::OrderTriggers { .. }) {
            choose(&mut game, |action| {
                matches!(action, Action::PutTrigger { .. })
            });
        }
    }
    assert_eq!(game.state.stack.len(), 2);
    settle(&mut game);
    assert_eq!(game.counter_count(mentor, CounterKind::PlusOne), 2);

    game.state.objects[mentor as usize].summoning_sick = false;
    begin_attackers(&mut game);
    for card in [mentor, attacker] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    reject(&mut game, target(mentor));
    reject(&mut game, target(idle));
    act(&mut game, target(attacker));
    let mut unchanged = game.clone();
    settle(&mut unchanged);
    assert_eq!(stats(&unchanged, attacker), (5, 5));

    act(&mut game, Action::TapMana { card: forest });
    cast(&mut game, growth, Some(Target::Card(mentor)));
    pass_pair(&mut game);
    assert_eq!(stats(&game, mentor), (6, 6));
    act(&mut game, Action::Pass);
    for card in swamps {
        act(&mut game, Action::TapMana { card });
    }
    cast(&mut game, removal, Some(Target::Card(mentor)));
    settle(&mut game);
    assert_eq!(game.object_view(mentor).zone, Zone::Graveyard);
    assert_eq!(stats(&game, attacker), (8, 8));
}

#[test]
fn stockpiling_celebrant_returns_another_nonland_then_scries_privately() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let celebrant = candidate(&mut game, player, "Stockpiling Celebrant", Zone::Hand);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let foreign = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let bounce = put(&mut game, opponent, "Unsummon", Zone::Hand);
    let before = game.state.players[player.index()].library.clone();
    let top = *before.last().unwrap();
    let next = before[before.len() - 2];

    let mut no_target = game.clone();
    cast(&mut no_target, celebrant, None);
    settle(&mut no_target);
    assert_eq!(stats(&no_target, celebrant), (3, 2));
    assert_eq!(no_target.state.players[player.index()].library, before);

    let stolen = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let exact = game.object_ref(stolen);
    game.change_controller(exact, player);
    cast(&mut game, celebrant, None);
    pass_pair(&mut game);
    for invalid in [celebrant, land, foreign] {
        reject(&mut game, target(invalid));
    }
    act(&mut game, target(stolen));

    let mut stale = game.clone();
    act(&mut stale, Action::Pass);
    cast(&mut stale, bounce, Some(Target::Card(stolen)));
    settle(&mut stale);
    assert_eq!(stale.state.players[player.index()].library, before);

    pass_pair(&mut game);
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    settle(&mut decline);
    assert_eq!(decline.object_view(stolen).zone, Zone::Battlefield);
    assert_eq!(decline.state.players[player.index()].library, before);

    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.object_view(stolen).zone, Zone::Hand);
    assert_eq!(game.object_view(stolen).owner, opponent);
    assert!(game.observe(player).library_choice.is_some());
    assert!(game.observe(opponent).library_choice.is_none());
    for (card, destination) in [
        (top, LibraryDestination::Bottom),
        (next, LibraryDestination::Top),
    ] {
        act(&mut game, Action::ArrangeLibrary { card, destination });
    }
    settle(&mut game);
    assert_eq!(game.top_library_card(player), Some(next));
    assert_eq!(
        game.state.players[player.index()].library.first(),
        Some(&top)
    );
}

fn pass_to_later_main(game: &mut Game, player: Player) {
    let turn = game.state.turn;
    for _ in 0..48 {
        if game.state.turn > turn && game.state.active == player && game.state.step == Step::Main1 {
            return;
        }
        let action = match game.state.pending {
            Pending::Attackers => Action::FinishAttackers,
            Pending::Blockers => Action::FinishBlockers,
            Pending::Priority => Action::Pass,
            _ => panic!("unexpected pending state {:?}", game.state.pending),
        };
        act(game, action);
    }
    panic!("later main phase did not arrive");
}

#[test]
fn twisted_fealty_has_independent_theft_and_wicked_role_targets() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = candidate(&mut game, player, "Twisted Fealty", Zone::Hand);
    let stolen = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let other = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let bounce = put(&mut game, opponent, "Unsummon", Zone::Hand);
    game.state.objects[stolen as usize].tapped = true;
    game.add_counter(stolen, CounterKind::Stun, 1);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, target(stolen));
    let mut no_role = game.clone();
    act(&mut no_role, Action::FinishTargets);
    pay(&mut no_role);
    settle(&mut no_role);
    assert_eq!(no_role.object_view(stolen).controller, player);
    assert_eq!(stats(&no_role, stolen), (2, 2));

    let mut only_role = game.clone();
    act(&mut only_role, target(other));
    pay(&mut only_role);
    act(&mut only_role, Action::Pass);
    cast(&mut only_role, bounce, Some(Target::Card(stolen)));
    settle(&mut only_role);
    assert_eq!(only_role.object_view(stolen).zone, Zone::Hand);
    assert_eq!(only_role.object_view(other).controller, opponent);
    assert_eq!(stats(&only_role, other), (3, 3));
    assert!(!only_role
        .object_view(other)
        .keywords
        .contains(Keywords::HASTE));

    act(&mut game, target(stolen));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(stolen).controller, player);
    assert!(game.object_view(stolen).tapped);
    assert_eq!(game.counter_count(stolen, CounterKind::Stun), 0);
    assert!(game.object_view(stolen).keywords.contains(Keywords::HASTE));
    assert_eq!(stats(&game, stolen), (3, 3));
    let exact = game.object_ref(stolen);
    assert!(game.observe(player).objects.iter().any(|object| {
        object.zone == Zone::Battlefield
            && object.controller == player
            && object.card == Card::from_catalogue_id(5248)
            && object.attached_to == Some(exact)
    }));
    pass_to_later_main(&mut game, opponent);
    assert_eq!(game.object_view(stolen).controller, opponent);
    assert!(!game.object_view(stolen).keywords.contains(Keywords::HASTE));
    assert_eq!(stats(&game, stolen), (3, 3));
}

#[test]
fn tough_cookie_animates_an_owned_noncreature_artifact_and_has_its_own_food_ability() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let cookie = candidate(&mut game, player, "Tough Cookie", Zone::Hand);
    let foreign = put(&mut game, opponent, "Candy Trail", Zone::Battlefield);
    let lands = [0; 2].map(|_| put(&mut game, player, "Forest", Zone::Battlefield));
    let activation = |card, ability| Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    };
    cast(&mut game, cookie, None);
    settle(&mut game);
    assert_eq!(stats(&game, cookie), (2, 2));
    assert_eq!(game.object_view(cookie).kinds & 17, 17);
    assert!(game.object_view(cookie).has_subtype("Food"));
    let food = game
        .observe(player)
        .objects
        .into_iter()
        .find(|object| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.is_token()
                && object.has_subtype("Food")
        })
        .unwrap()
        .id;
    assert_eq!(game.object_view(food).kinds & 17, 16);
    assert!(!game.legal_actions().iter().any(|action| matches!(
        action,
        Action::Activate { card, ability: 1, .. } | Action::AnnounceActivate { card, ability: 1, .. } if *card == cookie
    )));

    act(&mut game, activation(cookie, 0));
    reject(&mut game, target(cookie));
    reject(&mut game, target(foreign));
    act(&mut game, target(food));
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 1, 2],
        },
    );
    settle(&mut game);
    assert_eq!(stats(&game, food), (4, 4));
    assert_eq!(game.object_view(food).kinds & 17, 17);
    assert!(game.object_view(food).has_subtype("Food"));
    assert!(!game.legal_actions().iter().any(|action| matches!(
        action,
        Action::Activate { card, .. } | Action::AnnounceActivate { card, .. } if *card == food
    )));

    pass_to_later_main(&mut game, player);
    assert_eq!(game.object_view(food).kinds & 17, 16);
    for card in lands {
        act(&mut game, Action::TapMana { card });
    }
    choose(&mut game, |action| {
        matches!(
            action,
            Action::Activate { card, ability: 1, .. } | Action::AnnounceActivate { card, ability: 1, .. } if *card == cookie
        )
    });
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    assert_eq!(game.object_view(cookie).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 20);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 23);
    assert_eq!(game.object_view(food).zone, Zone::Battlefield);
}

#[test]
fn totentanz_counts_self_and_nontoken_deaths_and_gives_attacking_rats_deathtouch() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let piper = candidate(&mut game, player, "Totentanz, Swarm Piper", Zone::Hand);
    let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foreign = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let lands = [0; 2].map(|_| put(&mut game, player, "Swamp", Zone::Battlefield));
    let song = put(&mut game, player, "Song of Totentanz", Zone::Hand);
    let day = put(&mut game, player, "Day of Judgment", Zone::Hand);
    let rite = put(&mut game, player, "Rite of Replication", Zone::Hand);
    let murder = put(&mut game, player, "Murder", Zone::Hand);
    let rats = |game: &Game| {
        game.observe(player)
            .objects
            .into_iter()
            .filter(|object| {
                object.zone == Zone::Battlefield
                    && object.controller == player
                    && object.is_token()
                    && object.has_subtype("Rat")
            })
            .collect::<Vec<_>>()
    };
    let order = |game: &mut Game| {
        for _ in 0..2 {
            if matches!(game.state.pending, Pending::OrderTriggers { .. }) {
                choose(game, |action| matches!(action, Action::PutTrigger { .. }));
            }
        }
    };
    cast(&mut game, piper, None);
    settle(&mut game);
    assert_eq!(stats(&game, piper), (2, 3));
    act(&mut game, Action::AnnounceCast { card: song });
    act(&mut game, Action::ChooseX { value: 2 });
    pay(&mut game);
    settle(&mut game);
    let before = rats(&game);
    assert_eq!(before.len(), 2);

    let mut swept = game.clone();
    cast(&mut swept, day, None);
    pass_pair(&mut swept);
    order(&mut swept);
    assert_eq!(swept.state.stack.len(), 2);
    settle(&mut swept);
    assert_eq!(swept.object_view(piper).zone, Zone::Graveyard);
    let after = rats(&swept);
    assert_eq!(after.len(), 2);
    assert!(after.iter().all(|rat| {
        rat.power == 1
            && rat.toughness == 1
            && rat.keywords.contains(Keywords::CANT_BLOCK)
            && before.iter().all(|old| old.reference != rat.reference)
    }));

    let mut copied = game.clone();
    act(&mut copied, Action::AnnounceCast { card: rite });
    act(&mut copied, Action::ChooseKicker { pay: false });
    act(&mut copied, target(piper));
    pay(&mut copied);
    pass_pair(&mut copied);
    let token = copied
        .observe(player)
        .objects
        .into_iter()
        .find(|object| {
            object.zone == Zone::Battlefield
                && object.is_token()
                && object.name == "Totentanz, Swarm Piper"
        })
        .unwrap()
        .reference;
    act(&mut copied, Action::KeepLegend { object: token });
    order(&mut copied);
    settle(&mut copied);
    let rat_count = rats(&copied).len();
    cast(&mut copied, murder, Some(Target::Card(token.card)));
    settle(&mut copied);
    assert_eq!(rats(&copied).len(), rat_count + 1);

    let announce = Action::AnnounceActivate {
        card: piper,
        ability: 0,
        counter_payment: Counters::default(),
    };
    reject(&mut game, announce.clone());
    begin_attackers(&mut game);
    act(&mut game, Action::Attack { card: before[0].id });
    act(&mut game, Action::Attack { card: own });
    act(&mut game, Action::FinishAttackers);
    for card in lands {
        act(&mut game, Action::TapMana { card });
    }
    act(&mut game, announce);
    reject(&mut game, target(before[1].id));
    reject(&mut game, target(own));
    act(&mut game, target(before[0].id));
    pay(&mut game);
    settle(&mut game);
    assert!(game
        .object_view(before[0].id)
        .keywords
        .contains(Keywords::DEATHTOUCH));
    pass_pair(&mut game);
    act(
        &mut game,
        Action::Block {
            blocker: foreign,
            attacker: before[0].id,
        },
    );
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    assert_eq!(game.object_view(foreign).zone, Zone::Graveyard);
    assert_eq!(rats(&game).len(), 1);
}
