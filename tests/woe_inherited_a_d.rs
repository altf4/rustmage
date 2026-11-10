//! Paid card-level proofs for inherited Wilds of Eldraine definitions.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
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
        let action = match game.state.pending {
            Pending::Attackers => Action::FinishAttackers,
            Pending::Blockers => Action::FinishBlockers,
            Pending::Priority => Action::Pass,
            ref pending => panic!("unexpected turn choice {pending:?}"),
        };
        act(game, action);
    }
    panic!("turn did not finish");
}

#[test]
fn groves_bounty_pays_positive_x_with_zero_targets_or_exact_counter_shares() {
    let player = Player::P0;
    let mut game = scenario(player);
    let otter = put(
        &mut game,
        player,
        "Elusive Otter // Grove's Bounty",
        Zone::Hand,
    );
    let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foreign = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    act(&mut game, Action::AnnounceAdventure { card: otter });
    act(&mut game, Action::ChooseX { value: 3 });
    let payment = Action::PayCosts {
        payment: [0, 0, 0, 0, 1, 3],
    };

    let mut empty = game.clone();
    act(&mut empty, Action::FinishTargets);
    act(&mut empty, payment.clone());
    assert_eq!(empty.state.stack.last().unwrap().x, Some(3));
    assert!(empty.state.stack.last().unwrap().targets.is_empty());
    settle(&mut empty);
    assert_eq!(empty.object_view(otter).zone, Zone::Exile);
    for recipient in [first, second] {
        assert_eq!(empty.counter_count(recipient, CounterKind::PlusOne), 0);
    }
    act(&mut empty, Action::AnnounceExile { card: otter });
    act(
        &mut empty,
        Action::PayCosts {
            payment: [0, 1, 0, 0, 0, 0],
        },
    );
    settle(&mut empty);
    assert_eq!(empty.object_view(otter).name, "Elusive Otter");
    assert_eq!(empty.object_view(otter).zone, Zone::Battlefield);
    assert_eq!(stats(&empty, otter), (1, 1));

    reject(&mut game, target(foreign));
    act(&mut game, target(first));
    act(&mut game, target(second));
    act(&mut game, Action::FinishTargets);
    reject(&mut game, Action::AllocateTargetAmount { amount: 0 });
    reject(&mut game, Action::AllocateTargetAmount { amount: 3 });
    act(&mut game, Action::AllocateTargetAmount { amount: 1 });
    act(&mut game, Action::AllocateTargetAmount { amount: 2 });
    act(&mut game, payment);

    let mut stale = game.clone();
    cast(&mut stale, bounce, Some(Target::Card(first)));
    settle(&mut stale);
    assert_eq!(stale.object_view(first).zone, Zone::Hand);
    assert_eq!(stale.counter_count(second, CounterKind::PlusOne), 2);
    assert_eq!(stale.object_view(otter).zone, Zone::Exile);

    settle(&mut game);
    assert_eq!(game.counter_count(first, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(second, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(foreign, CounterKind::PlusOne), 0);
    assert_eq!(game.object_view(otter).zone, Zone::Exile);
}

#[test]
fn elusive_otter_prowess_changes_which_creatures_can_block_it() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let otter = put(
        &mut game,
        player,
        "Elusive Otter // Grove's Bounty",
        Zone::Hand,
    );
    let own_creature = put(&mut game, player, "Bear Cub", Zone::Hand);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    let boots = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let small = put(&mut game, opponent, "Llanowar Elves", Zone::Battlefield);
    let equal = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    cast(&mut game, otter, None);
    settle(&mut game);
    assert_eq!(stats(&game, otter), (1, 1));
    cast(&mut game, own_creature, None);
    settle(&mut game);
    assert_eq!(stats(&game, otter), (1, 1));
    act(&mut game, activate(boots, 0));
    act(&mut game, target(otter));
    pay(&mut game);
    settle(&mut game);

    cast(&mut game, shock, Some(Target::Player(opponent)));
    pass_twice(&mut game);
    assert_eq!(stats(&game, otter), (2, 2));
    assert_eq!(game.state.players[opponent.index()].life, 20);
    settle(&mut game);

    let mut cleanup = game.clone();
    finish_turn(&mut cleanup);
    assert_eq!(stats(&cleanup, otter), (1, 1));

    reach_attackers(&mut game);
    act(&mut game, Action::Attack { card: otter });
    act(&mut game, Action::FinishAttackers);
    pass_twice(&mut game);
    reject(
        &mut game,
        Action::Block {
            blocker: small,
            attacker: otter,
        },
    );
    act(
        &mut game,
        Action::Block {
            blocker: equal,
            attacker: otter,
        },
    );
    act(&mut game, Action::FinishBlockers);
    assert!(game.state.blocks.contains(&(equal, otter)));
}

fn reach_main(game: &mut Game) {
    for _ in 0..8 {
        if game.state.step == Step::Main1 {
            return;
        }
        act(game, Action::Pass);
    }
    panic!("main phase did not begin");
}

fn bargain(game: &mut Game, offering: Option<CardId>) {
    let sacrifice = offering.map(|card| game.object_ref(card));
    act(game, Action::ChooseBargain { sacrifice });
}

#[test]
fn roles_keep_controllers_separate_and_apply_timestamps_and_enchantment_bonuses() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let tale = put(&mut game, player, "A Tale for the Ages", Zone::Hand);
    let archon = put(&mut game, player, "Archon of the Wild Rose", Zone::Hand);
    let reaper = put(&mut game, player, "Ashiok's Reaper", Zone::Hand);
    let clothier = put(&mut game, player, "Charmed Clothier", Zone::Hand);
    let witch = put(
        &mut game,
        player,
        "Conceited Witch // Price of Beauty",
        Zone::Hand,
    );
    let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let aura = put(&mut game, player, "Bestial Bloodline", Zone::Hand);
    let foreign = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let plains = [(); 5].map(|_| put(&mut game, player, "Plains", Zone::Battlefield));
    let islands = [(); 7].map(|_| put(&mut game, opponent, "Island", Zone::Battlefield));
    let witches = [(); 2].map(|_| put(&mut game, opponent, "Diminisher Witch", Zone::Hand));
    let offerings = [(); 2].map(|_| put(&mut game, opponent, "Short Bow", Zone::Battlefield));
    let bounce = put(&mut game, opponent, "Unsummon", Zone::Hand);

    for card in [tale, archon, reaper] {
        cast(&mut game, card, None);
        settle(&mut game);
    }
    assert_eq!(stats(&game, first), (2, 2));
    act(&mut game, Action::AnnounceAdventure { card: witch });
    reject(&mut game, target(foreign));
    act(&mut game, target(first));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(witch).zone, Zone::Exile);
    assert_eq!(stats(&game, first), (7, 7));
    assert!(game.object_view(first).keywords.contains(Keywords::FLYING));
    let wicked = game
        .observe(player)
        .objects
        .into_iter()
        .find(|object| object.is_token() && object.attached_to == Some(game.object_ref(first)))
        .unwrap();
    assert_eq!(wicked.controller, player);
    assert!(wicked.has_subtype("Role"));

    cast(&mut game, aura, Some(Target::Card(first)));
    settle(&mut game);
    assert_eq!(stats(&game, first), (9, 9));
    act(&mut game, Action::AnnounceExile { card: witch });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(witch).zone, Zone::Battlefield);
    assert!(game.object_view(witch).keywords.contains(Keywords::MENACE));

    finish_turn(&mut game);
    reach_main(&mut game);
    for &card in &islands[..6] {
        act(&mut game, Action::TapMana { card });
    }
    for (index, creature) in [first, second].into_iter().enumerate() {
        act(
            &mut game,
            Action::AnnounceCast {
                card: witches[index],
            },
        );
        if index == 0 {
            let mut unpaid = game.clone();
            bargain(&mut unpaid, None);
            pay(&mut unpaid);
            settle(&mut unpaid);
            assert_eq!(stats(&unpaid, first), (9, 9));
            assert_eq!(unpaid.object_view(offerings[0]).zone, Zone::Battlefield);
        }
        bargain(&mut game, Some(offerings[index]));
        pay(&mut game);
        assert_eq!(game.object_view(offerings[index]).zone, Zone::Graveyard);
        pass_twice(&mut game);
        reject(&mut game, target(foreign));
        act(&mut game, target(creature));
        settle(&mut game);
    }

    assert_eq!(game.object_view(wicked.id).zone, Zone::Battlefield);
    assert_eq!(stats(&game, first), (6, 6));
    assert!(game.object_view(first).keywords.contains(Keywords::FLYING));
    assert_eq!(stats(&game, second), (3, 3));
    assert!(!game.object_view(second).keywords.contains(Keywords::FLYING));
    let curse = game
        .observe(player)
        .objects
        .into_iter()
        .find(|object| {
            object.is_token()
                && object.controller == opponent
                && object.attached_to == Some(game.object_ref(first))
        })
        .unwrap();

    finish_turn(&mut game);
    reach_main(&mut game);
    for card in plains {
        act(&mut game, Action::TapMana { card });
    }
    let draws = game.state.players[player.index()].cards_drawn_this_turn;
    let life = game.state.players[opponent.index()].life;
    cast(&mut game, clothier, None);
    pass_twice(&mut game);
    assert!(
        game.object_view(clothier)
            .keywords
            .contains(Keywords::FLYING)
    );
    reject(&mut game, target(clothier));
    act(&mut game, target(first));
    pass_twice(&mut game);
    if matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
    }
    settle(&mut game);
    assert_ne!(game.object_view(wicked.id).zone, Zone::Battlefield);
    assert_eq!(game.object_view(curse.id).reference, curse.reference);
    assert_eq!(game.object_view(curse.id).zone, Zone::Battlefield);
    assert_eq!(stats(&game, first), (6, 6));
    assert_eq!(
        game.state.players[player.index()].cards_drawn_this_turn,
        draws + 1
    );
    assert_eq!(game.state.players[opponent.index()].life, life - 1);

    act(&mut game, Action::Pass);
    act(&mut game, Action::TapMana { card: islands[6] });
    cast(&mut game, bounce, Some(Target::Card(first)));
    pass_twice(&mut game);
    assert_eq!(
        game.observe(opponent)
            .resolution_payment
            .unwrap()
            .cost
            .generic,
        1
    );
    act(&mut game, Action::DeclineResolutionMana);
    settle(&mut game);
    assert_eq!(game.object_view(bounce).zone, Zone::Graveyard);
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
}

