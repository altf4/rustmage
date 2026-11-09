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
    reject(&mut game, Action::SelectLibrary { card: town });
    act(&mut game, Action::SelectLibrary { card: forest });
    act(&mut game, Action::FinishLibrarySelection);
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
    let enchantment = put(&mut game, player, "Nowhere to Run", Zone::Battlefield);
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
