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
const HUMAN: Card = Card::from_catalogue_id(5254);

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

fn enter_main_phase(game: &mut Game) {
    for _ in 0..8 {
        if game.state.step == Step::Main1 {
            return;
        }
        act(game, Action::Pass);
    }
    panic!("precombat main phase did not begin");
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
        enter_main_phase(&mut game);
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

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn enter_end_step(game: &mut Game) {
    game.begin_step(Step::Main2);
    pass_pair(game);
    assert_eq!(game.state.step, Step::End);
}

#[test]
fn malevolent_witchkite_sacrifices_any_subset_and_draws_before_death_triggers() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let kite = spawn(&mut game, player, named("Malevolent Witchkite"), Zone::Hand);
        let artifact = put(&mut game, player, "Gingerbrute", Zone::Battlefield);
        let enchantment = put(&mut game, player, "Demonic Pact", Zone::Battlefield);
        let token = spawn(&mut game, player, RAT, Zone::Battlefield);
        let unchosen = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let ordinary = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let foreign = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
        let observer = put(&mut game, player, "Midnight Reaper", Zone::Battlefield);
        let library = game.state.players[player.index()].library.len();

        act(&mut game, Action::AnnounceCast { card: kite });
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 2, 0, 0, 4],
            },
        );
        pass_pair(&mut game);
        let creature = game.object_view(kite);
        assert_eq!((creature.power, creature.toughness), (5, 4));
        assert!(creature.keywords.contains(Keywords::FLYING));
        pass_pair(&mut game);
        assert_eq!(game.state.pending, Pending::EffectChoice { player });

        let mut zero = game.clone();
        act(&mut zero, Action::FinishEffectChoice);
        assert_eq!(zero.state.players[player.index()].library.len(), library);
        assert!(zero.state.stack.is_empty());
        for card in [artifact, enchantment, token] {
            assert_eq!(zero.object_view(card).zone, Zone::Battlefield);
        }

        let sacrifice = |game: &Game, card| Action::Sacrifice {
            object: game.object_ref(card),
        };
        for card in [ordinary, foreign, kite] {
            let choice = sacrifice(&game, card);
            reject(&mut game, choice);
        }
        for card in [artifact, enchantment, token] {
            let choice = sacrifice(&game, card);
            act(&mut game, choice);
            assert_eq!(game.object_view(card).zone, Zone::Battlefield);
        }
        assert_eq!(game.state.players[player.index()].library.len(), library);
        act(&mut game, Action::FinishEffectChoice);
        for card in [artifact, enchantment] {
            assert_eq!(game.object_view(card).zone, Zone::Graveyard);
        }
        assert_ne!(game.object_view(token).zone, Zone::Battlefield);
        for card in [ordinary, foreign, unchosen] {
            assert_eq!(game.object_view(card).zone, Zone::Battlefield);
        }
        assert_eq!(
            game.state.players[player.index()].library.len(),
            library - 3
        );
        assert_eq!(game.state.stack.len(), 1);
        assert_eq!(
            game.state.stack.last().unwrap().source.unwrap().id,
            observer
        );
        settle(&mut game);
        assert_eq!(
            game.state.players[player.index()].library.len(),
            library - 4
        );
    }
}