#[test]
fn celebration_counts_paid_entries_even_after_an_entrant_leaves() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let ash = put(&mut game, player, "Ash, Party Crasher", Zone::Hand);
    let mice = put(&mut game, player, "Armory Mice", Zone::Hand);
    let belligerent = put(
        &mut game,
        player,
        "Belligerent of the Ball",
        Zone::Battlefield,
    );
    let land = put(&mut game, player, "Plains", Zone::Hand);
    let foreign = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let bounce = put(&mut game, opponent, "Unsummon", Zone::Hand);
    let mut mice_only = game.clone();
    cast(&mut mice_only, mice, None);
    settle(&mut mice_only);
    assert_eq!(stats(&mice_only, mice), (3, 1));
    cast(&mut game, ash, None);
    settle(&mut game);
    act(&mut game, Action::PlayLand { card: land });

    let mut unqualified = game.clone();
    reach_attackers(&mut unqualified);
    act(&mut unqualified, Action::Attack { card: ash });
    act(&mut unqualified, Action::FinishAttackers);
    settle(&mut unqualified);
    assert_eq!(unqualified.counter_count(ash, CounterKind::PlusOne), 0);
    assert!(
        !unqualified
            .object_view(ash)
            .keywords
            .contains(Keywords::MENACE)
    );

    cast(&mut game, mice, None);
    settle(&mut game);
    assert_eq!(stats(&game, mice), (3, 3));
    act(&mut game, Action::Pass);
    cast(&mut game, bounce, Some(Target::Card(mice)));
    settle(&mut game);
    assert_eq!(game.object_view(mice).zone, Zone::Hand);

    pass_twice(&mut game);
    assert_eq!(game.state.step, Step::BeginCombat);
    reject(&mut game, target(foreign));
    act(&mut game, target(ash));
    settle(&mut game);
    assert_eq!(stats(&game, ash), (3, 2));
    assert!(game.object_view(ash).keywords.contains(Keywords::MENACE));
    assert_eq!(stats(&game, belligerent), (3, 3));
    reach_attackers(&mut game);
    act(&mut game, Action::Attack { card: ash });
    act(&mut game, Action::FinishAttackers);
    settle(&mut game);
    assert_eq!(game.counter_count(ash, CounterKind::PlusOne), 1);
    assert_eq!(stats(&game, ash), (4, 3));
    finish_turn(&mut game);
    assert_eq!(stats(&game, ash), (3, 3));
    assert!(!game.object_view(ash).keywords.contains(Keywords::MENACE));
}

