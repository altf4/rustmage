//! Paid gameplay scenarios for Wilds of Eldraine cards from M through R.
mod support;

use rustmage as _;
use rustmage_core::engine::replacements::TokenGroupView;
use rustmage_core::{cards::*, engine::*};
use support::*;

const RAT: Card = Card::from_catalogue_id(5323);
const MONSTER: Card = Card::from_catalogue_id(5330);
const YOUNG_HERO: Card = Card::from_catalogue_id(5275);
const FOOD: Card = Card::from_catalogue_id(5246);

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn tokens(game: &Game, controller: Player, recipe: Card) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield
                && object.controller == controller
                && object.is_token()
                && object.card == recipe)
                .then_some(id as CardId)
        })
        .collect()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn attack(game: &mut Game, attacker: CardId) {
    game.begin_step(Step::BeginCombat);
    pass_pair(game);
    act(game, Action::Attack { card: attacker });
    act(game, Action::FinishAttackers);
}

fn finish_turn(game: &mut Game) {
    let turn = game.state.turn;
    game.begin_step(Step::End);
    for _ in 0..12 {
        if game.state.turn != turn {
            return;
        }
        assert_eq!(game.state.pending, Pending::Priority);
        act(game, Action::Pass);
    }
    panic!("turn did not reach cleanup");
}

#[test]
fn rat_out_zero_target_resolution_and_stale_target_are_distinct() {
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(other);
        let attacker = put(&mut game, other, "Hill Giant", Zone::Battlefield);
        let victim = put(&mut game, other, "Llanowar Elves", Zone::Battlefield);
        let spell = spawn(&mut game, player, named("Rat Out"), Zone::Hand);
        let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
        act(&mut game, Action::Pass);
        act(&mut game, Action::AnnounceCast { card: spell });

        let checkpoint = game.snapshot();
        let finish_index = game
            .legal_actions()
            .iter()
            .position(|action| *action == Action::FinishTargets)
            .unwrap();
        let mut declined = game.clone();
        let mut indexed = game.clone();
        act(&mut declined, Action::FinishTargets);
        indexed
            .apply_index(player, game.revision(), finish_index)
            .unwrap();
        assert_eq!(declined.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(indexed.snapshot(), checkpoint);
        pay(&mut declined);
        settle(&mut declined);
        let rats = tokens(&declined, player, RAT);
        assert_eq!(rats.len(), 1);
        let rat = rats[0];
        let rat_view = declined.object_view(rat);
        assert_eq!((rat_view.power, rat_view.toughness), (1, 1));
        assert_eq!(rat_view.colors, 4);
        assert!(rat_view.has_subtype("Rat"));
        assert!(rat_view.keywords.contains(Keywords::CANT_BLOCK));
        assert_eq!(stats(&declined, victim), (1, 1));
        attack(&mut declined, attacker);
        pass_pair(&mut declined);
        assert_eq!(declined.state.pending, Pending::Blockers);
        reject(
            &mut declined,
            Action::Block {
                blocker: rat,
                attacker,
            },
        );

        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        pay(&mut game);
        let mut stale = game.clone();
        cast(&mut stale, bounce, Some(Target::Card(victim)));
        settle(&mut stale);
        assert_eq!(stale.object_view(victim).zone, Zone::Hand);
        assert!(tokens(&stale, player, RAT).is_empty());
        settle(&mut game);
        assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
        assert_eq!(tokens(&game, player, RAT).len(), 1);
    }
}

#[test]
fn monstrous_rage_replaces_only_its_controllers_role_and_expires_the_pump() {
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        let target = put(&mut game, other, "Bear Cub", Zone::Battlefield);
        let recipient = game.object_ref(target);
        let spell = spawn(&mut game, player, named("Monstrous Rage"), Zone::Hand);
        let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
        let group = [TokenGroupView {
            token: YOUNG_HERO,
            amount: 1,
            tapped: false,
        }];
        let prior =
            game.create_token_groups_attached(player, &group, None, true, Some(recipient))[0];
        let foreign =
            game.create_token_groups_attached(other, &group, None, true, Some(recipient))[0];

        cast(&mut game, spell, Some(Target::Card(target)));
        let mut stale = game.clone();
        cast(&mut stale, bounce, Some(Target::Card(target)));
        settle(&mut stale);
        assert!(tokens(&stale, player, MONSTER).is_empty());

        settle(&mut game);
        let monsters = tokens(&game, player, MONSTER);
        assert_eq!(monsters.len(), 1);
        assert_eq!(game.object_view(monsters[0]).attached_to, Some(recipient));
        assert_ne!(game.object_view(prior.card).zone, Zone::Battlefield);
        assert_eq!(game.object_view(foreign.card).attached_to, Some(recipient));
        assert_eq!(stats(&game, target), (5, 3));
        assert!(
            game.object_view(target)
                .keywords
                .contains(Keywords::TRAMPLE)
        );
        finish_turn(&mut game);
        assert_eq!(stats(&game, target), (3, 3));
        assert!(
            game.object_view(target)
                .keywords
                .contains(Keywords::TRAMPLE)
        );
    }
}