#[test]
fn ogre_chitterlord_counts_rat_permanents_after_creation_on_entry_and_attack() {
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        let ogre = spawn(&mut game, player, named("Ogre Chitterlord"), Zone::Hand);
        let core = put(&mut game, player, "Firdoch Core", Zone::Battlefield);
        let rat = put(&mut game, player, "Burglar Rat", Zone::Battlefield);
        spawn(&mut game, player, RAT, Zone::Battlefield);
        let foreign_rat = spawn(&mut game, other, RAT, Zone::Battlefield);
        let bystander = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let blockers = [0; 2].map(|_| put(&mut game, other, "Bear Cub", Zone::Battlefield));
        let core_view = game.object_view(core);
        assert!(core_view.has_subtype("Rat") && !core_view.is_creature());

        let mut below = game.clone();
        below.move_object(core, Zone::Exile);
        cast(&mut below, ogre, None);
        settle(&mut below);
        assert_eq!(stats(&below, rat), (1, 1));
        let below_rats = tokens(&below, player, RAT);
        assert_eq!(below_rats.len(), 3);
        assert!(below_rats.iter().all(|&id| stats(&below, id) == (1, 1)));

        act(&mut game, Action::AnnounceCast { card: ogre });
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 0, 2, 0, 4],
            },
        );
        settle(&mut game);
        assert_eq!(stats(&game, ogre), (6, 5));
        assert!(game.object_view(ogre).keywords.contains(Keywords::MENACE));
        assert_eq!(stats(&game, rat), (3, 1));
        assert_eq!(stats(&game, foreign_rat), (1, 1));
        assert_eq!(stats(&game, bystander), (2, 2));
        let entered_rats = tokens(&game, player, RAT);
        assert_eq!(entered_rats.len(), 3);
        for &id in &entered_rats {
            let view = game.object_view(id);
            assert_eq!((view.power, view.toughness, view.colors), (3, 1, 4));
            assert!(view.has_subtype("Rat") && view.keywords.contains(Keywords::CANT_BLOCK));
        }
        act(&mut game, activate(core, 1));
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 0, 4],
            },
        );
        settle(&mut game);
        assert_eq!(stats(&game, core), (6, 4));

        finish_turn(&mut game);
        assert_eq!(stats(&game, rat), (1, 1));
        assert!(entered_rats.iter().all(|&id| stats(&game, id) == (1, 1)));
        finish_turn(&mut game);
        assert_eq!(game.state.active, player);
        attack(&mut game, ogre);
        assert_eq!(game.state.stack.len(), 1);
        settle(&mut game);
        let attacking_rats = tokens(&game, player, RAT);
        assert_eq!(attacking_rats.len(), 5);
        assert!(attacking_rats.iter().all(|&id| stats(&game, id) == (3, 1)));

        pass_pair(&mut game);
        reject(
            &mut game,
            Action::Block {
                blocker: foreign_rat,
                attacker: ogre,
            },
        );
        act(
            &mut game,
            Action::Block {
                blocker: blockers[0],
                attacker: ogre,
            },
        );
        reject(&mut game, Action::FinishBlockers);
        act(
            &mut game,
            Action::Block {
                blocker: blockers[1],
                attacker: ogre,
            },
        );
        act(&mut game, Action::FinishBlockers);
    }
}

#[test]
fn old_flitterfang_creates_food_on_either_end_step_only_if_a_creature_already_died() {
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        let flitterfang = spawn(&mut game, player, named("Old Flitterfang"), Zone::Hand);
        let first_shock = put(&mut game, player, "Shock", Zone::Hand);
        let second_shock = put(&mut game, other, "Shock", Zone::Hand);
        let mountain = put(&mut game, other, "Mountain", Zone::Battlefield);
        let first_victim = put(&mut game, other, "Bear Cub", Zone::Battlefield);
        let second_victim = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        cast(&mut game, first_shock, Some(Target::Card(first_victim)));
        settle(&mut game);
        assert_eq!(game.object_view(first_victim).zone, Zone::Graveyard);
        cast(&mut game, flitterfang, None);
        settle(&mut game);
        let creature = game.object_view(flitterfang);
        assert_eq!((creature.power, creature.toughness), (3, 4));
        assert!(creature.is_legendary() && creature.keywords.contains(Keywords::FLYING));

        enter_end_step(&mut game);
        assert_eq!(game.state.stack.len(), 1);
        settle(&mut game);
        assert_eq!(tokens(&game, player, FOOD).len(), 1);
        pass_pair(&mut game);
        assert_eq!(game.state.active, other);

        let mut late = game.clone();
        enter_end_step(&mut late);
        assert!(late.state.stack.is_empty());
        act(&mut late, Action::TapMana { card: mountain });
        cast(&mut late, second_shock, Some(Target::Card(second_victim)));
        settle(&mut late);
        assert_eq!(late.object_view(second_victim).zone, Zone::Graveyard);
        assert_eq!(tokens(&late, player, FOOD).len(), 1);

        act(&mut game, Action::TapMana { card: mountain });
        cast(&mut game, second_shock, Some(Target::Card(second_victim)));
        settle(&mut game);
        enter_end_step(&mut game);
        assert_eq!(game.state.stack.len(), 1);
        settle(&mut game);
        assert_eq!(tokens(&game, player, FOOD).len(), 2);
        assert!(tokens(&game, other, FOOD).is_empty());
    }
}

