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
        assert!(
            attacking
                .object_view(slinger)
                .keywords
                .contains(Keywords::REACH)
        );
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
    assert!(
        qualified
            .object_view(wolf)
            .keywords
            .contains(Keywords::DEFENDER)
    );
    assert!(
        qualified
            .legal_actions()
            .contains(&Action::Attack { card: wolf })
    );
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
    assert!(
        game.object_view(twins)
            .keywords
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );

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
    assert!(
        game.object_view(messenger)
            .keywords
            .contains(Keywords::FLYING)
    );
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
    assert!(
        !only_role
            .object_view(other)
            .keywords
            .contains(Keywords::HASTE)
    );

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
    assert!(
        game.object_view(before[0].id)
            .keywords
            .contains(Keywords::DEATHTOUCH)
    );
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

fn cast_adventure(game: &mut Game, card: CardId, target: Option<Target>) {
    act(game, Action::AnnounceAdventure { card });
    if let Some(target) = target {
        act(game, Action::ChooseTarget { target });
    }
    pay(game);
}

fn advance_to_step(game: &mut Game, active: Player, step: Step) {
    for _ in 0..64 {
        if game.state.active == active && game.state.step == step {
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
    panic!("step {step:?} did not arrive for {active:?}");
}

#[test]
fn troyan_funds_expensive_or_printed_x_spells_and_loots_for_blue() {
    let player = Player::P0;
    let mut game = scenario(player);
    let troyan = candidate(
        &mut game,
        player,
        "Troyan, Gutsy Explorer",
        Zone::Battlefield,
    );
    let vault = put(&mut game, player, "Collector's Vault", Zone::Battlefield);
    let opt = put(&mut game, player, "Opt", Zone::Hand);
    let goose = put(&mut game, player, "The Goose Mother", Zone::Hand);
    let wurm = put(&mut game, player, "Beanstalk Wurm", Zone::Hand);
    let produce = Action::ActivateMana {
        card: troyan,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: [0, 1, 0, 0, 1, 0],
    };
    assert_eq!(stats(&game, troyan), (1, 3));

    let mut loot = game.clone();
    loot.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
    let drawn = loot.top_library_card(player).unwrap();
    act(
        &mut loot,
        Action::AnnounceActivate {
            card: troyan,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut loot);
    assert!(loot.object_view(troyan).tapped);
    assert_eq!(loot.state.players[player.index()].mana, [0; 6]);
    pass_pair(&mut loot);
    assert_eq!(loot.object_view(drawn).zone, Zone::Hand);
    act(&mut loot, Action::Discard { card: opt });
    settle(&mut loot);
    assert_eq!(loot.object_view(opt).zone, Zone::Graveyard);

    game.state.players[player.index()].mana = [0; 6];
    let mut x_zero = game.clone();
    act(&mut x_zero, Action::AnnounceCast { card: goose });
    act(&mut x_zero, Action::ChooseX { value: 0 });
    act(&mut x_zero, produce.clone());
    pay(&mut x_zero);
    assert_eq!(x_zero.state.stack.last().unwrap().mana_value(), 2);
    settle(&mut x_zero);
    assert_eq!(x_zero.object_view(goose).zone, Zone::Battlefield);
    assert_eq!(stats(&x_zero, goose), (2, 2));

    act(&mut game, produce);
    assert!(game.object_view(troyan).tapped);
    reject(&mut game, Action::AnnounceCast { card: opt });
    reject(
        &mut game,
        Action::AnnounceActivate {
            card: vault,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    game.state.players[player.index()].mana[5] = 3;
    reject(&mut game, Action::AnnounceAdventure { card: wurm });
    cast(&mut game, wurm, None);
    settle(&mut game);
    assert_eq!(stats(&game, wurm), (5, 4));
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn up_the_beanstalk_draws_on_entry_and_uses_the_committed_spell_mana_value() {
    let card = Card::from_name("Up the Beanstalk").unwrap();
    let mut deck = vec![Card::Forest; 59];
    deck.push(card);
    let mut config = Config {
        decks: [deck, vec![Card::Forest; 60]],
        seed: 20261110,
        starting_player: Player::P0,
        deck_rules: DeckRules::Standard,
    };
    assert!(
        matches!(Game::new(config.clone()), Err(Error::InvalidDeck(message))
        if message == "not Standard legal in this catalogue: Up the Beanstalk")
    );
    config.deck_rules = DeckRules::Freeform;
    assert_eq!(Game::new(config).is_ok(), card.is_supported());

    let player = Player::P0;
    let mut game = scenario(player);
    let beanstalk = candidate(&mut game, player, "Up the Beanstalk", Zone::Hand);
    let on_entry = game.top_library_card(player).unwrap();
    cast(&mut game, beanstalk, None);
    settle(&mut game);
    assert_eq!(game.object_view(on_entry).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);

    let goose = put(&mut game, player, "The Goose Mother", Zone::Hand);
    for (x, draws) in [(2, 1), (3, 2)] {
        let mut branch = game.clone();
        act(&mut branch, Action::AnnounceCast { card: goose });
        act(&mut branch, Action::ChooseX { value: x });
        pay(&mut branch);
        settle(&mut branch);
        assert_eq!(
            branch.state.players[player.index()].cards_drawn_this_turn,
            draws
        );
    }

    let wurm = put(&mut game, player, "Beanstalk Wurm", Zone::Hand);
    cast_adventure(&mut game, wurm, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    act(&mut game, Action::AnnounceExile { card: wurm });
    pay(&mut game);
    assert_eq!(game.state.stack.len(), 2);
    pass_pair(&mut game);
    assert_eq!(game.object_view(wurm).zone, Zone::Stack);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    settle(&mut game);

    for name in ["Shock", "Opt", "Murder"] {
        put(&mut game, player, name, Zone::Graveyard);
    }
    let elemental = put(&mut game, player, "Hearth Elemental", Zone::Hand);
    let mana_before: u32 = game.state.players[player.index()].mana.iter().sum();
    cast(&mut game, elemental, None);
    let mana_after: u32 = game.state.players[player.index()].mana.iter().sum();
    assert_eq!(mana_before - mana_after, 3);
    assert_eq!(game.state.stack[0].mana_value(), 6);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 3);

    let opponent = player.other();
    game.state.active = opponent;
    game.state.priority = opponent;
    let opposing_wurm = put(&mut game, opponent, "Beanstalk Wurm", Zone::Hand);
    cast(&mut game, opposing_wurm, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 3);
}

#[test]
fn virtue_of_courage_offers_actual_noncombat_damage_and_expires_its_play_permission() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let name = "Virtue of Courage // Embereth Blaze";
    let virtue = candidate(&mut game, player, name, Zone::Hand);
    let blaze = candidate(&mut game, player, name, Zone::Hand);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
    cast(&mut game, virtue, None);
    settle(&mut game);

    let mut creature_damage = game.clone();
    cast_adventure(&mut creature_damage, blaze, Some(Target::Card(enemy)));
    settle(&mut creature_damage);
    assert_eq!(creature_damage.object_view(enemy).damage, 2);
    assert_eq!(creature_damage.object_view(blaze).zone, Zone::Exile);

    let mut combat = game.clone();
    declare_blocks(&mut combat, &[bear], &[]);
    pass_pair(&mut combat);
    settle(&mut combat);
    assert_eq!(combat.state.players[opponent.index()].life, 18);

    let mut opposing_source = game.clone();
    act(&mut opposing_source, Action::Pass);
    let shock = put(&mut opposing_source, opponent, "Shock", Zone::Hand);
    cast(&mut opposing_source, shock, Some(Target::Player(player)));
    settle(&mut opposing_source);
    assert_eq!(opposing_source.state.players[player.index()].life, 18);

    let spare = game.top_library_card(player).unwrap();
    let land = put(&mut game, player, "Mountain", Zone::Library);
    let opt = put(&mut game, player, "Opt", Zone::Library);
    cast_adventure(&mut game, blaze, Some(Target::Player(opponent)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(blaze).zone, Zone::Exile);
    assert_eq!(game.state.players[opponent.index()].life, 18);
    assert_eq!(game.state.stack.len(), 1);
    pass_pair(&mut game);

    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    settle(&mut declined);
    assert_eq!(declined.top_library_card(player), Some(opt));

    act(&mut game, Action::ChooseOptional { accept: true });
    settle(&mut game);
    assert_eq!(game.object_view(opt).zone, Zone::Exile);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    assert_eq!(game.top_library_card(player), Some(spare));
    assert!(
        game.legal_actions()
            .contains(&Action::AnnounceExile { card: opt })
    );
    act(&mut game, Action::PlayLand { card: land });
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    pass_to_later_main(&mut game, player);
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
    reject(&mut game, Action::AnnounceExile { card: opt });
    assert_eq!(game.object_view(opt).zone, Zone::Exile);
}

#[test]
fn virtue_of_knowledge_adds_only_owned_entry_triggers_and_visions_retargets_an_ability() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let name = "Virtue of Knowledge // Vantress Visions";
    for _ in 0..2 {
        let virtue = candidate(&mut game, player, name, Zone::Hand);
        cast(&mut game, virtue, None);
        settle(&mut game);
    }
    let visions = candidate(&mut game, player, name, Zone::Hand);
    let witch = put(&mut game, player, "Sweettooth Witch", Zone::Hand);
    cast(&mut game, witch, None);
    reject(&mut game, Action::AnnounceAdventure { card: visions });
    pass_pair(&mut game);
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.state.stack.len(), 3);
    assert!(game.state.stack.iter().all(|trigger| {
        trigger.source.unwrap().id == witch
            && matches!(trigger.kind, StackKind::TriggeredAbility { .. })
    }));
    let trigger = game.state.stack.last().unwrap().id;
    let mut copied_trigger = game.clone();
    cast_adventure(&mut copied_trigger, visions, Some(Target::Stack(trigger)));
    settle(&mut copied_trigger);
    assert_eq!(
        copied_trigger
            .observe(player)
            .objects
            .iter()
            .filter(|object| object.zone == Zone::Battlefield
                && object.controller == player
                && object.has_subtype("Food"))
            .count(),
        4
    );
    settle(&mut game);

    let cookie = put(&mut game, player, "Tough Cookie", Zone::Battlefield);
    let first = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let second = put(&mut game, player, "Collector's Vault", Zone::Battlefield);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: cookie,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(&mut game, target(first));
    pay(&mut game);
    let ability = game.state.stack.last().unwrap().id;
    cast_adventure(&mut game, visions, Some(Target::Stack(ability)));
    pass_pair(&mut game);
    act(
        &mut game,
        Action::SetStackTarget {
            index: 0,
            target: Target::Card(second),
        },
    );
    act(&mut game, Action::FinishStackTargets);
    assert_eq!(game.state.stack.len(), 2);
    assert!(game.state.stack.last().unwrap().is_copy);
    settle(&mut game);
    assert_eq!(stats(&game, first), (4, 4));
    assert_eq!(stats(&game, second), (4, 4));
    assert_eq!(game.object_view(visions).zone, Zone::Exile);

    game.state.active = opponent;
    game.state.priority = opponent;
    let foreign = put(&mut game, opponent, "Sweettooth Witch", Zone::Hand);
    let our_visions = candidate(&mut game, player, name, Zone::Hand);
    cast(&mut game, foreign, None);
    pass_pair(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    act(&mut game, Action::Pass);
    reject(&mut game, Action::AnnounceAdventure { card: our_visions });
    settle(&mut game);
}

#[test]
fn virtue_of_loyalty_makes_a_vigilant_knight_and_counters_then_untaps_its_team() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let virtue = candidate(
        &mut game,
        player,
        "Virtue of Loyalty // Ardenvale Fealty",
        Zone::Hand,
    );
    cast_adventure(&mut game, virtue, None);
    settle(&mut game);
    let knight = game
        .observe(player)
        .objects
        .into_iter()
        .find(|object| object.zone == Zone::Battlefield && object.is_token())
        .unwrap();
    assert_eq!(knight.controller, player);
    assert_eq!(knight.colors, 1);
    assert!(knight.has_subtype("Knight") && knight.keywords.contains(Keywords::VIGILANCE));
    assert_eq!((knight.power, knight.toughness), (2, 2));
    assert_eq!(game.object_view(virtue).zone, Zone::Exile);
    act(&mut game, Action::AnnounceExile { card: virtue });
    pay(&mut game);
    settle(&mut game);

    let ready = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let stunned = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, player, "Plains", Zone::Battlefield);
    let foreign = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    for card in [ready, stunned, land, foreign] {
        game.state.objects[card as usize].tapped = true;
    }
    game.add_counter(stunned, CounterKind::Stun, 1);
    advance_to_step(&mut game, player, Step::End);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    for card in [knight.id, ready, stunned] {
        assert_eq!(game.counter_count(card, CounterKind::PlusOne), 1);
    }
    assert!(!game.object_view(ready).tapped);
    assert!(game.object_view(stunned).tapped);
    assert_eq!(game.counter_count(stunned, CounterKind::Stun), 0);
    for card in [land, foreign] {
        assert!(game.object_view(card).tapped);
        assert_eq!(game.counter_count(card, CounterKind::PlusOne), 0);
    }
    advance_to_step(&mut game, opponent, Step::End);
    assert!(game.state.stack.is_empty());
    assert_eq!(game.counter_count(knight.id, CounterKind::PlusOne), 1);
}

#[test]
fn virtue_of_persistence_returns_a_creature_from_either_graveyard_and_scorn_can_fizzle() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let virtue = candidate(
        &mut game,
        player,
        "Virtue of Persistence // Locthwain Scorn",
        Zone::Hand,
    );
    let giant = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
    let own = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let enemy = put(&mut game, opponent, "Bear Cub", Zone::Graveyard);
    let noncreature = put(&mut game, opponent, "Forest", Zone::Graveyard);
    cast_adventure(&mut game, virtue, Some(Target::Card(giant)));

    let mut stale = game.clone();
    act(&mut stale, Action::Pass);
    let bounce = put(&mut stale, opponent, "Unsummon", Zone::Hand);
    cast(&mut stale, bounce, Some(Target::Card(giant)));
    settle(&mut stale);
    assert_eq!(stale.object_view(virtue).zone, Zone::Graveyard);
    assert_eq!(stale.state.players[player.index()].life, 20);

    settle(&mut game);
    assert_eq!(stats(&game, giant), (7, 7));
    assert_eq!(game.state.players[player.index()].life, 22);
    act(&mut game, Action::AnnounceExile { card: virtue });
    pay(&mut game);
    settle(&mut game);
    advance_to_step(&mut game, opponent, Step::Upkeep);
    assert!(game.state.stack.is_empty());
    assert_eq!(stats(&game, giant), (10, 10));
    advance_to_step(&mut game, player, Step::Upkeep);
    reject(&mut game, target(noncreature));
    for creature in [own, enemy] {
        let mut branch = game.clone();
        act(&mut branch, target(creature));
        settle(&mut branch);
        let view = branch.object_view(creature);
        assert_eq!(view.zone, Zone::Battlefield);
        assert_eq!(view.controller, player);
        assert_eq!(view.owner, game.object_view(creature).owner);
    }
}

#[test]
fn virtue_of_strength_returns_a_land_or_creature_and_triples_only_its_basic_tap_mana() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let virtue = candidate(
        &mut game,
        player,
        "Virtue of Strength // Garenbrig Growth",
        Zone::Hand,
    );
    let land = put(&mut game, player, "Forest", Zone::Graveyard);
    let creature = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let opt = put(&mut game, player, "Opt", Zone::Graveyard);
    let foreign = put(&mut game, opponent, "Forest", Zone::Graveyard);
    act(&mut game, Action::AnnounceAdventure { card: virtue });
    reject(&mut game, target(opt));
    reject(&mut game, target(foreign));
    let mut recover_land = game.clone();
    act(&mut recover_land, target(land));
    pay(&mut recover_land);
    settle(&mut recover_land);
    assert_eq!(recover_land.object_view(land).zone, Zone::Hand);

    act(&mut game, target(creature));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Hand);
    act(&mut game, Action::AnnounceExile { card: virtue });
    pay(&mut game);
    settle(&mut game);

    let forest = put(&mut game, player, "Forest", Zone::Battlefield);
    let island = put(&mut game, player, "Island", Zone::Battlefield);
    let dual = put(&mut game, player, "Lush Portico", Zone::Battlefield);
    let enemy_forest = put(&mut game, opponent, "Forest", Zone::Battlefield);
    game.state.players[player.index()].mana = [0; 6];
    game.state.players[opponent.index()].mana = [0; 6];
    for card in [forest, island] {
        act(&mut game, Action::TapMana { card });
    }
    assert_eq!(game.state.players[player.index()].mana, [0, 3, 0, 0, 3, 0]);
    assert!(game.object_view(dual).has_subtype("Forest"));
    act(&mut game, Action::TapMana { card: dual });
    assert_eq!(
        game.state.players[player.index()].mana.iter().sum::<u32>(),
        7
    );
    act(&mut game, Action::Pass);
    act(&mut game, Action::TapMana { card: enemy_forest });
    assert_eq!(
        game.state.players[opponent.index()].mana,
        [0, 0, 0, 0, 1, 0]
    );
}