#[test]
fn monstrous_rage_remains_illegal_in_standard_decks() {
    let rage = named("Monstrous Rage");
    let mut deck = vec![Card::Forest; 60];
    deck[0] = rage;
    let result = Game::new(Config {
        decks: [deck, vec![Card::Forest; 60]],
        seed: 7,
        starting_player: Player::P0,
        deck_rules: DeckRules::Standard,
    });
    assert!(matches!(
        result,
        Err(Error::InvalidDeck(reason)) if reason.contains("not Standard legal")
            && reason.contains("Monstrous Rage")
    ));
}

#[test]
fn moonshaker_cavalry_counts_and_grants_at_resolution() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let survivor = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let witness = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let opposing = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let cavalry = spawn(&mut game, player, named("Moonshaker Cavalry"), Zone::Hand);
        let first_bounce = put(&mut game, player, "Unsummon", Zone::Hand);
        let second_bounce = put(&mut game, player, "Unsummon", Zone::Hand);
        let late = put(&mut game, player, "Bear Cub", Zone::Hand);

        act(&mut game, Action::AnnounceCast { card: cavalry });
        act(
            &mut game,
            Action::PayCosts {
                payment: [3, 0, 0, 0, 0, 5],
            },
        );
        pass_pair(&mut game);
        assert_eq!(game.object_view(cavalry).zone, Zone::Battlefield);
        assert_eq!(stats(&game, cavalry), (6, 6));
        assert!(
            game.object_view(cavalry)
                .keywords
                .contains(Keywords::FLYING)
        );
        assert!(
            !game
                .object_view(survivor)
                .keywords
                .contains(Keywords::FLYING)
        );
        assert_eq!(game.state.stack.len(), 1);

        let mut all_present = game.clone();
        settle(&mut all_present);
        assert_eq!(stats(&all_present, cavalry), (9, 9));
        assert_eq!(stats(&all_present, witness), (5, 5));

        cast(&mut game, first_bounce, Some(Target::Card(witness)));
        settle(&mut game);
        assert_eq!(game.object_view(witness).zone, Zone::Hand);
        assert_eq!(stats(&game, cavalry), (8, 8));
        assert_eq!(stats(&game, survivor), (4, 4));
        assert!(
            game.object_view(survivor)
                .keywords
                .contains(Keywords::FLYING)
        );
        assert_eq!(stats(&game, opposing), (2, 2));
        assert!(
            !game
                .object_view(opposing)
                .keywords
                .contains(Keywords::FLYING)
        );

        cast(&mut game, late, None);
        settle(&mut game);
        assert_eq!(stats(&game, late), (2, 2));
        assert!(!game.object_view(late).keywords.contains(Keywords::FLYING));
        cast(&mut game, second_bounce, Some(Target::Card(cavalry)));
        settle(&mut game);
        assert_eq!(stats(&game, survivor), (4, 4));
        assert!(
            game.object_view(survivor)
                .keywords
                .contains(Keywords::FLYING)
        );
        finish_turn(&mut game);
        assert_eq!(stats(&game, survivor), (2, 2));
        assert!(
            !game
                .object_view(survivor)
                .keywords
                .contains(Keywords::FLYING)
        );
    }
}