#[test]
fn old_flitterfang_can_pay_to_sacrifice_another_artifact_or_creature_twice() {
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(other);
        let flitterfang = spawn(
            &mut game,
            player,
            named("Old Flitterfang"),
            Zone::Battlefield,
        );
        let artifact = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let enchantment = put(&mut game, player, "Demonic Pact", Zone::Battlefield);
        let foreign = put(&mut game, other, "Bear Cub", Zone::Battlefield);
        let choose_cost = |game: &Game, card| Action::ChooseAdditionalCost {
            object: game.object_ref(card),
        };
        for (sacrifice, expected) in [(artifact, (5, 6)), (creature, (7, 8))] {
            act(&mut game, Action::Pass);
            act(&mut game, activate(flitterfang, 0));
            for card in [flitterfang, enchantment, foreign] {
                let choice = choose_cost(&game, card);
                reject(&mut game, choice);
            }
            let choice = choose_cost(&game, sacrifice);
            act(&mut game, choice);
            act(
                &mut game,
                Action::PayCosts {
                    payment: [0, 0, 1, 0, 0, 2],
                },
            );
            assert_eq!(game.object_view(sacrifice).zone, Zone::Graveyard);
            settle(&mut game);
            assert_eq!(stats(&game, flitterfang), expected);
        }

        enter_end_step(&mut game);
        settle(&mut game);
        pass_pair(&mut game);
        assert_eq!(game.state.active, player);
        assert_eq!(stats(&game, flitterfang), (3, 4));
    }
}

#[test]
fn night_of_the_sweets_revenge_funds_its_paid_boost_then_samples_remaining_foods() {
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        let night = spawn(
            &mut game,
            player,
            named("Night of the Sweets' Revenge"),
            Zone::Hand,
        );
        let cookie = put(&mut game, player, "Gingerbrute", Zone::Battlefield);
        let artifact = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let foreign = put(&mut game, other, "Bear Cub", Zone::Battlefield);
        let foreign_food = spawn(&mut game, other, FOOD, Zone::Battlefield);
        let disenchant = put(&mut game, other, "Disenchant", Zone::Hand);
        let forest = put(&mut game, player, "Forest", Zone::Hand);
        let late = put(&mut game, player, "Llanowar Elves", Zone::Hand);
        cast(&mut game, night, None);
        settle(&mut game);
        let foods = tokens(&game, player, FOOD);
        assert_eq!(foods.len(), 1);
        let food = foods[0];
        let grantor = game.object_ref(night);
        let tap = |card| Action::TapGrantedMana {
            card,
            grantor,
            production: [0, 0, 0, 0, 1, 0],
        };
        for card in [artifact, ally, foreign_food] {
            reject(&mut game, tap(card));
        }
        assert!(game.legal_actions().contains(&tap(cookie)));

        let announce = activate(night, 0);
        let mut combat = game.clone();
        combat.begin_step(Step::BeginCombat);
        reject(&mut combat, announce.clone());
        game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 5];
        act(&mut game, announce);
        act(&mut game, tap(food));
        assert!(game.object_view(food).tapped);
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 2, 5],
            },
        );
        assert_eq!(game.object_view(night).zone, Zone::Graveyard);
        reject(&mut game, tap(cookie));

        let mut both_foods = game.clone();
        settle(&mut both_foods);
        assert_eq!(stats(&both_foods, ally), (4, 4));
        assert_eq!(stats(&both_foods, cookie), (3, 3));

        act(&mut game, Action::Pass);
        cast(&mut game, disenchant, Some(Target::Card(food)));
        settle(&mut game);
        assert!(tokens(&game, player, FOOD).is_empty());
        assert_eq!(stats(&game, ally), (3, 3));
        assert_eq!(stats(&game, cookie), (2, 2));
        assert_eq!(stats(&game, foreign), (2, 2));

        act(&mut game, Action::PlayLand { card: forest });
        act(&mut game, Action::TapMana { card: forest });
        cast(&mut game, late, None);
        settle(&mut game);
        assert_eq!(stats(&game, late), (1, 1));
        finish_turn(&mut game);
        assert_eq!(stats(&game, ally), (2, 2));
        assert_eq!(stats(&game, cookie), (1, 1));
    }
}