#[test]
fn cut_in_keeps_damage_and_young_hero_in_separate_target_groups() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let cut_in = put(&mut game, player, "Cut In", Zone::Hand);
    let dragon = put(&mut game, player, "Shivan Dragon", Zone::Battlefield);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foe = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    let growth = put(&mut game, player, "Giant Growth", Zone::Hand);
    let forest = put(&mut game, player, "Forest", Zone::Battlefield);
    act(&mut game, Action::AnnounceCast { card: cut_in });

    let mut same_target = game.clone();
    act(&mut same_target, target(dragon));
    act(&mut same_target, target(dragon));
    pay(&mut same_target);
    settle(&mut same_target);
    assert_eq!(same_target.object_view(dragon).damage, 4);
    assert!(same_target.observe(player).objects.iter().any(|object| {
        object.is_token() && object.attached_to == Some(same_target.object_ref(dragon))
    }));

    act(&mut game, target(foe));
    let mut damage_only = game.clone();
    act(&mut damage_only, Action::FinishTargets);
    pay(&mut damage_only);
    settle(&mut damage_only);
    assert_eq!(damage_only.object_view(foe).zone, Zone::Graveyard);
    assert!(
        damage_only
            .observe(player)
            .objects
            .iter()
            .all(|object| !object.is_token())
    );

    reject(&mut game, target(foe));
    act(&mut game, target(bear));
    pay(&mut game);
    cast(&mut game, bounce, Some(Target::Card(foe)));
    settle(&mut game);
    assert_eq!(game.object_view(foe).zone, Zone::Hand);
    assert!(
        game.observe(player).objects.iter().any(|object| {
            object.is_token() && object.attached_to == Some(game.object_ref(bear))
        })
    );

    reach_attackers(&mut game);
    act(&mut game, Action::Attack { card: bear });
    act(&mut game, Action::FinishAttackers);
    let mut too_tough = game.clone();
    act(&mut too_tough, Action::TapMana { card: forest });
    cast(&mut too_tough, growth, Some(Target::Card(bear)));
    settle(&mut too_tough);
    assert_eq!(too_tough.counter_count(bear, CounterKind::PlusOne), 0);

    settle(&mut game);
    assert_eq!(game.counter_count(bear, CounterKind::PlusOne), 1);
}