#[test]
fn hare_raising_locks_the_creature_count_and_the_hare_buffs_only_own_creature_tokens() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let hare = spawn(&mut game, player, named("Pollen-Shield Hare"), Zone::Hand);
        let target = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let rat = spawn(&mut game, player, RAT, Zone::Battlefield);
        let opposing_rat = spawn(&mut game, player.other(), RAT, Zone::Battlefield);
        let food = spawn(&mut game, player, FOOD, Zone::Battlefield);
        let bounce = put(&mut game, player, "Unsummon", Zone::Hand);

        act(&mut game, Action::AnnounceAdventure { card: hare });
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(opposing_rat),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 1, 0],
            },
        );
        let spell = game.state.stack.last().unwrap();
        assert_eq!(spell.cast.unwrap().face, CastFace::Adventure);
        assert_eq!(spell.source.unwrap().card.name(), "Hare Raising");
        settle(&mut game);
        assert_eq!(game.object_view(hare).zone, Zone::Exile);
        assert_eq!(stats(&game, target), (4, 4));
        assert!(
            game.object_view(target)
                .keywords
                .contains(Keywords::VIGILANCE)
        );
        assert_eq!(stats(&game, rat), (1, 1));

        act(&mut game, Action::AnnounceExile { card: hare });
        act(
            &mut game,
            Action::PayCosts {
                payment: [1, 0, 0, 0, 0, 1],
            },
        );
        settle(&mut game);
        assert_eq!(game.object_view(hare).zone, Zone::Battlefield);
        assert_eq!(stats(&game, hare), (2, 2));
        assert_eq!(stats(&game, target), (4, 4));
        assert_eq!(stats(&game, rat), (2, 2));
        assert_eq!(stats(&game, opposing_rat), (1, 1));
        assert_eq!(stats(&game, food), (0, 0));

        let mut departed = game.clone();
        cast(&mut departed, bounce, Some(Target::Card(hare)));
        settle(&mut departed);
        assert_eq!(departed.object_view(hare).zone, Zone::Hand);
        assert_eq!(stats(&departed, rat), (1, 1));
        assert_eq!(stats(&departed, target), (4, 4));

        finish_turn(&mut game);
        assert_eq!(stats(&game, target), (2, 2));
        assert!(
            !game
                .object_view(target)
                .keywords
                .contains(Keywords::VIGILANCE)
        );
        assert_eq!(stats(&game, rat), (2, 2));
    }
}

#[test]
fn stolen_goodies_can_choose_zero_and_preserves_the_paid_shares_of_legal_targets() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let picnic = spawn(&mut game, player, named("Picnic Ruiner"), Zone::Hand);
        let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let foreign = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
        let target = |card| Action::ChooseTarget {
            target: Target::Card(card),
        };
        let green_payment = Action::PayCosts {
            payment: [0, 0, 0, 0, 1, 3],
        };

        act(&mut game, Action::AnnounceAdventure { card: picnic });
        reject(&mut game, target(foreign));
        let mut zero = game.clone();
        act(&mut zero, Action::FinishTargets);
        act(&mut zero, green_payment.clone());
        let adventure = zero.state.stack.last().unwrap();
        assert!(adventure.targets.is_empty());
        assert_eq!(adventure.cast.unwrap().face, CastFace::Adventure);
        assert_eq!(adventure.source.unwrap().card.name(), "Stolen Goodies");
        settle(&mut zero);
        assert_eq!(zero.object_view(picnic).zone, Zone::Exile);
        assert_eq!(zero.counter_count(first, CounterKind::PlusOne), 0);
        assert_eq!(zero.counter_count(second, CounterKind::PlusOne), 0);
        act(&mut zero, Action::AnnounceExile { card: picnic });
        act(
            &mut zero,
            Action::PayCosts {
                payment: [0, 0, 0, 1, 0, 1],
            },
        );
        settle(&mut zero);
        let creature = zero.object_view(picnic);
        assert_eq!(creature.zone, Zone::Battlefield);
        assert_eq!((creature.power, creature.toughness), (2, 2));
        assert!(creature.has_subtype("Goblin") && creature.has_subtype("Rogue"));

        for recipient in [first, second] {
            act(&mut game, target(recipient));
        }
        act(&mut game, Action::FinishTargets);
        reject(&mut game, Action::AllocateTargetAmount { amount: 0 });
        act(&mut game, Action::AllocateTargetAmount { amount: 1 });
        act(&mut game, Action::AllocateTargetAmount { amount: 2 });
        act(&mut game, green_payment);

        let mut both_legal = game.clone();
        settle(&mut both_legal);
        assert_eq!(both_legal.counter_count(first, CounterKind::PlusOne), 1);
        assert_eq!(both_legal.counter_count(second, CounterKind::PlusOne), 2);

        cast(&mut game, bounce, Some(Target::Card(first)));
        settle(&mut game);
        assert_eq!(game.object_view(first).zone, Zone::Hand);
        assert_eq!(game.object_view(picnic).zone, Zone::Exile);
        assert_eq!(game.counter_count(second, CounterKind::PlusOne), 2);
        assert_eq!(game.counter_count(foreign, CounterKind::PlusOne), 0);
    }
}