#[test]
fn realm_scorcher_hellkite_bargains_for_four_colored_mana_and_has_paid_damage_and_haste() {
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut setup = scenario(player);
        let hellkite = spawn(
            &mut setup,
            player,
            named("Realm-Scorcher Hellkite"),
            Zone::Hand,
        );
        let artifact = put(&mut setup, player, "Short Bow", Zone::Battlefield);
        let enchantment = put(&mut setup, player, "Demonic Pact", Zone::Battlefield);
        let token = spawn(&mut setup, player, RAT, Zone::Battlefield);
        let ordinary = put(&mut setup, player, "Bear Cub", Zone::Battlefield);
        let victim = put(&mut setup, other, "Llanowar Elves", Zone::Battlefield);
        let counterspell = put(&mut setup, other, "Essence Scatter", Zone::Hand);
        let exact_payment = [0, 0, 0, 2, 0, 4];
        setup.state.players[player.index()].mana = exact_payment;

        for offering in [None, Some(artifact), Some(enchantment), Some(token)] {
            let mut game = setup.clone();
            act(&mut game, Action::AnnounceCast { card: hellkite });
            let invalid = Some(game.object_ref(ordinary));
            reject(&mut game, Action::ChooseBargain { sacrifice: invalid });
            let sacrifice = offering.map(|id| game.object_ref(id));
            act(&mut game, Action::ChooseBargain { sacrifice });
            act(
                &mut game,
                Action::PayCosts {
                    payment: exact_payment,
                },
            );
            assert_eq!(
                game.state.stack.last().unwrap().cast.unwrap().bargain,
                Some(offering.is_some())
            );
            assert_eq!(game.state.players[player.index()].mana, [0; 6]);
            if let Some(card) = offering {
                assert_ne!(game.object_view(card).zone, Zone::Battlefield);
            }

            if offering == Some(artifact) {
                let mut countered = game.clone();
                act(&mut countered, Action::Pass);
                cast(&mut countered, counterspell, Some(Target::Spell(hellkite)));
                settle(&mut countered);
                assert_eq!(countered.object_view(hellkite).zone, Zone::Graveyard);
                assert_eq!(countered.state.players[player.index()].mana, [0; 6]);
            }

            pass_pair(&mut game);
            let view = game.object_view(hellkite);
            assert_eq!(view.zone, Zone::Battlefield);
            assert_eq!((view.power, view.toughness), (4, 6));
            assert!(
                view.keywords
                    .contains(Keywords::FLYING.union(Keywords::HASTE))
            );
            if offering.is_none() {
                assert!(game.state.stack.is_empty());
                assert_eq!(game.state.players[player.index()].mana, [0; 6]);
                continue;
            }

            pass_pair(&mut game);
            reject(
                &mut game,
                Action::ChooseMana {
                    production: [0, 0, 0, 0, 0, 4],
                },
            );
            reject(
                &mut game,
                Action::ChooseMana {
                    production: [0, 0, 0, 3, 0, 0],
                },
            );
            let production = [1, 0, 0, 3, 0, 0];
            act(&mut game, Action::ChooseMana { production });
            assert_eq!(game.state.players[player.index()].mana, production);

            if offering == Some(artifact) {
                for (target, payment) in [
                    (Target::Card(victim), [1, 0, 0, 1, 0, 0]),
                    (Target::Player(other), [0, 0, 0, 2, 0, 0]),
                ] {
                    act(&mut game, activate(hellkite, 0));
                    act(&mut game, Action::ChooseTarget { target });
                    act(&mut game, Action::PayCosts { payment });
                    settle(&mut game);
                }
                assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
                assert_eq!(game.state.players[other.index()].life, 19);
                assert_eq!(game.state.players[player.index()].mana, [0; 6]);
                attack(&mut game, hellkite);
                assert_eq!(game.state.attackers[0].card, hellkite);
            }
        }
    }
}