#[test]
fn specter_of_mortality_exiles_own_creature_cards_then_triggers_a_separate_shrink() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let specter = candidate(&mut game, player, "Specter of Mortality", Zone::Hand);
    let friend = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let first = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let second = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let land = put(&mut game, player, "Forest", Zone::Graveyard);
    let foreign = put(&mut game, opponent, "Bear Cub", Zone::Graveyard);
    cast(&mut game, specter, None);
    pass_pair(&mut game);
    assert_eq!(stats(&game, specter), (3, 3));
    assert!(
        game.object_view(specter)
            .keywords
            .contains(Keywords::FLYING)
    );
    pass_pair(&mut game);

    let select = |game: &Game, card| Action::SelectResolutionExile {
        object: game.object_ref(card),
    };
    for card in [land, foreign] {
        let action = select(&game, card);
        reject(&mut game, action);
    }
    let mut decline = game.clone();
    act(&mut decline, Action::FinishEffectChoice);
    settle(&mut decline);
    assert_eq!(decline.object_view(first).zone, Zone::Graveyard);
    assert_eq!(stats(&decline, friend), (3, 3));

    let first_action = select(&game, first);
    let second_action = select(&game, second);
    act(&mut game, first_action.clone());
    reject(&mut game, first_action);
    let mut one = game.clone();
    act(&mut one, Action::FinishEffectChoice);
    settle(&mut one);
    assert_eq!(stats(&one, friend), (2, 2));
    assert_eq!(stats(&one, enemy), (1, 1));
    assert_eq!(one.object_view(second).zone, Zone::Graveyard);

    act(&mut game, second_action);
    act(&mut game, Action::FinishEffectChoice);
    assert_eq!(game.object_view(first).zone, Zone::Exile);
    assert_eq!(game.object_view(second).zone, Zone::Exile);
    assert_eq!(stats(&game, enemy), (2, 2));
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert_eq!(stats(&game, specter), (3, 3));
    assert_eq!(stats(&game, friend), (1, 1));
    assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
    pass_to_later_main(&mut game, opponent);
    assert_eq!(stats(&game, friend), (3, 3));
}

