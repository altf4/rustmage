//! Rules scenarios for the authored Final Fantasy definitions in rows 235–312.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn trigger_target(game: &mut Game, target: Target) {
    let action = Action::ChooseTarget { target };
    for _ in 0..16 {
        if game.legal_actions().contains(&action) {
            act(game, action);
            if game.legal_actions().contains(&Action::FinishTargets) {
                act(game, Action::FinishTargets);
            }
            return;
        }
        choose(game, |choice| {
            matches!(choice, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("missing trigger target {target:?}");
}

#[test]
fn swallowed_by_leviathan_surveils_then_prices_the_exact_spell() {
    let caster = Player::P1;
    let payer = caster.other();
    let mut game = scenario(payer);
    let victim = put(&mut game, payer, "Opt", Zone::Hand);
    let counter = put(&mut game, caster, "Swallowed by Leviathan", Zone::Hand);
    let buried = put(&mut game, caster, "Island", Zone::Library);
    let kept = put(&mut game, caster, "Forest", Zone::Library);
    put(&mut game, caster, "Grizzly Bears", Zone::Graveyard);
    cast(&mut game, victim, None);
    let id = game.state.stack.last().unwrap().id;
    act(&mut game, Action::Pass);
    cast(&mut game, counter, Some(Target::Spell(victim)));
    pass_pair(&mut game);
    assert!(game.observe(caster).library_choice.is_some());
    assert!(game.observe(payer).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: buried,
            destination: LibraryDestination::Graveyard,
        },
    );
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: kept,
            destination: LibraryDestination::Top,
        },
    );
    let request = game.observe(payer).resolution_payment.unwrap();
    assert_eq!(request.cost.generic, 2);
    assert_eq!(request.victim, Some(id));
    assert_eq!(request.payer, payer);
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineResolutionMana);
    assert_eq!(decline.object_view(victim).zone, Zone::Graveyard);
    choose(&mut game, |choice| {
        matches!(choice, Action::PayResolutionMana { .. })
    });
    assert_eq!(game.object_view(victim).zone, Zone::Stack);
    assert_eq!(game.object_view(counter).zone, Zone::Graveyard);
}

#[test]
fn tifa_doubles_only_signed_power_from_her_own_landfall() {
    let player = Player::P0;
    let mut game = scenario(player);
    let tifa = put(&mut game, player, "Tifa Lockhart", Zone::Hand);
    cast(&mut game, tifa, None);
    settle(&mut game);
    assert!(game.keywords(tifa).contains(Keywords::TRAMPLE));
    assert_eq!(stats(&game, tifa), (1, 2));
    game.state.objects[tifa as usize].growth.power = -3;
    let their_land = put(&mut game, player.other(), "Forest", Zone::Hand);
    game.move_object(their_land, Zone::Battlefield);
    assert!(game.state.pending_triggers.is_empty());
    let land = put(&mut game, player, "Forest", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    settle(&mut game);
    assert_eq!(stats(&game, tifa), (-4, 2));
    game.finish_cleanup();
    assert_eq!(stats(&game, tifa), (1, 2));
}

#[test]
fn tonberry_enters_stunned_and_has_the_knife_on_its_controllers_turn() {
    let player = Player::P0;
    let mut game = scenario(player);
    let tonberry = put(&mut game, player, "Tonberry", Zone::Hand);
    cast(&mut game, tonberry, None);
    settle(&mut game);
    assert_eq!(stats(&game, tonberry), (2, 1));
    assert!(game.object_view(tonberry).tapped);
    assert_eq!(game.counter_count(tonberry, CounterKind::Stun), 1);
    let knife = Keywords::FIRST_STRIKE.union(Keywords::DEATHTOUCH);
    assert!(game.keywords(tonberry).contains(knife));
    game.state.active = player.other();
    assert!(!game.keywords(tonberry).contains(Keywords::FIRST_STRIKE));
    assert!(!game.keywords(tonberry).contains(Keywords::DEATHTOUCH));
    game.untap_objects(&[tonberry], untap_rules::UntapCause::Effect);
    assert_eq!(game.counter_count(tonberry, CounterKind::Stun), 0);
    assert!(game.object_view(tonberry).tapped);
    game.untap_objects(&[tonberry], untap_rules::UntapCause::Effect);
    assert!(!game.object_view(tonberry).tapped);
}

#[test]
fn town_greeter_offers_only_milled_lands_and_rewards_an_actual_town_return() {
    let player = Player::P1;
    let mut game = scenario(player);
    let old = put(
        &mut game,
        player,
        "Vector, Imperial Capital",
        Zone::Graveyard,
    );
    let filler = put(&mut game, player, "Shock", Zone::Library);
    let forest = put(&mut game, player, "Forest", Zone::Library);
    let town = put(&mut game, player, "Vector, Imperial Capital", Zone::Library);
    put(&mut game, player, "Swiftfoot Boots", Zone::Library);
    let greeter = put(&mut game, player, "Town Greeter", Zone::Hand);
    cast(&mut game, greeter, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.state.pending, Pending::MilledSelection { player });
    assert_eq!(game.object_view(filler).zone, Zone::Graveyard);
    let invalid = game.object_ref(old);
    reject(&mut game, Action::SelectMilled { object: invalid });
    let mut basic = game.clone();
    let object = basic.object_ref(forest);
    act(&mut basic, Action::SelectMilled { object });
    assert_eq!(basic.object_view(forest).zone, Zone::Hand);
    assert_eq!(basic.state.players[player.index()].life, 20);
    let object = game.object_ref(town);
    act(&mut game, Action::SelectMilled { object });
    assert_eq!(game.object_view(town).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].life, 22);
}

#[test]
fn the_dire_rat_creates_treasure_after_lethal_damage() {
    let player = Player::P0;
    let mut game = scenario(player);
    let rat = put(&mut game, player, "Undercity Dire Rat", Zone::Hand);
    cast(&mut game, rat, None);
    settle(&mut game);
    assert_eq!(stats(&game, rat), (2, 2));
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(rat)));
    settle(&mut game);
    assert_eq!(game.object_view(rat).zone, Zone::Graveyard);
    assert_eq!(
        game.state
            .objects
            .iter()
            .filter(|object| {
                object.zone == Zone::Battlefield
                    && object.controller == player
                    && object.card == Card::from_catalogue_id(5302)
            })
            .count(),
        1
    );
}