#[test]
fn return_from_the_wilds_resolves_each_pair_and_shuffles_even_without_a_land() {
    for player in [Player::P0, Player::P1] {
        let mut parent = scenario(player);
        let spell = spawn(
            &mut parent,
            player,
            named("Return from the Wilds"),
            Zone::Hand,
        );
        let basic = put(&mut parent, player, "Island", Zone::Library);
        let nonbasic = put(&mut parent, player, "Evolving Wilds", Zone::Library);
        act(&mut parent, Action::AnnounceCast { card: spell });
        assert_eq!(
            parent
                .legal_actions()
                .iter()
                .filter(|action| matches!(action, Action::ChooseMode { .. }))
                .count(),
            3
        );
        reject(&mut parent, Action::ChooseMode { mode: 3 });

        for (mode, search, human, food) in [
            (0, true, true, false),
            (1, true, false, true),
            (2, false, true, true),
        ] {
            let mut game = parent.clone();
            act(&mut game, Action::ChooseMode { mode });
            act(
                &mut game,
                Action::PayCosts {
                    payment: [0, 0, 0, 0, 1, 2],
                },
            );
            pass_pair(&mut game);
            if search {
                assert_eq!(game.state.pending, Pending::EffectSearch { player });
                assert!(game.observe(player.other()).search_choice.is_none());
                assert!(tokens(&game, player, HUMAN).is_empty());
                assert!(tokens(&game, player, FOOD).is_empty());
                reject(&mut game, Action::ChooseSearch { card: nonbasic });
                if mode == 0 {
                    let mut miss = game.clone();
                    let rng = miss.state.rng;
                    act(&mut miss, Action::FinishSearch);
                    settle(&mut miss);
                    assert_ne!(miss.state.rng, rng);
                    assert_eq!(miss.object_view(basic).zone, Zone::Library);
                    assert_eq!(tokens(&miss, player, HUMAN).len(), 1);
                }
                act(&mut game, Action::ChooseSearch { card: basic });
                act(&mut game, Action::FinishSearch);
                settle(&mut game);
                assert_eq!(game.object_view(basic).zone, Zone::Battlefield);
                assert!(game.object_view(basic).tapped);
            } else {
                assert_eq!(game.object_view(basic).zone, Zone::Library);
                assert_eq!(
                    game.state.players[player.index()].nonland_permanents_entered_this_turn,
                    2
                );
            }
            let humans = tokens(&game, player, HUMAN);
            assert_eq!(humans.len(), usize::from(human));
            if let Some(&id) = humans.first() {
                let view = game.object_view(id);
                assert_eq!((view.power, view.toughness, view.colors), (1, 1, 1));
                assert!(view.has_subtype("Human"));
            }
            assert_eq!(tokens(&game, player, FOOD).len(), usize::from(food));
            assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn rowans_grim_search_privately_keeps_zero_one_or_two_and_draws_in_the_selected_order() {
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let decline_miracle = |game: &mut Game, card, can_reveal| {
            let view = game.observe(player).miracle.unwrap();
            assert_eq!(view.object, game.object_ref(card));
            assert_eq!(view.can_reveal, can_reveal);
            assert!(game.observe(other).miracle.is_none());
            assert_eq!(
                game.legal_actions().contains(&Action::RevealMiracle),
                can_reveal
            );
            act(game, Action::DeclineMiracle);
            settle(game);
        };
        let mut game = scenario(player);
        let spell = spawn(&mut game, player, named("Rowan's Grim Search"), Zone::Hand);
        let offering = spawn(&mut game, player, FOOD, Zone::Battlefield);
        put(
            &mut game,
            player,
            "Lorehold, the Historian",
            Zone::Battlefield,
        );
        let [fifth, fourth, island, shock, top] =
            ["Forest", "Swamp", "Island", "Shock", "Mountain"]
                .map(|card| put(&mut game, player, card, Zone::Library));
        act(&mut game, Action::AnnounceCast { card: spell });

        let mut unpaid = game.clone();
        act(&mut unpaid, Action::ChooseBargain { sacrifice: None });
        pay(&mut unpaid);
        pass_pair(&mut unpaid);
        decline_miracle(&mut unpaid, top, false);
        assert_eq!(unpaid.object_view(offering).zone, Zone::Battlefield);
        assert_eq!(unpaid.object_view(top).zone, Zone::Hand);
        assert_eq!(unpaid.object_view(shock).zone, Zone::Hand);
        assert_eq!(unpaid.top_library_card(player), Some(island));
        assert_eq!(unpaid.state.players[player.index()].life, 18);

        let sacrifice = Some(game.object_ref(offering));
        act(&mut game, Action::ChooseBargain { sacrifice });
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 1, 0, 0, 2],
            },
        );
        pass_pair(&mut game);
        assert_ne!(game.object_view(offering).zone, Zone::Battlefield);
        let view = game.observe(player).library_selection.unwrap();
        assert_eq!((view.minimum, view.maximum), (0, 2));
        assert_eq!(
            view.remaining
                .iter()
                .map(|object| object.id)
                .collect::<Vec<_>>(),
            [top, shock, island, fourth]
        );
        assert!(game.observe(other).library_selection.is_none());
        reject(&mut game, Action::SelectLibrary { card: fifth });

        let mut zero = game.clone();
        act(&mut zero, Action::FinishLibrarySelection);
        decline_miracle(&mut zero, fifth, false);
        for card in [top, shock, island, fourth] {
            assert_eq!(zero.object_view(card).zone, Zone::Graveyard);
        }
        assert_eq!(zero.object_view(fifth).zone, Zone::Hand);
        assert_eq!(zero.state.players[player.index()].cards_drawn_this_turn, 2);
        assert_eq!(zero.state.players[player.index()].life, 18);

        let mut one = game.clone();
        act(&mut one, Action::SelectLibrary { card: island });
        act(&mut one, Action::FinishLibrarySelection);
        decline_miracle(&mut one, island, false);
        for card in [top, shock, fourth] {
            assert_eq!(one.object_view(card).zone, Zone::Graveyard);
        }
        assert_eq!(one.object_view(island).zone, Zone::Hand);
        assert_eq!(one.object_view(fifth).zone, Zone::Hand);
        assert_eq!(one.state.players[player.index()].life, 18);

        let mut reverse = game.clone();
        act(&mut reverse, Action::SelectLibrary { card: island });
        act(&mut reverse, Action::SelectLibrary { card: shock });
        act(&mut reverse, Action::FinishLibrarySelection);
        decline_miracle(&mut reverse, island, false);
        assert!(reverse.observe(player).miracle.is_none());
        assert_eq!(reverse.object_view(shock).zone, Zone::Hand);

        let first = Action::SelectLibrary { card: shock };
        let saved = game.snapshot();
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == first)
            .unwrap();
        let mut indexed = game.clone();
        indexed.apply_index(player, revision, index).unwrap();
        act(&mut game, first.clone());
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        assert!(indexed.observe(other).library_selection.is_none());
        act(&mut indexed, first);
        assert_eq!(game.snapshot(), indexed.snapshot());
        act(&mut game, Action::SelectLibrary { card: island });
        reject(&mut game, Action::SelectLibrary { card: top });
        act(&mut game, Action::FinishLibrarySelection);
        assert_eq!(game.object_view(island).zone, Zone::Library);
        decline_miracle(&mut game, shock, true);
        assert_eq!(game.object_view(shock).zone, Zone::Hand);
        assert_eq!(game.object_view(island).zone, Zone::Hand);
        assert_eq!(game.object_view(top).zone, Zone::Graveyard);
        assert_eq!(game.object_view(fourth).zone, Zone::Graveyard);
        assert_eq!(game.top_library_card(player), Some(fifth));
        assert_eq!(game.state.players[player.index()].life, 18);
    }
}