fn activate_untargeted(game: &mut Game, card: CardId, ability: u16) {
    choose(game, |action| match action {
        Action::Activate {
            card: source,
            ability: index,
            ..
        }
        | Action::AnnounceActivate {
            card: source,
            ability: index,
            ..
        } => *source == card && *index == ability,
        _ => false,
    });
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

#[test]
fn syr_ginger_reacts_to_own_other_artifacts_and_gains_its_last_power() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let ginger = candidate(&mut game, player, "Syr Ginger, the Meal Ender", Zone::Hand);
    let own = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let foreign = put(&mut game, opponent, "Candy Trail", Zone::Battlefield);
    let own_walker = put(
        &mut game,
        player,
        "Ajani, Caller of the Pride",
        Zone::Battlefield,
    );
    let first = put(&mut game, player, "Disenchant", Zone::Hand);
    let second = put(&mut game, player, "Disenchant", Zone::Hand);
    game.add_counter(own_walker, CounterKind::Loyalty, 4);
    cast(&mut game, ginger, None);
    settle(&mut game);
    let keywords = Keywords::TRAMPLE
        .union(Keywords::HEXPROOF)
        .union(Keywords::HASTE);
    let view = game.object_view(ginger);
    assert_eq!(view.kinds & 17, 17);
    assert!(view.has_subtype("Food") && view.has_subtype("Knight"));
    assert_eq!((view.power, view.toughness), (3, 1));
    for keyword in [Keywords::TRAMPLE, Keywords::HEXPROOF, Keywords::HASTE] {
        assert!(!view.keywords.contains(keyword));
    }

    let walker = put(
        &mut game,
        opponent,
        "Ajani, Caller of the Pride",
        Zone::Battlefield,
    );
    game.add_counter(walker, CounterKind::Loyalty, 4);
    assert!(game.object_view(ginger).keywords.contains(keywords));
    let mut gone = game.clone();
    let down = put(&mut gone, player, "Hero's Downfall", Zone::Hand);
    cast(&mut gone, down, Some(Target::Card(walker)));
    settle(&mut gone);
    for keyword in [Keywords::TRAMPLE, Keywords::HEXPROOF, Keywords::HASTE] {
        assert!(!gone.object_view(ginger).keywords.contains(keyword));
    }

    cast(&mut game, first, Some(Target::Card(foreign)));
    settle(&mut game);
    assert_eq!(game.counter_count(ginger, CounterKind::PlusOne), 0);
    cast(&mut game, second, Some(Target::Card(own)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Graveyard);
    assert_eq!(game.counter_count(ginger, CounterKind::PlusOne), 0);
    complete_scry_one(&mut game, player);
    settle(&mut game);
    assert_eq!(stats(&game, ginger), (4, 2));

    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    let mut negative = game.clone();
    negative.state.objects[ginger as usize].growth.power = -6;
    for (branch, gained) in [(&mut game, 4), (&mut negative, 0)] {
        activate_untargeted(branch, ginger, 0);
        assert_eq!(branch.state.players[player.index()].mana, [0; 6]);
        assert_eq!(branch.object_view(ginger).zone, Zone::Graveyard);
        assert_eq!(branch.state.players[player.index()].life, 20);
        settle(branch);
        assert_eq!(branch.state.players[player.index()].life, 20 + gained);
    }
}

#[test]
fn the_irencrag_produces_colorless_and_can_become_persistent_everflame() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let irencrag = candidate(&mut game, player, "The Irencrag", Zone::Hand);
    let bear = put(&mut game, player, "Bear Cub", Zone::Hand);
    let fynn = put(&mut game, player, "Fynn, the Fangbearer", Zone::Hand);
    let foreign = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let tap = Action::ActivateMana {
        card: irencrag,
        ability: 0,
        counter_payment: Counters::default(),
        payment: [0; 6],
        production: [0, 0, 0, 0, 0, 1],
    };
    cast(&mut game, irencrag, None);
    settle(&mut game);
    let exact = game.object_ref(irencrag);
    let mut mana = game.clone();
    let before = mana.state.players[player.index()].mana[5];
    act(&mut mana, tap.clone());
    assert!(mana.object_view(irencrag).tapped);
    assert_eq!(mana.state.players[player.index()].mana[5], before + 1);

    cast(&mut game, bear, None);
    settle(&mut game);
    cast(&mut game, fynn, None);
    pass_pair(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    pass_pair(&mut game);
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    settle(&mut decline);
    assert_eq!(decline.object_view(irencrag).name(), "The Irencrag");
    assert!(decline.legal_actions().contains(&tap));

    act(&mut game, Action::ChooseOptional { accept: true });
    let equipment = game.object_view(irencrag);
    assert_eq!(equipment.reference, exact);
    assert_eq!(equipment.name(), "Everflame, Heroes' Legacy");
    assert!(equipment.is_legendary() && equipment.has_subtype("Equipment"));
    assert_eq!(equipment.kinds, 16);
    assert_eq!(game.object_mana_value(irencrag), 2);
    reject(&mut game, tap.clone());

    let grant = game
        .observe(player)
        .self_granted_abilities
        .unwrap()
        .into_iter()
        .find(|row| row.recipient == exact)
        .unwrap();
    let equip = Action::AnnounceSelfGrantedAbility {
        source: exact,
        grant_timestamp: grant.timestamp,
        counter_payment: Counters::default(),
    };
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    reject(&mut game, equip.clone());
    game.state.players[player.index()].mana[5] = 3;
    act(&mut game, equip);
    reject(&mut game, target(foreign));
    act(&mut game, target(bear));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(
        game.object_view(irencrag).attached_to,
        Some(game.object_ref(bear))
    );
    assert_eq!(stats(&game, bear), (5, 5));

    pass_to_later_main(&mut game, player);
    assert_eq!(
        game.object_view(irencrag).name(),
        "Everflame, Heroes' Legacy"
    );
    assert_eq!(stats(&game, bear), (5, 5));
    reject(&mut game, tap);
    let troyan = put(&mut game, player, "Troyan, Gutsy Explorer", Zone::Hand);
    game.state.players[player.index()].mana = [0, 1, 0, 0, 1, 1];
    cast(&mut game, troyan, None);
    settle(&mut game);
    assert_eq!(
        game.object_view(irencrag).name(),
        "Everflame, Heroes' Legacy"
    );
}

#[test]
fn three_bowls_spends_each_mode_once_and_sacrifices_on_resolution() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let porridge = candidate(&mut game, player, "Three Bowls of Porridge", Zone::Hand);
    let creature = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let lands = [0; 2].map(|_| put(&mut game, player, "Swamp", Zone::Battlefield));
    let plains = [0; 2].map(|_| put(&mut game, opponent, "Plains", Zone::Battlefield));
    let disenchant = put(&mut game, opponent, "Disenchant", Zone::Hand);
    cast(&mut game, porridge, None);
    settle(&mut game);
    let view = game.object_view(porridge);
    assert!(view.has_subtype("Food") && view.kinds == 16);
    let announce = Action::AnnounceActivate {
        card: porridge,
        ability: 0,
        counter_payment: Counters::default(),
    };
    reject(
        &mut game,
        Action::AnnounceActivate {
            card: porridge,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    for mode in 0..3 {
        if mode != 0 {
            pass_to_later_main(&mut game, player);
            for card in lands {
                act(&mut game, Action::TapMana { card });
            }
        }
        let before: u32 = game.state.players[player.index()].mana.iter().sum();
        act(&mut game, announce.clone());
        for spent in 0..mode {
            reject(&mut game, Action::ChooseMode { mode: spent });
        }
        act(&mut game, Action::ChooseMode { mode });
        if mode < 2 {
            reject(&mut game, target(porridge));
            act(&mut game, target(creature));
        }
        pay(&mut game);
        assert!(game.object_view(porridge).tapped);
        assert_eq!(game.object_view(porridge).zone, Zone::Battlefield);
        let after: u32 = game.state.players[player.index()].mana.iter().sum();
        assert_eq!(before - after, 2);
        if mode == 2 {
            let mut gone = game.clone();
            act(&mut gone, Action::Pass);
            for card in plains {
                act(&mut gone, Action::TapMana { card });
            }
            cast(&mut gone, disenchant, Some(Target::Card(porridge)));
            settle(&mut gone);
            assert_eq!(gone.object_view(porridge).zone, Zone::Graveyard);
            assert_eq!(gone.state.players[player.index()].life, 23);
        }
        settle(&mut game);
        match mode {
            0 => assert_eq!(game.object_view(creature).damage, 2),
            1 => assert!(game.object_view(creature).tapped),
            2 => {
                assert_eq!(game.object_view(porridge).zone, Zone::Graveyard);
                assert_eq!(game.state.players[player.index()].life, 23);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn will_freezes_gross_life_gained_and_discounts_white_and_blue_only_this_turn() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let will = candidate(&mut game, player, "Will, Scion of Peace", Zone::Battlefield);
    let glutton = put(&mut game, player, "Hamlet Glutton", Zone::Hand);
    let strike = put(&mut game, opponent, "Lightning Strike", Zone::Hand);
    let guard = put(&mut game, player, "Werefox Bodyguard", Zone::Battlefield);
    assert!(
        game.object_view(will)
            .keywords
            .contains(Keywords::VIGILANCE)
    );
    act(&mut game, Action::AnnounceCast { card: glutton });
    act(&mut game, Action::ChooseBargain { sacrifice: None });
    pay(&mut game);
    reject(
        &mut game,
        Action::AnnounceActivate {
            card: will,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 23);
    act(&mut game, Action::Pass);
    cast(&mut game, strike, Some(Target::Player(player)));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 20);
    activate_untargeted(&mut game, will, 0);
    assert!(game.object_view(will).tapped);
    settle(&mut game);

    sacrifice_bodyguard(&mut game, guard, player);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 22);
    let blue = put(&mut game, player, "Virtue of Knowledge", Zone::Hand);
    let white = put(&mut game, player, "Virtue of Loyalty", Zone::Hand);
    let red = put(&mut game, player, "Virtue of Courage", Zone::Hand);
    game.state.players[player.index()].mana = [1, 1, 0, 2, 0, 0];
    reject(&mut game, Action::AnnounceCast { card: white });
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
    reject(&mut game, Action::AnnounceCast { card: blue });
    game.state.players[player.index()].mana = [0, 0, 0, 2, 0, 0];
    reject(&mut game, Action::AnnounceCast { card: red });

    game.state.players[player.index()].mana = [2, 1, 0, 0, 0, 1];
    act(&mut game, Action::AnnounceCast { card: blue });
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 1, 0, 0, 0, 1],
        },
    );
    assert_eq!(game.state.stack.last().unwrap().mana_value(), 5);
    assert_eq!(game.state.players[player.index()].mana, [2, 0, 0, 0, 0, 0]);
    settle(&mut game);
    cast(&mut game, white, None);
    assert_eq!(game.state.stack.last().unwrap().mana_value(), 5);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    settle(&mut game);

    pass_to_later_main(&mut game, player);
    let again = put(&mut game, player, "Virtue of Knowledge", Zone::Hand);
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 1];
    reject(&mut game, Action::AnnounceCast { card: again });
}