#[test]
fn vaynes_treachery_checks_the_kicker_sacrifice_and_uses_the_alternate_amount() {
    let player = Player::P0;
    for kicked in [false, true] {
        let mut game = scenario(player);
        let target = put(
            &mut game,
            player.other(),
            "Ghalta, Primal Hunger",
            Zone::Battlefield,
        );
        let artifact = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let land = put(&mut game, player, "Forest", Zone::Battlefield);
        let spell = put(&mut game, player, "Vayne's Treachery", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::ChooseKicker { pay: kicked });
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if kicked {
            let invalid = game.object_ref(land);
            reject(&mut game, Action::ChooseAdditionalCost { object: invalid });
            let object = game.object_ref(artifact);
            act(&mut game, Action::ChooseAdditionalCost { object });
        }
        pay(&mut game);
        settle(&mut game);
        assert_eq!(stats(&game, target), if kicked { (6, 6) } else { (10, 10) });
        assert_eq!(
            game.object_view(artifact).zone,
            if kicked {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
    }
}

#[test]
fn yshtola_returns_to_the_owner_and_adds_only_the_first_extra_end_step() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Y'shtola Rhul", Zone::Battlefield);
    let stolen = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[stolen as usize].controller = player;
    let exact = game.object_ref(stolen);
    game.state.step = Step::Main2;
    pass_pair(&mut game);
    trigger_target(&mut game, Target::Card(stolen));
    settle(&mut game);
    assert_ne!(game.object_ref(stolen), exact);
    assert_eq!(game.object_view(stolen).controller, opponent);
    assert_eq!(
        game.observe(player).additional_end_steps.unwrap().pending,
        1
    );
    pass_pair(&mut game);
    trigger_target(&mut game, Target::Card(source));
    settle(&mut game);
    let extra = game.observe(player).additional_end_steps.unwrap();
    assert!(extra.current_is_additional);
    assert_eq!(extra.pending, 0);
}

#[test]
fn zell_grants_an_extra_land_counts_live_lands_and_returns_one_to_its_owner() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Zell Dincht", Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    let first = put(&mut game, player, "Forest", Zone::Hand);
    let second = put(&mut game, player, "Forest", Zone::Hand);
    let third = put(&mut game, player, "Forest", Zone::Hand);
    act(&mut game, Action::PlayLand { card: first });
    act(&mut game, Action::PlayLand { card: second });
    reject(&mut game, Action::PlayLand { card: third });
    assert_eq!(stats(&game, source), (2, 3));

    game.state.step = Step::Main2;
    pass_pair(&mut game);
    pass_pair(&mut game);
    let object = game.object_ref(first);
    act(&mut game, Action::ChooseEffectObject { object });
    assert_eq!(game.object_view(first).zone, Zone::Hand);
    assert_eq!(stats(&game, source), (1, 3));
}

const SYNTHETIC_FACTORIES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CreateTokens {
                token: Card::from_catalogue_id(5531),
                amount: 1,
                tapped: false,
            },
            Instruction::CreateTokens {
                token: Card::from_catalogue_id(5532),
                amount: 1,
                tapped: false,
            },
            Instruction::CreateTokens {
                token: Card::from_catalogue_id(5533),
                amount: 1,
                tapped: false,
            },
            Instruction::CreateTokens {
                token: Card::from_catalogue_id(5534),
                amount: 1,
                tapped: false,
            },
            Instruction::CreateEmblem(Card::from_catalogue_id(5535)),
        ],
    ),
    ..CardRules::EMPTY
};

#[test]
fn the_pinned_tokens_and_emblem_supply_their_printed_characteristics_and_triggers() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &SYNTHETIC_FACTORIES)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);
    for (index, name, subtype, color, kinds, legendary, expected) in [
        (5531, "Wizard Token", "Wizard", 4, 1, false, (0, 1)),
        (5532, "Robot Token", "Warrior", 2, 17, false, (3, 3)),
        (5533, "Angelo", "Dog", 17, 1, true, (1, 1)),
        (5534, "Darkstar", "Dog", 5, 1, true, (2, 2)),
    ] {
        let token = game
            .state
            .objects
            .iter()
            .position(|object| {
                object.zone == Zone::Battlefield && object.card == Card::from_catalogue_id(index)
            })
            .unwrap() as CardId;
        let view = game.object_view(token);
        assert!(view.is_token() && view.has_subtype(subtype));
        assert_eq!((view.name, view.colors, view.kinds), (name, color, kinds));
        assert_eq!(view.card.is_legendary(), legendary);
        assert_eq!(stats(&game, token), expected);
    }
    let victim = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(victim)));
    pass_pair(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 19);
    pass_pair(&mut game);
    trigger_target(&mut game, Target::Player(opponent));
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(game.state.players[opponent.index()].life, 18);
    assert_eq!(game.state.players[player.index()].life, 21);
}

fn advance_chapter(game: &mut Game) {
    game.begin_step(Step::Main1);
    game.prepare_triggers();
}

fn attack_with(game: &mut Game, attackers: &[CardId]) {
    game.begin_step(Step::BeginCombat);
    pass_pair(game);
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn battlefield_tokens(game: &Game, factory: u16) -> Vec<CardId> {
    let token = Card::from_catalogue_id(factory);
    game.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield && object.card == token && object.is_token())
                .then_some(id as CardId)
        })
        .collect()
}

#[test]
fn fenrir_searches_a_basic_then_rewards_the_next_creature_and_a_power_tie() {
    let player = Player::P0;
    let mut game = scenario(player);
    let saga = put(&mut game, player, "Summon: Fenrir", Zone::Hand);
    let town = put(&mut game, player, "Vector, Imperial Capital", Zone::Library);
    let forest = put(&mut game, player, "Forest", Zone::Library);
    cast(&mut game, saga, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert!(game.observe(player.other()).library_selection.is_none());
    reject(&mut game, Action::ChooseSearch { card: town });
    act(&mut game, Action::ChooseSearch { card: forest });
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.object_view(forest).zone, Zone::Battlefield);
    assert!(game.object_view(forest).tapped);

    advance_chapter(&mut game);
    settle(&mut game);
    let equipment = put(&mut game, player, "Swiftfoot Boots", Zone::Hand);
    cast(&mut game, equipment, None);
    settle(&mut game);
    assert_eq!(game.observe(player).delayed_spell_copies.len(), 1);
    for expected in [1, 0] {
        let bear = put(&mut game, player, "Grizzly Bears", Zone::Hand);
        cast(&mut game, bear, None);
        settle(&mut game);
        assert_eq!(game.counter_count(bear, CounterKind::PlusOne), expected);
    }

    put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    advance_chapter(&mut game);
    let mut behind = game.clone();
    put(
        &mut behind,
        player.other(),
        "Ghalta, Primal Hunger",
        Zone::Battlefield,
    );
    let hand = game.hand_count(player);
    settle(&mut behind);
    settle(&mut game);
    assert_eq!(behind.hand_count(player), hand);
    assert_eq!(game.hand_count(player), hand + 1);
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);
}

#[test]
fn cerberus_surveils_then_copies_once_and_twice_with_independent_targets() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let saga = put(&mut game, player, "Summon: G.F. Cerberus", Zone::Hand);
    let top = put(&mut game, player, "Shock", Zone::Library);
    cast(&mut game, saga, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert!(game.observe(opponent).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Graveyard,
        },
    );
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);

    for count in [1, 2] {
        advance_chapter(&mut game);
        settle(&mut game);
        assert_eq!(
            game.object_view(saga).zone,
            if count == 2 {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        let original = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut game, player, "Unsummon", Zone::Hand);
        let other: Vec<_> = (0..count)
            .map(|_| put(&mut game, opponent, "Bear Cub", Zone::Battlefield))
            .collect();
        cast(&mut game, spell, Some(Target::Card(original)));
        pass_pair(&mut game);
        for (index, &target) in other.iter().enumerate() {
            let choice = game.observe(player).retarget.unwrap();
            assert_eq!(choice.remaining_copies, (other.len() - index - 1) as u64);
            assert_eq!(choice.original[0].target, Target::Card(original));
            act(
                &mut game,
                Action::SetStackTarget {
                    index: 0,
                    target: Target::Card(target),
                },
            );
            act(&mut game, Action::FinishStackTargets);
        }
        settle(&mut game);
        for target in other.into_iter().chain([original]) {
            assert_eq!(game.object_view(target).zone, Zone::Hand);
        }
        assert!(game.observe(player).delayed_spell_copies.is_empty());
    }
}