#[test]
fn fetch_quest_selects_exact_milled_cards_across_public_replacements() {
    let player = Player::P1;
    for destination in [Zone::Graveyard, Zone::Exile] {
        let mut game = scenario(player);
        let source = put(
            &mut game,
            player,
            "Bramble Familiar // Fetch Quest",
            Zone::Hand,
        );
        let old = put(&mut game, player, "Bear Cub", Zone::Graveyard);
        let creature = put(&mut game, player, "Bear Cub", Zone::Library);
        let enchantment = put(&mut game, player, "A Tale for the Ages", Zone::Library);
        let land = put(&mut game, player, "Forest", Zone::Library);
        let instant = put(&mut game, player, "Shock", Zone::Library);
        if destination == Zone::Exile {
            put(
                &mut game,
                player.other(),
                "Rest in Peace",
                Zone::Battlefield,
            );
        }
        let library_before = game.state.players[player.index()].library.len();
        act(&mut game, Action::AnnounceAdventure { card: source });
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 2, 5],
            },
        );
        pass_twice(&mut game);
        assert_eq!(
            game.state.players[player.index()].library.len(),
            library_before - 7
        );
        assert_eq!(game.state.pending, Pending::MilledSelection { player });
        assert_eq!(game.object_view(instant).zone, destination);
        reject(&mut game, Action::FinishMilledSelection);
        for invalid in [instant, old] {
            let object = game.object_ref(invalid);
            reject(&mut game, Action::SelectMilled { object });
        }

        let eligible = [creature, enchantment, land];
        for selected in eligible {
            let mut branch = game.clone();
            let object = branch.object_ref(selected);
            act(&mut branch, Action::SelectMilled { object });
            settle(&mut branch);
            for card in eligible {
                assert_eq!(
                    branch.object_view(card).zone,
                    if card == selected {
                        Zone::Battlefield
                    } else {
                        destination
                    }
                );
            }
            assert_eq!(branch.object_view(source).zone, Zone::Exile);
            assert!(
                branch
                    .legal_actions()
                    .contains(&Action::AnnounceExile { card: source })
            );
        }
    }
}