#[test]
fn yenna_requires_unique_owned_enchantments_and_continues_only_after_an_aura_copy() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let yenna = candidate(
        &mut game,
        player,
        "Yenna, Redtooth Regent",
        Zone::Battlefield,
    );
    let bearer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let aura = put(&mut game, player, "Bestial Bloodline", Zone::Hand);
    let legendary = put(&mut game, player, "The Sibsig Ceremony", Zone::Battlefield);
    let duplicate = put(&mut game, player, "Virtue of Strength", Zone::Battlefield);
    put(&mut game, player, "Virtue of Strength", Zone::Battlefield);
    let foreign = put(&mut game, opponent, "Virtue of Loyalty", Zone::Battlefield);
    let bounce = put(&mut game, opponent, "Unsummon", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(bearer)));
    settle(&mut game);
    assert_eq!(stats(&game, bearer), (4, 4));
    assert_eq!(stats(&game, yenna), (4, 4));
    assert!(game.object_view(legendary).is_legendary());

    let announce = Action::AnnounceActivate {
        card: yenna,
        ability: 0,
        counter_payment: Counters::default(),
    };
    act(&mut game, announce);
    for invalid in [yenna, duplicate, foreign] {
        reject(&mut game, target(invalid));
    }
    let mut non_aura = game.clone();
    act(&mut non_aura, target(legendary));
    pay(&mut non_aura);
    settle(&mut non_aura);
    assert!(non_aura.object_view(yenna).tapped);
    assert!(non_aura.observe(player).objects.iter().any(|object| {
        object.zone == Zone::Battlefield
            && object.name() == "The Sibsig Ceremony"
            && object.is_token()
            && !object.is_legendary()
    }));

    act(&mut game, target(aura));
    pay(&mut game);
    let mut gone = game.clone();
    act(&mut gone, Action::Pass);
    cast(&mut gone, bounce, Some(Target::Card(yenna)));
    pass_pair(&mut gone);
    for branch in [&mut game, &mut gone] {
        pass_pair(branch);
        let attached = branch.object_ref(bearer);
        act(branch, Action::ChooseAuraAttachment { object: attached });
        assert!(branch.observe(player).objects.iter().any(|object| {
            object.zone == Zone::Battlefield
                && object.name() == "Bestial Bloodline"
                && object.is_token()
                && object.attached_to == Some(attached)
        }));
        assert_eq!(stats(branch, bearer), (6, 6));
        assert_eq!(branch.state.pending, Pending::EffectLibrary { player });
        assert!(branch.observe(opponent).library_choice.is_none());
        let library = &branch.state.players[player.index()].library;
        let top = *library.last().unwrap();
        let next = library[library.len() - 2];
        for (card, destination) in [
            (top, LibraryDestination::Bottom),
            (next, LibraryDestination::Top),
        ] {
            act(branch, Action::ArrangeLibrary { card, destination });
        }
        settle(branch);
        assert_eq!(branch.top_library_card(player), Some(next));
    }
    assert!(!game.object_view(yenna).tapped);
    assert_eq!(gone.object_view(yenna).zone, Zone::Hand);
}