#[test]
fn knights_of_round_make_four_batches_and_leave_indestructible_counters() {
    let player = Player::P0;
    let mut game = scenario(player);
    let saga = put(&mut game, player, "Summon: Knights of Round", Zone::Hand);
    let friend = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let opponent = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    cast(&mut game, saga, None);
    settle(&mut game);
    assert!(game.keywords(saga).contains(Keywords::INDESTRUCTIBLE));
    for chapter in 1..=4 {
        assert_eq!(battlefield_tokens(&game, 5236).len(), 3 * chapter);
        if chapter < 4 {
            advance_chapter(&mut game);
            settle(&mut game);
        }
    }
    let tokens = battlefield_tokens(&game, 5236);
    for &token in &tokens {
        let view = game.object_view(token);
        assert!(view.has_subtype("Knight"));
        assert_eq!(view.colors, 1);
        assert_eq!(stats(&game, token), (2, 2));
        assert!(!game.keywords(token).contains(Keywords::VIGILANCE));
    }

    advance_chapter(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);
    assert_eq!(game.counter_count(opponent, CounterKind::Indestructible), 0);
    assert_eq!(stats(&game, opponent), (2, 2));
    for creature in tokens.iter().copied().chain([friend]) {
        assert_eq!(game.counter_count(creature, CounterKind::Indestructible), 1);
        assert_eq!(stats(&game, creature), (4, 4));
    }
    game.finish_cleanup();
    for creature in tokens.into_iter().chain([friend]) {
        assert!(game.keywords(creature).contains(Keywords::INDESTRUCTIBLE));
        assert_eq!(stats(&game, creature), (2, 2));
    }
}

#[test]
fn leviathan_preserves_sea_creatures_and_uses_ward_and_the_two_attack_chapters() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let saga = put(&mut game, player, "Summon: Leviathan", Zone::Hand);
    let bear = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let octopus = put(
        &mut game,
        player,
        "Ultros, Obnoxious Octopus",
        Zone::Battlefield,
    );
    let changeling = put(&mut game, player, "Three Tree Mascot", Zone::Battlefield);
    cast(&mut game, saga, None);
    settle(&mut game);
    assert_eq!(game.object_view(bear).zone, Zone::Hand);
    assert_eq!(game.object_view(octopus).zone, Zone::Battlefield);
    assert_eq!(game.object_view(changeling).zone, Zone::Battlefield);
    let shock = put(&mut game, opponent, "Shock", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, shock, Some(Target::Card(saga)));
    pass_pair(&mut game);
    assert_eq!(
        game.observe(opponent)
            .resolution_payment
            .unwrap()
            .cost
            .generic,
        2
    );
    act(&mut game, Action::DeclineResolutionMana);
    assert_eq!(game.object_view(shock).zone, Zone::Graveyard);
    assert_eq!(game.object_view(saga).damage, 0);

    for lore in [1, 2] {
        let mut branch = game.clone();
        branch.set_counters(saga, Counters::of(CounterKind::Lore, lore));
        advance_chapter(&mut branch);
        settle(&mut branch);
        if lore == 2 {
            assert_eq!(branch.object_view(saga).zone, Zone::Graveyard);
        }
        let before = branch.hand_count(player);
        attack_with(&mut branch, &[octopus, changeling]);
        for _ in 0..10 {
            if branch.state.pending == Pending::Priority
                && branch.state.stack.is_empty()
                && branch.state.pending_triggers.is_empty()
            {
                break;
            }
            choose(&mut branch, |action| {
                matches!(action, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert_eq!(branch.hand_count(player), before + 2);
    }
}

#[test]
fn odin_destroys_an_enemy_then_grants_a_combat_trigger_and_draws_at_the_end() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let saga = put(&mut game, player, "Summon: Primal Odin", Zone::Hand);
    let friend = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let victim = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    cast(&mut game, saga, None);
    pass_pair(&mut game);
    assert!(!game.legal_actions().contains(&Action::ChooseTarget {
        target: Target::Card(friend)
    }));
    trigger_target(&mut game, Target::Card(victim));
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    let mut prior = game.clone();
    advance_chapter(&mut game);
    settle(&mut game);
    let mut finale = game.clone();
    advance_chapter(&mut finale);
    settle(&mut finale);
    assert_eq!(finale.hand_count(player), 2);
    assert_eq!(finale.object_view(saga).zone, Zone::Graveyard);
    for p in [player, opponent] {
        assert_eq!(finale.state.players[p.index()].life, 18);
    }

    for branch in [&mut prior, &mut game] {
        branch.state.objects[saga as usize].summoning_sick = false;
        attack_with(branch, &[saga]);
        pass_pair(branch);
        act(branch, Action::FinishBlockers);
        pass_pair(branch);
        settle(branch);
        assert_eq!(branch.state.players[opponent.index()].life, 15);
    }
    assert_eq!(prior.outcome(), None);
    assert_eq!(game.outcome(), Some(Outcome::Winner(player)));
}

#[test]
fn titan_mills_and_recovers_own_lands_then_counts_them_at_resolution() {
    let player = Player::P1;
    let mut game = scenario(player);
    let saga = put(&mut game, player, "Summon: Titan", Zone::Hand);
    let friend = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let opposing_land = put(&mut game, player.other(), "Forest", Zone::Graveyard);
    let old_land = put(&mut game, player, "Plains", Zone::Graveyard);
    put(&mut game, player, "Forest", Zone::Battlefield);
    for _ in 0..4 {
        put(&mut game, player, "Shock", Zone::Library);
    }
    let milled_land = put(&mut game, player, "Forest", Zone::Library);
    cast(&mut game, saga, None);
    settle(&mut game);
    assert!(
        game.keywords(saga)
            .contains(Keywords::REACH.union(Keywords::TRAMPLE))
    );
    assert_eq!(game.object_view(milled_land).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].library.len(), 8);
    advance_chapter(&mut game);
    settle(&mut game);
    for land in [old_land, milled_land] {
        let view = game.object_view(land);
        assert_eq!(view.zone, Zone::Battlefield);
        assert!(view.tapped);
    }
    assert_eq!(game.object_view(opposing_land).zone, Zone::Graveyard);
    advance_chapter(&mut game);
    for illegal in [saga, enemy] {
        assert!(!game.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(illegal)
        }));
    }
    trigger_target(&mut game, Target::Card(friend));
    let late = put(&mut game, player, "Plains", Zone::Hand);
    game.move_object(late, Zone::Battlefield);
    settle(&mut game);
    assert_eq!(stats(&game, friend), (6, 6));
    assert!(game.keywords(friend).contains(Keywords::TRAMPLE));
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);
    game.finish_cleanup();
    assert_eq!(stats(&game, friend), (2, 2));
    assert!(!game.keywords(friend).contains(Keywords::TRAMPLE));
}