#[test]
fn picnic_ruiner_checks_its_controllers_power_at_attack_and_hits_twice_that_turn() {
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        let picnic = spawn(&mut game, player, named("Picnic Ruiner"), Zone::Battlefield);
        let witness = put(&mut game, player, "Axebane Ferox", Zone::Battlefield);
        put(&mut game, other, "Axebane Ferox", Zone::Battlefield);
        let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
        let island = put(&mut game, player, "Island", Zone::Battlefield);

        let mut unqualified = game.clone();
        unqualified.move_object(witness, Zone::Hand);
        attack(&mut unqualified, picnic);
        assert!(unqualified.state.stack.is_empty());
        assert!(
            !unqualified
                .object_view(picnic)
                .keywords
                .contains(Keywords::DOUBLE_STRIKE)
        );

        attack(&mut game, picnic);
        assert_eq!(game.state.stack.len(), 1);
        act(&mut game, Action::TapMana { card: island });
        cast(&mut game, bounce, Some(Target::Card(witness)));
        settle(&mut game);
        assert_eq!(game.object_view(witness).zone, Zone::Hand);
        assert!(
            game.object_view(picnic)
                .keywords
                .contains(Keywords::DOUBLE_STRIKE)
        );

        let before = game.state.players[other.index()].life;
        pass_pair(&mut game);
        act(&mut game, Action::FinishBlockers);
        pass_pair(&mut game);
        assert_eq!(game.state.step, Step::FirstStrikeDamage);
        assert_eq!(game.state.players[other.index()].life, before - 2);
        pass_pair(&mut game);
        assert_eq!(game.state.step, Step::CombatDamage);
        assert_eq!(game.state.players[other.index()].life, before - 4);
        finish_turn(&mut game);
        assert!(
            !game
                .object_view(picnic)
                .keywords
                .contains(Keywords::DOUBLE_STRIKE)
        );
    }
}

#[test]
fn mosswood_dreadknight_dies_into_a_paid_next_turn_adventure_and_returns_from_exile() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let dreadknight = spawn(&mut game, player, named("Mosswood Dreadknight"), Zone::Hand);
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        let lands = ["Swamp", "Swamp", "Forest", "Forest"]
            .map(|land| put(&mut game, player, land, Zone::Battlefield));
        let creature_payment = Action::PayCosts {
            payment: [0, 0, 0, 0, 2, 0],
        };
        let adventure = Action::AnnounceAdventure { card: dreadknight };

        act(&mut game, Action::AnnounceCast { card: dreadknight });
        act(&mut game, creature_payment.clone());
        settle(&mut game);
        assert_eq!(stats(&game, dreadknight), (3, 2));
        assert!(
            game.object_view(dreadknight)
                .keywords
                .contains(Keywords::TRAMPLE)
        );
        cast(&mut game, shock, Some(Target::Card(dreadknight)));
        pass_pair(&mut game);
        assert_eq!(game.object_view(dreadknight).zone, Zone::Graveyard);
        assert!(game.observe(player).graveyard_cast_permissions.is_empty());
        settle(&mut game);
        let object = game.object_ref(dreadknight);
        let permissions = game.observe(player).graveyard_cast_permissions;
        assert_eq!(permissions.len(), 1);
        assert_eq!(permissions[0].object, object);
        assert_eq!(permissions[0].kind, GraveyardCastKind::Adventure);
        assert!(game.legal_actions().contains(&adventure));
        reject(&mut game, Action::AnnounceCast { card: dreadknight });

        finish_turn(&mut game);
        assert_eq!(game.state.active, player.other());
        assert_eq!(game.observe(player).graveyard_cast_permissions, permissions);
        finish_turn(&mut game);
        assert_eq!(game.state.active, player);
        for _ in 0..8 {
            if game.state.step == Step::Main1 {
                break;
            }
            act(&mut game, Action::Pass);
        }
        assert_eq!(game.state.step, Step::Main1);
        for card in lands {
            act(&mut game, Action::TapMana { card });
        }

        let mut expired = game.clone();
        finish_turn(&mut expired);
        assert!(
            expired
                .observe(player)
                .graveyard_cast_permissions
                .is_empty()
        );

        act(&mut game, adventure.clone());
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 2, 0, 0, 0],
            },
        );
        let spell = game.state.stack.last().unwrap();
        assert_eq!(spell.source.unwrap().card.name(), "Dread Whispers");
        assert_eq!(spell.cast.unwrap().origin, Zone::Graveyard);
        let player_state = &game.state.players[player.index()];
        let life = player_state.life;
        let drawn = *player_state.library.last().unwrap();
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, life - 1);
        assert_eq!(game.object_view(drawn).zone, Zone::Hand);
        assert!(
            !game
                .observe(player.other())
                .known_hand
                .contains(&game.object_ref(drawn))
        );
        assert_eq!(game.object_view(dreadknight).zone, Zone::Exile);
        reject(&mut game, adventure);
        act(&mut game, Action::AnnounceExile { card: dreadknight });
        act(&mut game, creature_payment);
        settle(&mut game);
        assert_eq!(game.object_view(dreadknight).zone, Zone::Battlefield);
    }
}