#[test]
fn questing_druid_counts_each_qualifying_cast_once_and_ignores_spell_copies() {
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        let druid = spawn(&mut game, player, named("Questing Druid"), Zone::Hand);
        cast(&mut game, druid, None);
        settle(&mut game);
        assert_eq!(stats(&game, druid), (1, 1));
        for (name, target, count) in [
            ("Llanowar Elves", None, 0),
            ("Short Bow", None, 0),
            ("Healer's Hawk", None, 1),
            ("Vampire Gourmand", None, 2),
            ("Shock", Some(Target::Player(other)), 3),
            ("Lightning Helix", Some(Target::Player(other)), 4),
            ("Teach by Example", None, 5),
        ] {
            let spell = put(&mut game, player, name, Zone::Hand);
            cast(&mut game, spell, target);
            settle(&mut game);
            assert_eq!(
                game.counter_count(druid, CounterKind::PlusOne),
                count,
                "{name}"
            );
        }

        let divination = put(&mut game, player, "Divination", Zone::Hand);
        let library = game.state.players[player.index()].library.len();
        cast(&mut game, divination, None);
        for _ in 0..2 {
            if matches!(game.state.pending, Pending::OrderTriggers { .. }) {
                choose(&mut game, |action| {
                    matches!(action, Action::PutTrigger { .. })
                });
            }
        }
        settle(&mut game);
        assert_eq!(
            game.state.players[player.index()].library.len(),
            library - 4
        );
        assert_eq!(game.counter_count(druid, CounterKind::PlusOne), 6);

        let opposing_shock = put(&mut game, other, "Shock", Zone::Hand);
        act(&mut game, Action::Pass);
        cast(&mut game, opposing_shock, Some(Target::Player(player)));
        settle(&mut game);
        assert_eq!(game.counter_count(druid, CounterKind::PlusOne), 6);
    }
}