#[test]
fn final_days_counts_own_creatures_for_the_flashback_horror_batch() {
    let player = Player::P0;
    for flashback in [false, true] {
        let mut game = scenario(player);
        for name in ["Bear Cub", "Grizzly Bears", "Hill Giant", "Forest"] {
            put(&mut game, player, name, Zone::Graveyard);
        }
        put(&mut game, player.other(), "Grizzly Bears", Zone::Graveyard);
        let spell = put(
            &mut game,
            player,
            "The Final Days",
            if flashback {
                Zone::Graveyard
            } else {
                Zone::Hand
            },
        );
        if flashback {
            act(&mut game, Action::AnnounceFlashback { card: spell });
            pay(&mut game);
        } else {
            cast(&mut game, spell, None);
        }
        settle(&mut game);
        let tokens = battlefield_tokens(&game, 5536);
        assert_eq!(tokens.len(), if flashback { 3 } else { 2 });
        for token in tokens {
            let view = game.object_view(token);
            assert!(view.tapped && view.has_subtype("Horror"));
            assert_eq!((view.colors, view.controller), (4, player));
            assert_eq!(stats(&game, token), (2, 2));
        }
        assert_eq!(
            game.object_view(spell).zone,
            if flashback {
                Zone::Exile
            } else {
                Zone::Graveyard
            }
        );
    }
}

#[test]
fn ultima_destroys_both_types_and_ends_the_turn_before_a_death_trigger_stacks() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let rat = put(&mut game, player, "Undercity Dire Rat", Zone::Battlefield);
    let artifact = put(&mut game, opponent, "Swiftfoot Boots", Zone::Battlefield);
    let both = put(&mut game, opponent, "Three Tree Mascot", Zone::Battlefield);
    let indestructible = put(
        &mut game,
        player,
        "Summon: Knights of Round",
        Zone::Battlefield,
    );
    let land = put(&mut game, opponent, "Forest", Zone::Battlefield);
    let enchantment = put(&mut game, player, "A Realm Reborn", Zone::Battlefield);
    let spell = put(&mut game, player, "Ultima", Zone::Hand);
    let turn = game.state.turn;
    cast(&mut game, spell, None);
    settle(&mut game);
    for id in [rat, artifact, both] {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
    for id in [indestructible, land, enchantment] {
        assert_eq!(game.object_view(id).zone, Zone::Battlefield);
    }
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    assert!(battlefield_tokens(&game, 5302).is_empty());
    assert_eq!(game.state.turn, turn + 1);
    assert_eq!(game.state.active, opponent);
}

fn activate(game: &mut Game, source: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}

fn assert_crystal_discount(game: &Game, crystal: CardId, spell: &str, pool: Mana) {
    let player = game.actor().unwrap();
    let mut discounted = game.clone();
    let spell = put(&mut discounted, player, spell, Zone::Hand);
    discounted.state.players[player.index()].mana = pool;
    let announce = Action::AnnounceCast { card: spell };
    let mut without = discounted.clone();
    without.move_object(crystal, Zone::Exile);
    reject(&mut without, announce.clone());
    act(&mut discounted, announce);
    pay(&mut discounted);
    assert_eq!(discounted.object_view(spell).zone, Zone::Stack);
    assert_eq!(discounted.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn darkness_crystal_links_only_actual_opposing_creature_exiles() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let crystal = put(&mut game, player, "The Darkness Crystal", Zone::Hand);
    cast(&mut game, crystal, None);
    settle(&mut game);
    assert_crystal_discount(&game, crystal, "Undercity Dire Rat", [0, 0, 1, 0, 0, 0]);

    let victim = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let friendly = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let unlinked = put(&mut game, opponent, "Hill Giant", Zone::Exile);
    game.move_object(victim, Zone::Graveyard);
    game.move_object(friendly, Zone::Graveyard);
    assert_eq!(game.object_view(victim).zone, Zone::Exile);
    assert_eq!(game.object_view(friendly).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 22);

    activate(&mut game, crystal);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(unlinked),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    pay(&mut game);
    settle(&mut game);
    let returned = game.object_view(victim);
    assert_eq!(
        (returned.zone, returned.owner, returned.controller),
        (Zone::Battlefield, opponent, player)
    );
    assert!(returned.tapped);
    assert_eq!(game.counter_count(victim, CounterKind::PlusOne), 2);
}

#[test]
fn earth_crystal_doubles_the_announced_shares_for_surviving_creatures() {
    let player = Player::P0;
    let mut game = scenario(player);
    let crystal = put(&mut game, player, "The Earth Crystal", Zone::Hand);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opponent = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    cast(&mut game, crystal, None);
    settle(&mut game);
    assert_crystal_discount(&game, crystal, "Grizzly Bears", [0, 0, 0, 0, 1, 0]);

    activate(&mut game, crystal);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(opponent),
        },
    );
    for card in [first, second] {
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    for _ in 0..2 {
        act(&mut game, Action::AllocateTargetAmount { amount: 1 });
    }
    pay(&mut game);
    let mut responded = game.clone();
    responded.move_object(second, Zone::Hand);
    settle(&mut responded);
    assert_eq!(responded.counter_count(first, CounterKind::PlusOne), 2);
    assert_eq!(responded.counter_count(second, CounterKind::PlusOne), 0);
    settle(&mut game);
    for card in [first, second] {
        assert_eq!(game.counter_count(card, CounterKind::PlusOne), 2);
    }
    assert_eq!(game.counter_count(opponent, CounterKind::PlusOne), 0);
}

#[test]
fn water_crystal_adds_four_only_to_a_positive_opponent_mill() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let crystal = put(&mut game, player, "The Water Crystal", Zone::Hand);
    cast(&mut game, crystal, None);
    settle(&mut game);
    assert_crystal_discount(&game, crystal, "Ice Flan", [0, 2, 0, 0, 0, 3]);
    activate(&mut game, crystal);
    pay(&mut game);
    let mut empty = game.clone();
    settle(&mut empty);
    assert_eq!(empty.state.players[opponent.index()].library.len(), 8);

    for name in ["Island", "Opt"] {
        put(&mut game, player, name, Zone::Hand);
    }
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].library.len(), 2);
    assert_eq!(game.state.players[player.index()].library.len(), 8);
    assert!(game.object_view(crystal).tapped);
}

#[test]
fn wind_crystal_gives_only_current_controlled_creatures_flying_and_doubles_lifelink() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let crystal = put(&mut game, player, "The Wind Crystal", Zone::Hand);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    cast(&mut game, crystal, None);
    settle(&mut game);
    assert_crystal_discount(&game, crystal, "Cloudbound Moogle", [2, 0, 0, 0, 0, 2]);
    activate(&mut game, crystal);
    pay(&mut game);
    settle(&mut game);
    let late = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let keywords = Keywords::FLYING.union(Keywords::LIFELINK);
    assert!(game.keywords(bear).contains(keywords));
    for id in [late, enemy] {
        assert!(!game.keywords(id).contains(Keywords::FLYING));
        assert!(!game.keywords(id).contains(Keywords::LIFELINK));
    }
    attack_with(&mut game, &[bear]);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::Block {
            blocker: enemy,
            attacker: bear,
        },
    );
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 18);
    assert_eq!(game.state.players[player.index()].life, 24);
    game.finish_cleanup();
    assert!(!game.keywords(bear).contains(Keywords::FLYING));
    assert!(!game.keywords(bear).contains(Keywords::LIFELINK));
}