#[test]
fn sentinel_offers_each_legal_mode_set_and_keeps_surviving_target_groups() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let sentinel = candidate(&mut game, player, "Sentinel of Lost Lore", Zone::Hand);
    let own = put(&mut game, player, "Beanstalk Wurm", Zone::Exile);
    let foreign = put(&mut game, opponent, "Beanstalk Wurm", Zone::Exile);
    let plain = put(&mut game, player, "Bear Cub", Zone::Exile);
    let graveyard =
        ["Forest", "Bear Cub"].map(|name| put(&mut game, opponent, name, Zone::Graveyard));
    let own_grave = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    cast(&mut game, sentinel, None);
    pass_pair(&mut game);
    assert_eq!(stats(&game, sentinel), (3, 4));
    let mut available: Vec<_> = game
        .legal_actions()
        .into_iter()
        .filter_map(|action| match action {
            Action::ChooseMode { mode } => Some(mode),
            _ => None,
        })
        .collect();
    available.sort_unstable();
    assert_eq!(available, (0..7).collect::<Vec<_>>());

    for (mode, (recover, bottom, clear)) in [
        (true, false, false),
        (false, true, false),
        (false, false, true),
        (true, true, false),
        (true, false, true),
        (false, true, true),
        (true, true, true),
    ]
    .into_iter()
    .enumerate()
    {
        let mut branch = game.clone();
        act(&mut branch, Action::ChooseMode { mode: mode as u16 });
        if recover {
            reject(&mut branch, target(plain));
            reject(&mut branch, target(foreign));
            act(&mut branch, target(own));
        }
        if bottom {
            reject(&mut branch, target(own));
            act(&mut branch, target(foreign));
        }
        if clear {
            act(
                &mut branch,
                Action::ChooseTarget {
                    target: Target::Player(opponent),
                },
            );
        }

        if mode == 6 {
            let mut stale = branch.clone();
            stale.move_object(own, Zone::Graveyard);
            settle(&mut stale);
            assert_eq!(stale.object_view(own).zone, Zone::Graveyard);
            assert_eq!(stale.object_view(foreign).zone, Zone::Library);
            assert_eq!(stale.object_view(graveyard[0]).zone, Zone::Exile);
        }

        settle(&mut branch);
        assert_eq!(
            branch.object_view(own).zone,
            if recover { Zone::Hand } else { Zone::Exile }
        );
        assert_eq!(
            branch.object_view(foreign).zone,
            if bottom { Zone::Library } else { Zone::Exile }
        );
        if bottom {
            assert_eq!(
                branch.state.players[opponent.index()].library.first(),
                Some(&foreign)
            );
        }
        for card in graveyard {
            assert_eq!(
                branch.object_view(card).zone,
                if clear { Zone::Exile } else { Zone::Graveyard }
            );
        }
        assert_eq!(branch.object_view(own_grave).zone, Zone::Graveyard);
        assert_eq!(branch.object_view(plain).zone, Zone::Exile);
    }
}