#[test]
fn bramble_familiar_pays_distinct_tap_abilities_and_returns_to_hand() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bramble = put(
        &mut game,
        player,
        "Bramble Familiar // Fetch Quest",
        Zone::Hand,
    );
    let discard = put(&mut game, player, "Opt", Zone::Hand);
    let forests = [(); 2].map(|_| put(&mut game, player, "Forest", Zone::Battlefield));
    cast(&mut game, bramble, None);
    settle(&mut game);
    let green = [0, 0, 0, 0, 1, 0];
    let mana = Action::ActivateMana {
        card: bramble,
        ability: 0,
        counter_payment: Counters::default(),
        payment: [0; 6],
        production: green,
    };
    let recover = activate(bramble, 1);
    reject(&mut game, mana.clone());
    reject(&mut game, recover.clone());
    finish_turn(&mut game);
    finish_turn(&mut game);
    reach_main(&mut game);

    let mut mana_branch = game.clone();
    act(&mut mana_branch, mana);
    assert_eq!(mana_branch.state.players[player.index()].mana, green);
    assert!(mana_branch.object_view(bramble).tapped);
    reject(&mut mana_branch, recover.clone());

    act(&mut game, recover);
    let object = game.object_ref(discard);
    act(&mut game, Action::ChooseAdditionalCost { object });
    for card in forests {
        act(&mut game, Action::TapMana { card });
    }
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 2, 0],
        },
    );
    assert!(game.object_view(bramble).tapped);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(bramble).zone, Zone::Hand);
}

fn ashiok(game: &mut Game, player: Player) -> CardId {
    let card = put(game, player, "Ashiok, Wicked Manipulator", Zone::Hand);
    cast(game, card, None);
    settle(game);
    assert_eq!(game.counter_count(card, CounterKind::Loyalty), 5);
    card
}