#[test]
fn masamune_requires_an_available_block_and_doubles_its_bearers_death_trigger() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let rat = put(&mut game, player, "Undercity Dire Rat", Zone::Battlefield);
    let blocker = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let sword = put(&mut game, player, "The Masamune", Zone::Hand);
    cast(&mut game, sword, None);
    settle(&mut game);
    activate(&mut game, sword);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(rat),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(
        game.object_view(sword).attached_to,
        Some(game.object_ref(rat))
    );
    assert!(!game.keywords(rat).contains(Keywords::FIRST_STRIKE));

    attack_with(&mut game, &[rat]);
    assert!(game.keywords(rat).contains(Keywords::FIRST_STRIKE));
    pass_pair(&mut game);
    reject(&mut game, Action::FinishBlockers);
    act(
        &mut game,
        Action::Block {
            blocker,
            attacker: rat,
        },
    );
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::FirstStrikeDamage);
    assert_eq!(game.object_view(blocker).zone, Zone::Graveyard);
    assert_eq!(game.object_view(rat).damage, 0);

    game.state.players[player.index()].mana[3] = 1;
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(rat)));
    for _ in 0..16 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
        {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.object_view(rat).zone, Zone::Graveyard);
    assert_eq!(battlefield_tokens(&game, 5302).len(), 2);
}

#[test]
fn ultima_origin_replaces_the_attacked_lands_abilities_and_doubles_its_colorless_mana() {
    let player = Player::P0;
    let colorless = [0, 0, 0, 0, 0, 1];
    let mut game = scenario(player);
    let ultima = put(
        &mut game,
        player,
        "Ultima, Origin of Oblivion",
        Zone::Battlefield,
    );
    let town = put(
        &mut game,
        player,
        "Vector, Imperial Capital",
        Zone::Battlefield,
    );
    assert!(game.keywords(ultima).contains(Keywords::FLYING));
    assert!(game.object_view(town).has_subtype("Town"));
    attack_with(&mut game, &[ultima]);
    trigger_target(&mut game, Target::Card(town));
    settle(&mut game);
    assert_eq!(game.counter_count(town, CounterKind::Blight), 1);
    assert!(!game.object_view(town).has_subtype("Town"));

    let tap = Action::TapGrantedMana {
        card: town,
        grantor: game.object_ref(town),
        production: colorless,
    };
    game.state.players[player.index()].mana = [0; 6];
    act(&mut game, tap.clone());
    assert_eq!(game.state.players[player.index()].mana[5], 2);
    game.move_object(ultima, Zone::Graveyard);
    game.untap_objects(&[town], untap_rules::UntapCause::Effect);
    act(&mut game, tap);
    assert_eq!(game.state.players[player.index()].mana[5], 3);
    game.pay_counters(town, &Counters::of(CounterKind::Blight, 1));
    assert!(game.object_view(town).has_subtype("Town"));
}

fn ability(source: CardId, index: u16) -> Action {
    Action::AnnounceActivate {
        card: source,
        ability: index,
        counter_payment: Counters::default(),
    }
}

fn tap_production(source: CardId, production: Mana) -> Action {
    Action::ActivateMana {
        card: source,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}

fn crew_with(game: &mut Game, vehicle: CardId, pilot: CardId) {
    activate(game, vehicle);
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
    let object = game.object_ref(pilot);
    act(game, Action::ChooseCrewer { object });
    act(game, Action::FinishCrew);
    settle(game);
}

#[test]
fn tellah_samples_mana_spent_at_each_threshold_and_keeps_the_damage_after_sacrifice() {
    let player = Player::P0;
    let opponent = player.other();
    for spent in [1, 4, 8] {
        let mut game = scenario(opponent);
        let tellah = put(&mut game, player, "Tellah, Great Sage", Zone::Battlefield);
        let victim = put(&mut game, opponent, "Opt", Zone::Hand);
        let counter = put(&mut game, player, "Syncopate", Zone::Hand);
        cast(&mut game, victim, None);
        act(&mut game, Action::Pass);
        act(&mut game, Action::AnnounceCast { card: counter });
        act(&mut game, Action::ChooseX { value: spent - 1 });
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Spell(victim),
            },
        );
        pay(&mut game);
        pass_pair(&mut game);
        assert_eq!(battlefield_tokens(&game, 5328).len(), 1);
        assert_eq!(game.hand_count(player), if spent >= 4 { 2 } else { 0 });
        assert_eq!(
            game.object_view(tellah).zone,
            if spent == 8 {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(
            game.state.players[opponent.index()].life,
            if spent == 8 { 12 } else { 20 }
        );
        assert_eq!(game.object_view(victim).zone, Zone::Stack);
    }
}

#[test]
fn emperor_restricts_its_mana_and_transforms_after_its_third_counter() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let emperor = put(
        &mut game,
        player,
        "The Emperor of Palamecia",
        Zone::Battlefield,
    );
    let front = game.object_view(emperor).card;
    let exact = game.object_ref(emperor);
    let creature = put(&mut game, player, "Hill Giant", Zone::Hand);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    let land = put(&mut game, player, "Forest", Zone::Graveyard);
    let artifact = put(&mut game, player, "Swiftfoot Boots", Zone::Graveyard);
    put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    put(&mut game, opponent, "Opt", Zone::Graveyard);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
    reject(&mut game, tap_production(emperor, [0, 0, 0, 0, 1, 0]));
    act(&mut game, tap_production(emperor, [0, 0, 0, 1, 0, 0]));
    reject(&mut game, Action::AnnounceCast { card: creature });
    cast(&mut game, shock, Some(Target::Player(opponent)));
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 18);
    assert_eq!(game.counter_count(emperor, CounterKind::PlusOne), 0);

    game.set_counters(emperor, Counters::of(CounterKind::PlusOne, 2));
    game.state.players[player.index()].mana = [8; 6];
    let enough = put(&mut game, player, "The Final Days", Zone::Hand);
    cast(&mut game, enough, None);
    settle(&mut game);
    assert_eq!(game.object_ref(emperor), exact);
    assert_eq!(game.object_view(emperor).card, front.back_face().unwrap());
    assert_eq!(game.counter_count(emperor, CounterKind::PlusOne), 3);
    assert_eq!(stats(&game, emperor), (6, 6));

    game.untap_objects(&[emperor], untap_rules::UntapCause::Effect);
    attack_with(&mut game, &[emperor]);
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 15);
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
}