#[test]
fn apprentices_folly_makes_nonlegendary_hasty_reflections_and_sacrifices_controlled_reflections() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let saga = candidate(&mut game, player, "The Apprentice's Folly", Zone::Hand);
    let legendary = put(&mut game, player, "Fynn, the Fangbearer", Zone::Battlefield);
    let original = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let shapeshifter = put(&mut game, player, "Changeling Wayfinder", Zone::Battlefield);
    let other_shapeshifter = put(
        &mut game,
        opponent,
        "Changeling Wayfinder",
        Zone::Battlefield,
    );
    game.add_counter(legendary, CounterKind::PlusOne, 1);
    cast(&mut game, saga, None);
    pass_pair(&mut game);
    reject(&mut game, target(other_shapeshifter));
    act(&mut game, target(legendary));
    settle(&mut game);

    let reflection = game
        .observe(player)
        .objects
        .into_iter()
        .find(|object| {
            object.zone == Zone::Battlefield
                && object.is_token()
                && object.controller == player
                && object.name() == "Fynn, the Fangbearer"
        })
        .unwrap();
    assert!(reflection.has_subtype("Reflection") && reflection.has_subtype("Human"));
    assert!(
        reflection
            .keywords
            .contains(Keywords::HASTE.union(Keywords::DEATHTOUCH))
    );
    assert!(!reflection.is_legendary());
    assert_eq!(stats(&game, reflection.id), (1, 3));
    assert_eq!(game.counter_count(reflection.id, CounterKind::PlusOne), 0);
    assert!(game.object_view(legendary).is_legendary());

    pass_to_later_main(&mut game, player);
    reject(&mut game, target(legendary));
    reject(&mut game, target(reflection.id));
    act(&mut game, target(original));
    settle(&mut game);
    let bear_copy = game
        .observe(player)
        .objects
        .into_iter()
        .find(|object| {
            object.zone == Zone::Battlefield
                && object.is_token()
                && object.controller == player
                && object.name() == "Bear Cub"
        })
        .unwrap();
    assert!(bear_copy.has_subtype("Reflection") && bear_copy.has_subtype("Bear"));
    assert!(bear_copy.keywords.contains(Keywords::HASTE));

    pass_to_later_main(&mut game, player);
    settle(&mut game);
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);
    for card in [reflection.id, bear_copy.id, shapeshifter] {
        assert_ne!(game.object_view(card).zone, Zone::Battlefield);
    }
    for card in [legendary, original, other_shapeshifter] {
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
    }
}