#[test]
fn ashiok_exiles_one_looked_card_publicly_and_keeps_the_other_private() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = ashiok(&mut game, player);
    let first = put(&mut game, player, "Shivan Dragon", Zone::Library);
    let second = put(&mut game, player, "Opt", Zone::Library);

    let mut single = game.clone();
    let library = single.state.players[player.index()].library.clone();
    for card in library.into_iter().filter(|&card| card != second) {
        single.move_object(card, Zone::Graveyard);
    }
    act(&mut single, activate(source, 0));
    pay(&mut single);
    settle(&mut single);
    assert_eq!(single.object_view(second).zone, Zone::Exile);
    assert!(single.state.players[player.index()].library.is_empty());

    act(&mut game, activate(source, 0));
    pay(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::Loyalty), 6);
    pass_twice(&mut game);
    reject(&mut game, Action::FinishLibrarySelection);
    let choice = game.observe(player).library_selection.unwrap();
    assert_eq!(
        choice
            .remaining
            .iter()
            .map(|view| view.id)
            .collect::<Vec<_>>(),
        [second, first]
    );
    assert!(game.observe(opponent).library_selection.is_none());
    let checkpoint = game.snapshot();
    let exile_first = Action::SelectLibrary { card: first };
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == exile_first)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, game.revision(), index).unwrap();
    let mut raw = game.clone();
    act(&mut raw, exile_first);
    assert_eq!(indexed.snapshot(), raw.snapshot());
    act(&mut game, Action::SelectLibrary { card: second });
    for (mut branch, exiled, kept) in [(raw, first, second), (game, second, first)] {
        act(&mut branch, Action::FinishLibrarySelection);
        settle(&mut branch);
        assert_eq!(branch.object_view(exiled).zone, Zone::Exile);
        assert_eq!(branch.object_view(kept).zone, Zone::Hand);
        let public = branch.observe(opponent);
        assert!(public.objects.iter().any(|view| view.id == exiled));
        assert!(public.objects.iter().all(|view| view.id != kept));
        reject(&mut branch, activate(source, 1));
    }
    indexed.restore(&checkpoint).unwrap();
    assert!(
        indexed
            .legal_actions()
            .contains(&Action::SelectLibrary { card: second })
    );
}

#[test]
fn ashiok_nightmares_grow_only_if_a_card_was_exiled_this_turn() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = ashiok(&mut game, player);
    let ooze = put(&mut game, opponent, "Scavenging Ooze", Zone::Battlefield);
    let card = put(&mut game, opponent, "Opt", Zone::Graveyard);
    act(&mut game, activate(source, 1));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::Loyalty), 3);
    let nightmares = game
        .observe(player)
        .objects
        .into_iter()
        .filter(|view| view.is_token() && view.has_subtype("Nightmare"))
        .map(|view| {
            assert_eq!((view.controller, view.colors), (player, 0b00100));
            assert_eq!((view.power, view.toughness), (1, 1));
            view.id
        })
        .collect::<Vec<_>>();
    assert_eq!(nightmares.len(), 2);
    let mut no_exile = game.clone();
    reach_attackers(&mut no_exile);
    for &id in &nightmares {
        assert_eq!(no_exile.counter_count(id, CounterKind::PlusOne), 0);
    }

    act(&mut game, Action::Pass);
    act(&mut game, activate(ooze, 0));
    act(&mut game, target(card));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(card).zone, Zone::Exile);
    pass_twice(&mut game);
    assert_eq!(game.state.step, Step::BeginCombat);
    if matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
    }
    settle(&mut game);
    for id in nightmares {
        assert_eq!(game.counter_count(id, CounterKind::PlusOne), 1);
        assert_eq!(stats(&game, id), (2, 2));
    }
}

#[test]
fn ashiok_ultimate_counts_owned_face_up_exile_mana_value_for_either_player() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = ashiok(&mut game, player);
    reject(&mut game, activate(source, 2));
    game.add_counter(source, CounterKind::Loyalty, 2);
    put(&mut game, player, "Bear Cub", Zone::Exile);
    put(&mut game, player, "Opt", Zone::Exile);
    put(&mut game, opponent, "Shivan Dragon", Zone::Exile);
    let hidden = put(&mut game, player, "Shivan Dragon", Zone::Exile);
    game.remember_face_down_exile(game.object_ref(hidden), 1 << player.index(), None);

    for victim in [player, opponent] {
        let mut branch = game.clone();
        let library = branch.state.players[victim.index()].library.clone();
        act(&mut branch, activate(source, 2));
        act(
            &mut branch,
            Action::ChooseTarget {
                target: Target::Player(victim),
            },
        );
        pay(&mut branch);
        settle(&mut branch);
        assert_eq!(branch.object_view(source).zone, Zone::Graveyard);
        assert_eq!(
            branch.state.players[victim.index()].library.len(),
            library.len() - 3
        );
        for &card in library.iter().rev().take(3) {
            assert_eq!(branch.object_view(card).zone, Zone::Exile);
        }
        assert_eq!(
            branch.state.players[victim.other().index()].library.len(),
            8
        );
    }
}
