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