#[test]
fn seek_the_beast_exiles_two_paid_plays_until_the_next_own_end_step() {
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        let druid_card = named("Questing Druid");
        let druid = spawn(&mut game, player, druid_card, Zone::Hand);
        let adventure = spawn(&mut game, player, druid_card, Zone::Hand);
        let mountain = put(&mut game, player, "Mountain", Zone::Battlefield);
        let forest = put(&mut game, player, "Forest", Zone::Battlefield);
        let untouched = put(&mut game, player, "Swamp", Zone::Library);
        let land = put(&mut game, player, "Island", Zone::Library);
        let shock = put(&mut game, player, "Shock", Zone::Library);
        cast(&mut game, druid, None);
        settle(&mut game);

        enter_end_step(&mut game);
        for card in [mountain, forest] {
            act(&mut game, Action::TapMana { card });
        }
        act(&mut game, Action::AnnounceAdventure { card: adventure });
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 0, 1, 1, 0],
            },
        );
        assert!(game.state.stack.iter().any(|item| {
            item.cast
                .is_some_and(|cast| cast.face == CastFace::Adventure)
                && item.source.unwrap().card.name() == "Seek the Beast"
        }));
        settle(&mut game);
        assert_eq!(game.counter_count(druid, CounterKind::PlusOne), 1);
        assert_eq!(game.object_view(adventure).zone, Zone::Exile);
        assert_eq!(game.top_library_card(player), Some(untouched));
        let observed = game.observe(other);
        for card in [land, shock] {
            assert_eq!(game.object_view(card).zone, Zone::Exile);
            let exact = game.object_ref(card);
            assert!(observed.exile_permissions.iter().any(|row| {
                row.object == exact
                    && row.player == player
                    && row.expiry == ExilePlayExpiry::StartOfNextEndStep(player)
            }));
        }
        reject(&mut game, Action::PlayLand { card: land });

        pass_pair(&mut game);
        assert_eq!(game.state.active, other);
        finish_turn(&mut game);
        assert_eq!(game.state.active, player);
        enter_main_phase(&mut game);
        let mut expired = game.clone();
        enter_end_step(&mut expired);
        assert_eq!(expired.object_view(land).zone, Zone::Exile);
        assert_eq!(expired.object_view(shock).zone, Zone::Exile);
        assert!(
            !expired
                .observe(player)
                .exile_permissions
                .iter()
                .any(|row| [land, shock].contains(&row.object.card))
        );
        act(&mut expired, Action::TapMana { card: mountain });
        reject(&mut expired, Action::AnnounceExile { card: shock });

        act(&mut game, Action::PlayLand { card: land });
        assert_eq!(game.object_view(land).zone, Zone::Battlefield);
        act(&mut game, Action::TapMana { card: mountain });
        act(&mut game, Action::AnnounceExile { card: shock });
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(other),
            },
        );
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 0, 1, 0, 0],
            },
        );
        settle(&mut game);
        assert_eq!(game.state.players[other.index()].life, 18);
        assert_eq!(game.counter_count(druid, CounterKind::PlusOne), 2);

        for card in [forest, land] {
            act(&mut game, Action::TapMana { card });
        }
        act(&mut game, Action::AnnounceExile { card: adventure });
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 1, 0, 0, 1, 0],
            },
        );
        settle(&mut game);
        assert_eq!(game.object_view(adventure).zone, Zone::Battlefield);
        assert_eq!(stats(&game, adventure), (1, 1));
    }
}

fn deal_unblocked_combat_damage(game: &mut Game) {
    pass_pair(game);
    act(game, Action::FinishBlockers);
    pass_pair(game);
    assert_eq!(game.state.step, Step::CombatDamage);
}