#[test]
fn gold_saucer_draws_after_two_exact_artifacts_and_coin_result_replays() {
    let player = Player::P0;
    let mut game = scenario(player);
    let saucer = put(&mut game, player, "The Gold Saucer", Zone::Hand);
    let first = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let second = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let nonartifact = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    act(&mut game, Action::PlayLand { card: saucer });
    let mut mana = game.clone();
    mana.state.players[player.index()].mana = [0; 6];
    act(&mut mana, tap_production(saucer, [0, 0, 0, 0, 0, 1]));
    assert_eq!(mana.state.players[player.index()].mana[5], 1);

    let mut coin = game.clone();
    let mut rng = coin.state.rng;
    let won = rng.below(2) == 1;
    act(&mut coin, ability(saucer, 1));
    pay(&mut coin);
    act(&mut coin, Action::Pass);
    let frozen = coin.snapshot();
    let index = coin
        .legal_actions()
        .iter()
        .position(|action| *action == Action::Pass)
        .unwrap();
    let mut indexed = coin.clone();
    indexed
        .apply_index(player.other(), indexed.revision(), index)
        .unwrap();
    assert_eq!(coin.snapshot(), frozen);
    act(&mut coin, Action::Pass);
    assert_eq!(coin.snapshot(), indexed.snapshot());
    assert_eq!(coin.state.rng, rng);
    assert_eq!(battlefield_tokens(&coin, 5302).len(), usize::from(won));

    act(&mut game, ability(saucer, 2));
    let illegal = game.object_ref(nonartifact);
    reject(&mut game, Action::ChooseAdditionalCost { object: illegal });
    let first_cost = Action::ChooseAdditionalCost {
        object: game.object_ref(first),
    };
    act(&mut game, first_cost.clone());
    reject(&mut game, first_cost);
    let object = game.object_ref(second);
    act(&mut game, Action::ChooseAdditionalCost { object });
    pay(&mut game);
    for id in [first, second] {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
    assert!(game.object_view(saucer).tapped);
    settle(&mut game);
    assert_eq!(game.hand_count(player), 1);
}

#[test]
fn lunar_whale_exposes_the_top_privately_and_allows_normal_play_after_its_attack() {
    let player = Player::P1;
    let mut game = scenario(player);
    let whale = put(&mut game, player, "The Lunar Whale", Zone::Battlefield);
    let pilot = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    let equipment = put(&mut game, player, "Swiftfoot Boots", Zone::Library);
    let land = put(&mut game, player, "Island", Zone::Library);
    let play = Action::PlayLand { card: land };
    assert_eq!(
        game.observe(player).known_library_top[player.index()]
            .unwrap()
            .object,
        game.object_ref(land),
    );
    assert!(game.observe(player.other()).known_library_top[player.index()].is_none());
    reject(&mut game, play.clone());
    crew_with(&mut game, whale, pilot);
    assert_eq!(stats(&game, whale), (3, 5));
    assert!(game.keywords(whale).contains(Keywords::FLYING));
    attack_with(&mut game, &[whale]);
    reject(&mut game, play.clone());

    game.state.attackers.clear();
    game.state.step = Step::Main2;
    game.state.priority = player;
    let mut blinked = game.clone();
    blinked.move_object(whale, Zone::Exile);
    blinked.move_object(whale, Zone::Battlefield);
    reject(&mut blinked, play.clone());
    act(&mut game, play);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    cast(&mut game, equipment, None);
    settle(&mut game);
    assert_eq!(game.object_view(equipment).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn prima_vista_animates_only_after_a_qualifying_paid_noncreature_spell_or_crew() {
    let player = Player::P0;
    let mut game = scenario(player);
    let vista = put(&mut game, player, "The Prima Vista", Zone::Battlefield);
    let pilot = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let mut crewed = game.clone();
    crew_with(&mut crewed, vista, pilot);
    assert_eq!(stats(&crewed, vista), (5, 3));
    assert!(crewed.object_view(vista).is_creature());
    for name in ["Hill Giant", "Swiftfoot Boots"] {
        let spell = put(&mut game, player, name, Zone::Hand);
        cast(&mut game, spell, None);
        settle(&mut game);
        assert!(!game.object_view(vista).is_creature());
    }
    let days = put(&mut game, player, "The Final Days", Zone::Hand);
    cast(&mut game, days, None);
    pass_pair(&mut game);
    assert_eq!(game.object_view(days).zone, Zone::Stack);
    assert_eq!(game.object_view(vista).kinds & 17, 17);
    assert_eq!(stats(&game, vista), (5, 3));
    assert!(game.keywords(vista).contains(Keywords::FLYING));
    settle(&mut game);
    game.finish_cleanup();
    assert!(!game.object_view(vista).is_creature());
}

#[test]
fn regalia_reveals_only_through_the_first_land_and_replays_the_random_bottom() {
    let player = Player::P0;
    let mut game = scenario(player);
    let vehicle = put(&mut game, player, "The Regalia", Zone::Hand);
    let pilot = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    let untouched = put(&mut game, player, "Forest", Zone::Library);
    let land = put(&mut game, player, "Island", Zone::Library);
    let middle = put(&mut game, player, "Opt", Zone::Library);
    let top = put(&mut game, player, "Shock", Zone::Library);
    cast(&mut game, vehicle, None);
    settle(&mut game);
    crew_with(&mut game, vehicle, pilot);
    assert_eq!(stats(&game, vehicle), (4, 4));
    assert!(game.keywords(vehicle).contains(Keywords::HASTE));
    attack_with(&mut game, &[vehicle]);
    pass_pair(&mut game);
    let reveal = game.observe(player.other()).library_selection.unwrap();
    assert_eq!(
        reveal
            .remaining
            .iter()
            .map(|view| view.id)
            .collect::<Vec<_>>(),
        vec![top, middle, land]
    );
    assert_eq!(reveal.candidates, vec![game.object_ref(land)]);
    reject(&mut game, Action::SelectLibrary { card: top });
    let mut replay = game.clone();
    for branch in [&mut game, &mut replay] {
        act(branch, Action::SelectLibrary { card: land });
        if branch
            .legal_actions()
            .contains(&Action::FinishLibrarySelection)
        {
            act(branch, Action::FinishLibrarySelection);
        }
        settle(branch);
        let view = branch.object_view(land);
        assert_eq!(view.zone, Zone::Battlefield);
        assert!(view.tapped);
        assert_eq!(branch.top_library_card(player), Some(untouched));
        let bottom = &branch.state.players[player.index()].library[..2];
        assert!(bottom.contains(&top) && bottom.contains(&middle));
    }
    assert_eq!(game.snapshot(), replay.snapshot());
}

#[test]
fn torgal_counts_each_live_dog_or_wolf_for_only_the_first_human_spell() {
    let player = Player::P1;
    let mut game = scenario(player);
    let torgal = put(&mut game, player, "Torgal, A Fine Hound", Zone::Battlefield);
    let changeling = put(&mut game, player, "Three Tree Mascot", Zone::Battlefield);
    let dog = put(&mut game, player, "Chrome Companion", Zone::Battlefield);
    put(
        &mut game,
        player.other(),
        "Chrome Companion",
        Zone::Battlefield,
    );
    assert!(
        game.object_view(changeling).has_subtype("Dog")
            && game.object_view(changeling).has_subtype("Wolf")
    );
    let first = put(&mut game, player, "Zell Dincht", Zone::Hand);
    let second = put(
        &mut game,
        player,
        "Lightning, Security Sergeant",
        Zone::Hand,
    );
    cast(&mut game, first, None);
    pass_pair(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Stack);
    game.move_object(dog, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.counter_count(first, CounterKind::PlusOne), 2);
    cast(&mut game, second, None);
    settle(&mut game);
    assert_eq!(game.counter_count(second, CounterKind::PlusOne), 0);
    game.state.players[player.index()].mana = [0; 6];
    act(&mut game, tap_production(torgal, [0, 0, 1, 0, 0, 0]));
    assert_eq!(game.state.players[player.index()].mana[2], 1);
}

fn settle_triggers(game: &mut Game) {
    for _ in 0..32 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("trigger ordering exceeded the scenario budget");
}

#[test]
fn terra_recovers_its_milled_enchantment_and_esper_copies_a_saga_then_returns() {
    let player = Player::P0;
    let mut game = scenario(player);
    let terra = put(&mut game, player, "Terra, Magical Adept", Zone::Hand);
    let front = game.object_view(terra).card;
    let old = put(&mut game, player, "A Realm Reborn", Zone::Graveyard);
    let milled = put(&mut game, player, "A Realm Reborn", Zone::Library);
    let instant = put(&mut game, player, "Shock", Zone::Library);
    let original = put(&mut game, player, "Summon: Fat Chocobo", Zone::Battlefield);
    cast(&mut game, terra, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    for card in [old, instant] {
        let object = game.object_ref(card);
        reject(&mut game, Action::SelectMilled { object });
    }
    let object = game.object_ref(milled);
    act(&mut game, Action::SelectMilled { object });
    assert_eq!(game.object_view(milled).zone, Zone::Hand);
    assert_eq!(game.object_view(instant).zone, Zone::Graveyard);

    game.state.objects[terra as usize].summoning_sick = false;
    let initial = game.object_ref(terra);
    activate(&mut game, terra);
    pay(&mut game);
    pass_pair(&mut game);
    assert_ne!(game.object_ref(terra), initial);
    assert_eq!(game.object_view(terra).card, front.back_face().unwrap());
    assert!(game.keywords(terra).contains(Keywords::FLYING));
    assert_eq!(game.counter_count(terra, CounterKind::Lore), 1);
    trigger_target(&mut game, Target::Card(original));
    pass_pair(&mut game);
    let lore = game.observe(player).copied_saga_lore.unwrap();
    assert_eq!(lore.maximum, 3);
    let copy = lore.object;
    assert!(game.object_view(copy.card).is_token());
    assert_eq!(
        game.object_view(copy.card).card,
        game.object_view(original).card
    );
    assert!(game.keywords(copy.card).contains(Keywords::HASTE));
    reject(
        &mut game,
        Action::ChooseCopiedSagaLore {
            object: copy,
            amount: 4,
        },
    );
    act(
        &mut game,
        Action::ChooseCopiedSagaLore {
            object: copy,
            amount: 1,
        },
    );
    settle_triggers(&mut game);
    assert_eq!(game.counter_count(copy.card, CounterKind::Lore), 2);
    assert_eq!(battlefield_tokens(&game, 5269).len(), 1);

    game.state.step = Step::Main2;
    pass_pair(&mut game);
    settle_triggers(&mut game);
    assert_ne!(game.object_view(copy.card).zone, Zone::Battlefield);
    assert_eq!(game.object_view(original).zone, Zone::Battlefield);
    game.move_object(original, Zone::Exile);
    game.set_counters(terra, Counters::of(CounterKind::Lore, 3));
    game.state.players[player.index()].mana = [0; 6];
    advance_chapter(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(terra).card, front);
    assert_eq!(game.object_view(terra).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].mana, [2, 2, 2, 2, 2, 0]);
}

#[test]
fn traveling_chocobo_hides_the_top_and_doubles_a_controlled_landfall() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bird = put(&mut game, player, "Traveling Chocobo", Zone::Battlefield);
    let tifa = put(&mut game, player, "Tifa Lockhart", Zone::Battlefield);
    let rival_tifa = put(
        &mut game,
        player.other(),
        "Tifa Lockhart",
        Zone::Battlefield,
    );
    let top_bird = put(&mut game, player, "Most Decrepit Old Bird", Zone::Library);
    let instant = put(&mut game, player, "Opt", Zone::Library);
    let land = put(&mut game, player, "Forest", Zone::Library);
    assert_eq!(stats(&game, bird), (3, 2));
    assert_eq!(
        game.observe(player).known_library_top[player.index()]
            .unwrap()
            .object,
        game.object_ref(land)
    );
    assert!(game.observe(player.other()).known_library_top[player.index()].is_none());
    reject(&mut game, Action::AnnounceCast { card: top_bird });
    act(&mut game, Action::PlayLand { card: land });
    settle_triggers(&mut game);
    assert_eq!(stats(&game, tifa), (4, 2));
    assert_eq!(stats(&game, rival_tifa), (1, 2));
    reject(&mut game, Action::AnnounceCast { card: instant });
    game.move_object(instant, Zone::Exile);
    cast(&mut game, top_bird, None);
    settle_triggers(&mut game);
    assert_eq!(game.object_view(top_bird).zone, Zone::Battlefield);
}

#[test]
fn triple_triad_allows_only_its_own_card_and_the_strictly_cheaper_other_card() {
    let player = Player::P0;
    let opponent = player.other();
    for (other, allowed) in [("Opt", true), ("Hill Giant", false)] {
        let mut game = scenario(player);
        put(&mut game, player, "Triple Triad", Zone::Battlefield);
        let own = put(&mut game, player, "The Final Days", Zone::Library);
        let their = put(&mut game, opponent, other, Zone::Library);
        game.begin_step(Step::Upkeep);
        game.prepare_triggers();
        settle(&mut game);
        for card in [own, their] {
            assert_eq!(game.object_view(card).zone, Zone::Exile);
        }
        let free = |card| Action::AnnounceExileFree {
            card,
            face: CastFace::Normal,
            flash_surcharge: false,
        };
        reject(&mut game, free(own));
        game.state.step = Step::Main1;
        game.state.players[player.index()].mana = [0; 6];
        assert!(game.legal_actions().contains(&free(own)));
        assert_eq!(game.legal_actions().contains(&free(their)), allowed);
        let mut expired = game.clone();
        expired.finish_cleanup();
        assert!(expired.observe(player).exile_permissions.is_empty());
        act(&mut game, free(own));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(battlefield_tokens(&game, 5536).len(), 2);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    }
}

#[test]
fn ultimecia_surveys_both_events_and_transforms_only_after_atomic_payment() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Ultimecia, Time Sorceress", Zone::Hand);
    let front = game.object_view(source).card;
    let own = [(); 8].map(|_| put(&mut game, player, "Forest", Zone::Graveyard));
    let foreign = put(&mut game, player.other(), "Forest", Zone::Graveyard);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    for _ in 0..2 {
        choose(&mut game, |action| {
            matches!(
                action,
                Action::ArrangeLibrary {
                    destination: LibraryDestination::Top,
                    ..
                }
            )
        });
    }
    game.state.objects[source as usize].summoning_sick = false;
    attack_with(&mut game, &[source]);
    pass_pair(&mut game);
    assert!(game.observe(player).library_choice.is_some());
    assert!(game.observe(player.other()).library_choice.is_none());
    for _ in 0..2 {
        choose(&mut game, |action| {
            matches!(
                action,
                Action::ArrangeLibrary {
                    destination: LibraryDestination::Top,
                    ..
                }
            )
        });
    }

    game.begin_step(Step::End);
    game.prepare_triggers();
    let pool = [0, 2, 2, 0, 0, 4];
    game.state.players[player.index()].mana = pool;
    pass_pair(&mut game);
    let object = game.object_ref(foreign);
    reject(&mut game, Action::ChooseAdditionalCost { object });
    let payment = Action::PayResolutionMana { payment: pool };
    reject(&mut game, payment.clone());
    let object = game.object_ref(own[0]);
    act(&mut game, Action::ChooseAdditionalCost { object });
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    assert_eq!(declined.object_view(source).card, front);
    assert_eq!(declined.state.players[player.index()].mana, pool);
    for card in own {
        assert_eq!(declined.object_view(card).zone, Zone::Graveyard);
    }
    for card in &own[1..] {
        let object = game.object_ref(*card);
        act(&mut game, Action::ChooseAdditionalCost { object });
    }
    let exact = game.object_ref(source);
    act(&mut game, payment);
    settle(&mut game);
    assert_eq!(game.object_ref(source), exact);
    assert_eq!(game.object_view(source).card, front.back_face().unwrap());
    assert_eq!(stats(&game, source), (7, 7));
    assert!(game.keywords(source).contains(Keywords::MENACE));
    for card in own {
        assert_eq!(game.object_view(card).zone, Zone::Exile);
    }
    assert_eq!(game.object_view(foreign).zone, Zone::Graveyard);
    let turn = game.state.turn;
    game.finish_cleanup();
    assert_eq!(game.state.turn, turn + 1);
    assert_eq!(game.state.active, player);
}

#[test]
fn unexpected_request_reattaches_one_equipment_then_unattaches_that_exact_one() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let borrowed = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let bow = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let foreign = put(&mut game, opponent, "Swiftfoot Boots", Zone::Battlefield);
    let spell = put(&mut game, player, "Unexpected Request", Zone::Hand);
    game.tap_objects(&[borrowed]);
    cast(&mut game, spell, Some(Target::Card(borrowed)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(borrowed).controller, player);
    assert!(!game.object_view(borrowed).tapped);
    assert!(game.keywords(borrowed).contains(Keywords::HASTE));
    let equipment = game.object_ref(foreign);
    reject(
        &mut game,
        Action::ChooseEquipmentAttachment {
            equipment: Some(equipment),
        },
    );
    let mut declined = game.clone();
    act(
        &mut declined,
        Action::ChooseEquipmentAttachment { equipment: None },
    );
    assert!(declined.observe(player).delayed_steps.is_none());

    let equipment = game.object_ref(bow);
    act(
        &mut game,
        Action::ChooseEquipmentAttachment {
            equipment: Some(equipment),
        },
    );
    assert_eq!(
        game.object_view(bow).attached_to,
        Some(game.object_ref(borrowed))
    );
    activate(&mut game, bow);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(second),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(
        game.object_view(bow).attached_to,
        Some(game.object_ref(second))
    );
    game.state.step = Step::Main2;
    pass_pair(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(bow).attached_to, None);
    assert_eq!(game.object_view(borrowed).controller, player);
    game.finish_cleanup();
    assert_eq!(game.object_view(borrowed).controller, opponent);
}

#[test]
fn vaan_aggregates_the_combat_hit_and_casting_the_foreign_card_grows_the_union() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let vaan = put(&mut game, player, "Vaan, Street Thief", Zone::Battlefield);
    let both = put(&mut game, player, "Three Tree Mascot", Zone::Battlefield);
    let plain = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let offered = put(&mut game, opponent, "Hill Giant", Zone::Library);
    attack_with(&mut game, &[vaan, both, plain]);
    pass_pair(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    let hit = game.state.stack[0].damage_event.unwrap();
    assert_eq!((hit.player, hit.amount), (opponent, 4));
    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 3];
    pass_pair(&mut game);
    assert_eq!(game.object_view(offered).zone, Zone::Exile);
    let mut declined = game.clone();
    act(&mut declined, Action::FinishResolutionCasts);
    assert_eq!(battlefield_tokens(&declined, 5302).len(), 1);
    assert_eq!(declined.object_view(offered).zone, Zone::Exile);

    let object = game.object_ref(offered);
    act(
        &mut game,
        Action::CastExiledDuringResolution {
            object,
            face: CastFace::Normal,
        },
    );
    pay(&mut game);
    act(&mut game, Action::FinishResolutionCasts);
    settle(&mut game);
    assert!(battlefield_tokens(&game, 5302).is_empty());
    for card in [vaan, both] {
        assert_eq!(game.counter_count(card, CounterKind::PlusOne), 1);
    }
    assert_eq!(game.counter_count(plain, CounterKind::PlusOne), 0);
    let giant = game.object_view(offered);
    assert_eq!(
        (giant.zone, giant.owner, giant.controller),
        (Zone::Battlefield, opponent, player)
    );
}