#[test]
fn the_princess_returns_both_linked_exiles_to_their_owners_after_its_flying_chapter() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let saga = candidate(&mut game, player, "The Princess Takes Flight", Zone::Hand);
    let visions = put(&mut game, player, "Virtue of Knowledge", Zone::Hand);
    let stolen = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let exile_second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let protected = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foreign = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let disenchant = put(&mut game, opponent, "Disenchant", Zone::Hand);
    let plains = [0; 2].map(|_| put(&mut game, opponent, "Plains", Zone::Battlefield));
    game.change_controller(game.object_ref(stolen), player);
    cast(&mut game, saga, None);
    pass_pair(&mut game);
    let mut no_exile = game.clone();
    act(&mut no_exile, Action::FinishTargets);
    settle(&mut no_exile);
    assert_eq!(no_exile.object_view(stolen).zone, Zone::Battlefield);
    assert_eq!(no_exile.object_view(exile_second).zone, Zone::Battlefield);

    act(&mut game, target(stolen));
    let first = game.state.stack.last().unwrap().id;
    cast_adventure(&mut game, visions, Some(Target::Stack(first)));
    pass_pair(&mut game);
    act(
        &mut game,
        Action::SetStackTarget {
            index: 0,
            target: Target::Card(exile_second),
        },
    );
    act(&mut game, Action::FinishStackTargets);
    settle(&mut game);
    assert_eq!(game.object_view(stolen).zone, Zone::Exile);
    assert_eq!(game.object_view(exile_second).zone, Zone::Exile);

    pass_to_later_main(&mut game, player);
    reject(&mut game, target(foreign));
    act(&mut game, target(protected));
    settle(&mut game);
    assert_eq!(stats(&game, protected), (4, 4));
    assert!(
        game.object_view(protected)
            .keywords
            .contains(Keywords::FLYING)
    );
    pass_to_later_main(&mut game, player);
    assert_eq!(stats(&game, protected), (2, 2));
    assert!(
        !game
            .object_view(protected)
            .keywords
            .contains(Keywords::FLYING)
    );
    assert_eq!(game.state.stack.len(), 1);

    let mut destroyed = game.clone();
    act(&mut destroyed, Action::Pass);
    for card in plains {
        act(&mut destroyed, Action::TapMana { card });
    }
    cast(&mut destroyed, disenchant, Some(Target::Card(saga)));
    for branch in [&mut game, &mut destroyed] {
        settle(branch);
        assert_eq!(branch.object_view(saga).zone, Zone::Graveyard);
        assert_eq!(branch.object_view(stolen).zone, Zone::Battlefield);
        assert_eq!(branch.object_view(stolen).controller, opponent);
        assert_eq!(branch.object_view(exile_second).zone, Zone::Battlefield);
        assert_eq!(branch.object_view(exile_second).controller, player);
    }
}