#[test]
fn rotisserie_elemental_can_keep_skewers_then_sacrifice_for_exact_temporary_plays() {
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        let elemental = spawn(&mut game, player, named("Rotisserie Elemental"), Zone::Hand);
        let opposing_shock = put(&mut game, other, "Shock", Zone::Hand);
        let opposing_mountain = put(&mut game, other, "Mountain", Zone::Battlefield);
        let untouched = put(&mut game, player, "Swamp", Zone::Library);
        let shock = put(&mut game, player, "Shock", Zone::Library);
        let land = put(&mut game, player, "Mountain", Zone::Library);
        cast(&mut game, elemental, None);
        settle(&mut game);
        assert_eq!(stats(&game, elemental), (1, 1));
        assert!(
            game.object_view(elemental)
                .keywords
                .contains(Keywords::MENACE)
        );
        finish_turn(&mut game);
        finish_turn(&mut game);

        let mut attacks_walker = game.clone();
        let walker = put(
            &mut attacks_walker,
            other,
            "Kaito, Cunning Infiltrator",
            Zone::Battlefield,
        );
        attacks_walker.add_counter(walker, CounterKind::Loyalty, 3);
        let defender = attacks_walker.object_ref(walker);
        attacks_walker.begin_step(Step::BeginCombat);
        pass_pair(&mut attacks_walker);
        act(
            &mut attacks_walker,
            Action::AttackPlaneswalker {
                card: elemental,
                defender,
            },
        );
        act(&mut attacks_walker, Action::FinishAttackers);
        deal_unblocked_combat_damage(&mut attacks_walker);
        assert_eq!(
            attacks_walker.counter_count(walker, CounterKind::Loyalty),
            2
        );
        assert!(attacks_walker.state.stack.is_empty());
        assert_eq!(
            attacks_walker.counter_count(elemental, CounterKind::Skewer),
            0
        );

        attack(&mut game, elemental);
        deal_unblocked_combat_damage(&mut game);
        assert_eq!(game.state.stack.len(), 1);
        let mut departed = game.clone();
        act(&mut departed, Action::Pass);
        act(
            &mut departed,
            Action::TapMana {
                card: opposing_mountain,
            },
        );
        cast(&mut departed, opposing_shock, Some(Target::Card(elemental)));
        settle(&mut departed);
        assert_eq!(departed.object_view(elemental).zone, Zone::Graveyard);
        assert_eq!(departed.top_library_card(player), Some(land));

        pass_pair(&mut game);
        assert_eq!(game.counter_count(elemental, CounterKind::Skewer), 1);
        act(&mut game, Action::DeclineEffectPayment);
        assert_eq!(game.object_view(elemental).zone, Zone::Battlefield);
        assert_eq!(game.top_library_card(player), Some(land));
        game.add_counter(elemental, CounterKind::PlusOne, 1);
        pass_pair(&mut game);
        pass_pair(&mut game);
        finish_turn(&mut game);
        finish_turn(&mut game);

        attack(&mut game, elemental);
        deal_unblocked_combat_damage(&mut game);
        pass_pair(&mut game);
        assert_eq!(game.counter_count(elemental, CounterKind::Skewer), 2);
        let object = game.object_ref(elemental);
        act(&mut game, Action::Sacrifice { object });
        settle(&mut game);
        assert_eq!(game.object_view(elemental).zone, Zone::Graveyard);
        assert_eq!(game.object_view(land).zone, Zone::Exile);
        assert_eq!(game.object_view(shock).zone, Zone::Exile);
        assert_eq!(game.top_library_card(player), Some(untouched));
        reject(&mut game, Action::PlayLand { card: land });

        pass_pair(&mut game);
        pass_pair(&mut game);
        let mut expired = game.clone();
        finish_turn(&mut expired);
        assert!(expired.observe(player).exile_permissions.is_empty());
        assert_eq!(expired.object_view(shock).zone, Zone::Exile);
        act(&mut game, Action::PlayLand { card: land });
        act(&mut game, Action::TapMana { card: land });
        act(&mut game, Action::AnnounceExile { card: shock });
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(other),
            },
        );
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.state.players[other.index()].life, 15);
        assert_eq!(game.object_view(shock).zone, Zone::Graveyard);
    }
}