#[test]
fn vanille_recovers_an_older_permanent_then_melds_only_when_the_pair_is_owned() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let vanille = put(&mut game, player, "Vanille, Cheerful l'Cie", Zone::Hand);
    let old = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let foreign = put(&mut game, opponent, "Forest", Zone::Graveyard);
    let instant = put(&mut game, player, "Shock", Zone::Library);
    let land = put(&mut game, player, "Forest", Zone::Library);
    cast(&mut game, vanille, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    for id in [foreign, instant] {
        let object = game.object_ref(id);
        reject(&mut game, Action::ChooseEffectObject { object });
    }
    let object = game.object_ref(old);
    act(&mut game, Action::ChooseEffectObject { object });
    assert_eq!(game.object_view(old).zone, Zone::Hand);
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);

    let mut stolen = game.clone();
    let rival_fang = put(
        &mut stolen,
        opponent,
        "Fang, Fearless l'Cie",
        Zone::Battlefield,
    );
    stolen.record_control(stolen.object_ref(rival_fang), player, None);
    advance_chapter(&mut stolen);
    assert!(stolen.state.stack.is_empty());
    let fang = put(&mut game, player, "Fang, Fearless l'Cie", Zone::Battlefield);
    advance_chapter(&mut game);
    pass_pair(&mut game);
    choose(&mut game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    let object = game.object_ref(fang);
    act(&mut game, Action::ChooseEffectObject { object });
    settle(&mut game);
    let meld = game
        .observe(player)
        .objects
        .into_iter()
        .find(|view| view.zone == Zone::Battlefield && view.name == "Ragnarok, Divine Deliverance")
        .unwrap();
    assert_eq!(stats(&game, meld.id), (7, 6));
    assert!(
        game.observe(player)
            .melded_permanents
            .iter()
            .any(|entry| entry.object == meld.reference)
    );
    assert!(!game.on_battlefield(fang));
}