#[test]
fn redtooth_vanguard_offers_two_mana_for_its_exact_graveyard_card_on_own_enchantment_entry() {
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        let card = named("Redtooth Vanguard");
        let vanguard = spawn(&mut game, player, card, Zone::Graveyard);
        let battlefield = spawn(&mut game, player, card, Zone::Battlefield);
        let other_creature = put(&mut game, other, "Bear Cub", Zone::Battlefield);
        let bow = put(&mut game, player, "Short Bow", Zone::Hand);
        let opposing_role = put(&mut game, other, "Royal Treatment", Zone::Hand);
        let role = put(&mut game, player, "Royal Treatment", Zone::Hand);

        cast(&mut game, bow, None);
        settle(&mut game);
        act(&mut game, Action::Pass);
        cast(&mut game, opposing_role, Some(Target::Card(other_creature)));
        settle(&mut game);
        assert_eq!(game.object_view(vanguard).zone, Zone::Graveyard);

        cast(&mut game, role, Some(Target::Card(battlefield)));
        pass_pair(&mut game);
        assert_eq!(game.state.stack.len(), 1);
        let exact = game.object_ref(vanguard);
        assert_eq!(
            game.state.stack.last().unwrap().source.unwrap().object,
            exact
        );

        let payment = Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 2],
        };
        let mut stale = game.clone();
        stale.move_object(vanguard, Zone::Exile);
        stale.move_object(vanguard, Zone::Graveyard);
        assert_ne!(stale.object_ref(vanguard), exact);
        pass_pair(&mut stale);
        act(&mut stale, payment.clone());
        assert_eq!(stale.object_view(vanguard).zone, Zone::Graveyard);

        pass_pair(&mut game);
        assert_eq!(game.state.pending, Pending::ResolutionMana { player });
        let mut declined = game.clone();
        act(&mut declined, Action::DeclineResolutionMana);
        assert_eq!(declined.object_view(vanguard).zone, Zone::Graveyard);
        reject(
            &mut game,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 0, 1],
            },
        );
        let mana = game.state.players[player.index()].mana[5];
        act(&mut game, payment);
        assert_eq!(game.state.players[player.index()].mana[5], mana - 2);
        assert_eq!(game.object_view(vanguard).zone, Zone::Hand);
        assert!(
            game.observe(other)
                .known_hand
                .contains(&game.object_ref(vanguard))
        );

        act(&mut game, Action::AnnounceCast { card: vanguard });
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 1, 1],
            },
        );
        settle(&mut game);
        assert_eq!(game.object_view(vanguard).zone, Zone::Battlefield);
        assert_eq!(stats(&game, vanguard), (3, 1));
        assert!(
            game.object_view(vanguard)
                .keywords
                .contains(Keywords::TRAMPLE)
        );
    }
}