#[test]
fn three_blind_mice_creates_a_mouse_copies_controlled_tokens_and_temporarily_boosts_creatures() {
    use rustmage_core::engine::replacements::TokenGroupView;

    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let saga = candidate(&mut game, player, "Three Blind Mice", Zone::Hand);
    let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let food = Card::from_catalogue_id(5246);
    let own_food = spawn(&mut game, player, food, Zone::Battlefield);
    let their_food = spawn(&mut game, opponent, food, Zone::Battlefield);
    let first_ref = game.object_ref(first);
    let role = Card::from_catalogue_id(5248);
    let _ = game.create_token_groups_attached(
        player,
        &[TokenGroupView {
            token: role,
            amount: 1,
            tapped: false,
        }],
        None,
        true,
        Some(first_ref),
    );
    let own_role = game
        .observe(player)
        .objects
        .into_iter()
        .find(|object| object.zone == Zone::Battlefield && object.card == role)
        .unwrap()
        .id;
    cast(&mut game, saga, None);
    settle(&mut game);
    let mouse = game
        .observe(player)
        .objects
        .into_iter()
        .find(|object| {
            object.zone == Zone::Battlefield
                && object.is_token()
                && object.controller == player
                && object.has_subtype("Mouse")
        })
        .unwrap();
    assert_eq!((mouse.power, mouse.toughness, mouse.colors), (1, 1, 1));

    pass_to_later_main(&mut game, player);
    reject(&mut game, target(their_food));
    reject(&mut game, target(first));
    act(&mut game, target(own_food));
    settle(&mut game);
    assert_eq!(
        game.observe(player)
            .objects
            .iter()
            .filter(|object| {
                object.zone == Zone::Battlefield
                    && object.controller == player
                    && object.card == food
            })
            .count(),
        2
    );

    pass_to_later_main(&mut game, player);
    act(&mut game, target(own_role));
    pass_pair(&mut game);
    let second_ref = game.object_ref(second);
    act(
        &mut game,
        Action::ChooseAuraAttachment { object: second_ref },
    );
    settle(&mut game);
    assert!(game.observe(player).objects.iter().any(|object| {
        object.zone == Zone::Battlefield
            && object.controller == player
            && object.card == role
            && object.id != own_role
            && object.attached_to == Some(second_ref)
    }));
    assert_eq!(stats(&game, first), (3, 3));
    assert_eq!(stats(&game, second), (3, 3));

    pass_to_later_main(&mut game, player);
    settle(&mut game);
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);
    for (card, expected) in [(first, (4, 4)), (second, (4, 4)), (mouse.id, (2, 2))] {
        assert_eq!(stats(&game, card), expected);
        assert!(
            game.object_view(card)
                .keywords
                .contains(Keywords::VIGILANCE)
        );
    }
    assert_eq!(stats(&game, enemy), (2, 2));
    assert!(
        !game
            .object_view(enemy)
            .keywords
            .contains(Keywords::VIGILANCE)
    );
    pass_to_later_main(&mut game, opponent);
    assert_eq!(stats(&game, mouse.id), (1, 1));
    assert!(
        !game
            .object_view(mouse.id)
            .keywords
            .contains(Keywords::VIGILANCE)
    );
}

#[test]
fn thunderous_debut_privately_looks_twenty_and_reveals_up_to_two_before_moving_and_shuffling() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let debut = candidate(&mut game, player, "Thunderous Debut", Zone::Hand);
    let candy = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let outside = put(&mut game, player, "Bear Cub", Zone::Library);
    for _ in 0..16 {
        put(&mut game, player, "Forest", Zone::Library);
    }
    let spare = put(&mut game, player, "Bear Cub", Zone::Library);
    let invalid = put(&mut game, player, "Opt", Zone::Library);
    let angel = put(&mut game, player, "Serra Angel", Zone::Library);
    let giada = put(&mut game, player, "Giada, Font of Hope", Zone::Library);
    let original = game.state.players[player.index()].library.clone();

    for bargained in [false, true] {
        let mut branch = game.clone();
        act(&mut branch, Action::AnnounceCast { card: debut });
        let sacrifice = bargained.then(|| branch.object_ref(candy));
        act(&mut branch, Action::ChooseBargain { sacrifice });
        act(
            &mut branch,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 2, 6],
            },
        );
        assert_eq!(
            branch.object_view(candy).zone,
            if bargained {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        pass_pair(&mut branch);
        let view = branch.observe(player).library_selection.unwrap();
        assert_eq!((view.minimum, view.maximum), (0, 2));
        assert_eq!(view.remaining.len(), 20);
        assert!(view.reveal && !view.public);
        assert!(branch.observe(opponent).library_selection.is_none());
        assert_eq!(
            view.candidates
                .iter()
                .map(|object| object.card)
                .collect::<Vec<_>>(),
            [giada, angel, spare]
        );
        for card in [outside, invalid] {
            reject(&mut branch, Action::SelectLibrary { card });
        }

        let mut none = branch.clone();
        act(&mut none, Action::FinishLibrarySelection);
        settle(&mut none);
        let shuffled = &none.state.players[player.index()].library;
        assert_ne!(shuffled, &original);
        let mut same_cards = shuffled.clone();
        same_cards.sort_unstable();
        assert_eq!(same_cards, original);

        act(&mut branch, Action::SelectLibrary { card: giada });
        act(&mut branch, Action::SelectLibrary { card: angel });
        reject(&mut branch, Action::SelectLibrary { card: spare });
        act(&mut branch, Action::FinishLibrarySelection);
        settle(&mut branch);
        let zone = if bargained {
            Zone::Battlefield
        } else {
            Zone::Hand
        };
        for card in [giada, angel] {
            assert_eq!(branch.object_view(card).zone, zone);
        }
        assert_eq!(branch.object_view(spare).zone, Zone::Library);
        assert_eq!(branch.object_view(outside).zone, Zone::Library);
        let expected: Vec<_> = original
            .iter()
            .copied()
            .filter(|card| *card != giada && *card != angel)
            .collect();
        assert_ne!(
            &branch.state.players[player.index()].library[..],
            expected.as_slice()
        );
        if bargained {
            assert_eq!(stats(&branch, angel), (4, 4));
            assert_eq!(branch.counter_count(angel, CounterKind::PlusOne), 0);
        }
    }
}
