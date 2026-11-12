//! Paid scenarios for the remaining green and multicolored Duskmourn cards.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn progress_until(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..120 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::FinishAttackers
                    | Action::FinishBlockers
                    | Action::PutTrigger { .. }
            )
        });
    }
    panic!("scenario exceeded its action budget");
}

fn next_main(game: &mut Game, player: Player) {
    let turn = game.state.turn;
    progress_until(game, |game| {
        game.state.turn > turn && game.state.active == player && game.state.step == Step::Main1
    });
}

fn attack(game: &mut Game, card: CardId) {
    progress_until(game, |game| game.state.step == Step::Attackers);
    act(game, Action::Attack { card });
    act(game, Action::FinishAttackers);
}

fn cast_named(game: &mut Game, player: Player, name: &str, target: Option<Target>) -> CardId {
    let card = put(game, player, name, Zone::Hand);
    cast(game, card, target);
    card
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn additional_cost(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

fn choose_additional_clause(game: &mut Game, player: Player) {
    if matches!(game.state.pending, Pending::CastCosts { .. }) {
        let clause = game
            .observe(player)
            .cast_costs
            .unwrap()
            .clauses
            .iter()
            .position(|row| row.clause == CastCostClause::Additional && !row.paid)
            .unwrap();
        act(
            game,
            Action::ChooseCastCost {
                clause: clause as u16,
            },
        );
    }
}

fn activation(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn finish_targets_and_pay(game: &mut Game) {
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

#[test]
fn beastie_beatdown_uses_delirium_and_can_counter_only_its_surviving_own_target() {
    let player = Player::P0;
    let mut game = scenario(player);
    let yours = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let theirs = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    for name in ["Forest", "Shock", "Short Bow"] {
        put(&mut game, player, name, Zone::Graveyard);
    }
    let spell = put(&mut game, player, "Beastie Beatdown", Zone::Hand);
    let targets = [Target::Card(yours), Target::Card(theirs)];
    let mut three = game.clone();
    announce(&mut three, spell, &targets);
    settle(&mut three);
    assert_eq!(three.counter_count(yours, CounterKind::PlusOne), 0);
    assert_eq!(three.object_view(theirs).damage, 2);

    put(&mut game, player, "Bear Cub", Zone::Graveyard);
    announce(&mut game, spell, &targets);
    let mut no_enemy = game.clone();
    cast_named(
        &mut no_enemy,
        player,
        "Unsummon",
        Some(Target::Card(theirs)),
    );
    settle(&mut no_enemy);
    assert_eq!(no_enemy.counter_count(yours, CounterKind::PlusOne), 2);
    assert_eq!(no_enemy.object_view(theirs).zone, Zone::Hand);
    let mut no_ally = game.clone();
    cast_named(&mut no_ally, player, "Unsummon", Some(Target::Card(yours)));
    settle(&mut no_ally);
    assert_eq!(no_ally.object_view(theirs).damage, 0);

    settle(&mut game);
    assert_eq!(game.counter_count(yours, CounterKind::PlusOne), 2);
    assert_eq!(game.object_view(theirs).zone, Zone::Graveyard);
}

#[test]
fn break_down_the_door_pays_and_resolves_all_three_modes_independently() {
    let player = Player::P1;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Break Down the Door", Zone::Hand);
    let artifact = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let enchantment = put(
        &mut game,
        player.other(),
        "Smoky Lounge // Misty Salon",
        Zone::Battlefield,
    );
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 2];
    for (mode, selected, other) in [(0, artifact, enchantment), (1, enchantment, artifact)] {
        let mut branch = game.clone();
        act(&mut branch, Action::AnnounceCast { card: spell });
        act(&mut branch, Action::ChooseMode { mode });
        reject(&mut branch, target(other));
        act(&mut branch, target(selected));
        finish_targets_and_pay(&mut branch);
        settle(&mut branch);
        assert_eq!(branch.state.players[player.index()].mana, [0; 6]);
        assert_eq!(branch.object_view(selected).zone, Zone::Exile);
        assert_eq!(branch.object_view(other).zone, Zone::Battlefield);
    }
    let milled = put(&mut game, player, "Shock", Zone::Library);
    let hidden = put(&mut game, player, "Bear Cub", Zone::Library);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::ChooseMode { mode: 2 });
    pay(&mut game);
    progress_until(&mut game, |game| {
        game.observe(player).manifest_choice.is_some()
    });
    let object = game.object_ref(hidden);
    act(&mut game, Action::ChooseManifest { object });
    settle(&mut game);
    assert!(game.is_face_down(game.object_ref(hidden)));
    assert_eq!(game.object_view(milled).zone, Zone::Graveyard);
    assert_eq!(game.object_view(artifact).zone, Zone::Battlefield);
    assert_eq!(game.object_view(enchantment).zone, Zone::Battlefield);
}

#[test]
fn clobbering_rechecks_each_untapped_target_before_tapping_and_dealing_damage() {
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let tapped = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.state.objects[tapped as usize].tapped = true;
    let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, player, "Coordinated Clobbering", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, target(tapped));
    act(&mut game, target(first));
    act(&mut game, target(second));
    act(&mut game, target(enemy));
    finish_targets_and_pay(&mut game);
    assert!(!game.object_view(first).tapped && !game.object_view(second).tapped);

    let mut ordinary = game.clone();
    settle(&mut ordinary);
    assert!(ordinary.object_view(first).tapped && ordinary.object_view(second).tapped);
    assert_eq!(ordinary.object_view(enemy).zone, Zone::Graveyard);
    act(&mut game, Action::TapMana { card: first });
    settle(&mut game);
    assert_eq!(game.object_view(enemy).zone, Zone::Battlefield);
    assert_eq!(game.object_view(enemy).damage, 2);
    assert!(game.object_view(second).tapped);
}

#[test]
fn fear_of_exposure_requires_two_distinct_cost_objects_besides_its_mana() {
    let player = Player::P1;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let creature = cast_named(&mut game, player, "Bear Cub", None);
    settle(&mut game);
    assert!(game.object_view(creature).summoning_sick);
    let artifact = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    let spell = put(&mut game, player, "Fear of Exposure", Zone::Hand);
    let announce = Action::AnnounceCast { card: spell };
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    reject(&mut game, announce.clone());
    let mana = put(&mut game, player, "Forest", Zone::Battlefield);
    act(&mut game, announce);
    act(&mut game, Action::TapMana { card: mana });
    pay(&mut game);
    choose_additional_clause(&mut game, player);
    let bad = [
        additional_cost(&game, artifact),
        additional_cost(&game, enemy),
        additional_cost(&game, mana),
    ];
    for choice in bad {
        reject(&mut game, choice);
    }
    let first = additional_cost(&game, land);
    act(&mut game, first.clone());
    reject(&mut game, first);
    let second = additional_cost(&game, creature);
    act(&mut game, second);
    finish_targets_and_pay(&mut game);
    assert!(game.object_view(land).tapped && game.object_view(creature).tapped);
    assert!(game.state.stack.last().unwrap().targets.is_empty());
    settle(&mut game);
    assert_eq!(stats(&game, spell), (5, 4));
    assert!(game.object_view(spell).keywords.contains(Keywords::TRAMPLE));
}

#[test]
fn cartographer_reveals_only_through_the_first_land_and_bottoms_the_misses() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = cast_named(&mut game, player, "House Cartographer", None);
    settle(&mut game);
    next_main(&mut game, player);
    let deeper = put(&mut game, player, "Forest", Zone::Library);
    let land = put(&mut game, player, "Island", Zone::Library);
    let misses = ["Bear Cub", "Shock"].map(|name| put(&mut game, player, name, Zone::Library));
    attack(&mut game, source);
    progress_until(&mut game, |game| {
        game.observe(player).library_selection.is_some()
    });
    assert_eq!(game.state.step, Step::Main2);
    let public = game.observe(player.other()).library_selection.unwrap();
    assert_eq!(
        public
            .remaining
            .iter()
            .map(|view| view.id)
            .collect::<Vec<_>>(),
        [misses[1], misses[0], land]
    );
    reject(&mut game, Action::SelectLibrary { card: deeper });
    reject(&mut game, Action::FinishLibrarySelection);
    act(&mut game, Action::SelectLibrary { card: land });
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut game, Action::FinishLibrarySelection);
    }
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Hand);
    assert!(
        game.observe(player.other())
            .known_hand
            .contains(&game.object_ref(land))
    );
    assert_eq!(
        game.state.players[player.index()].library.last(),
        Some(&deeper)
    );
    assert!(
        misses
            .iter()
            .all(|card| game.state.players[player.index()].library[..2].contains(card))
    );
}

#[test]
fn leyline_opens_from_hand_and_pays_wubrg_for_an_expensive_spell() {
    let player = Player::P0;
    let leyline = Card::from_name("Leyline of Mutation").unwrap();
    let mut game = Game::new(Config {
        decks: [
            [vec![leyline], vec![Card::Forest; 6]].concat(),
            vec![Card::Forest; 7],
        ],
        seed: 20261023,
        starting_player: player,
        deck_rules: DeckRules::Freeform,
    })
    .unwrap();
    act(&mut game, Action::Keep);
    act(&mut game, Action::Keep);
    let begin = game
        .legal_actions()
        .into_iter()
        .find(|action| matches!(action, Action::BeginOnBattlefield { .. }))
        .unwrap();
    let Action::BeginOnBattlefield { object } = begin else {
        unreachable!()
    };
    let mut declined = game.clone();
    act(&mut declined, Action::FinishOpeningHand);
    act(&mut declined, Action::FinishOpeningHand);
    assert_eq!(declined.object_view(object.card).zone, Zone::Hand);
    assert!(declined.observe(player.other()).known_hand.is_empty());

    act(&mut game, begin);
    act(&mut game, Action::FinishOpeningHand);
    act(&mut game, Action::FinishOpeningHand);
    assert_eq!(game.object_view(object.card).zone, Zone::Battlefield);
    progress_until(&mut game, |game| game.state.step == Step::Main1);
    let colors = [1, 1, 1, 1, 1, 0];
    game.state.players[player.index()].mana = colors;
    let spell = put(&mut game, player, "Altanak, the Thrice-Called", Zone::Hand);
    reject(&mut game, Action::AnnounceCast { card: spell });
    let source = game.object_ref(object.card);
    choose(&mut game, |action| {
        matches!(action, Action::AnnounceCastWithManaAlternative { object, provider, cost, .. }
        if object.card == spell && *provider == source && cost.colored == colors && cost.generic == 0)
    });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
    assert_eq!(stats(&game, spell), (9, 9));
}

#[test]
fn monstrous_emergence_uses_a_private_hand_reveal_or_the_chosen_creatures_live_power() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let chosen = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let victim = put(
        &mut game,
        opponent,
        "Ghalta, Primal Hunger",
        Zone::Battlefield,
    );
    let revealed = put(&mut game, player, "Hill Giant", Zone::Hand);
    let private = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    let spell = put(&mut game, player, "Monstrous Emergence", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, target(victim));
    finish_targets_and_pay(&mut game);
    choose_additional_clause(&mut game, player);
    assert!(game.observe(opponent).additional_cost.is_none());
    let reveal = additional_cost(&game, revealed);
    let choose_creature = additional_cost(&game, chosen);
    let mut hand = game.clone();
    act(&mut hand, reveal);
    assert!(
        hand.observe(opponent)
            .known_hand
            .contains(&hand.object_ref(revealed))
    );
    assert!(
        !hand
            .observe(opponent)
            .known_hand
            .contains(&hand.object_ref(private))
    );
    settle(&mut hand);
    assert_eq!(hand.object_view(victim).damage, 3);
    assert_eq!(hand.object_view(revealed).zone, Zone::Hand);

    act(&mut game, choose_creature);
    assert_eq!(game.state.stack.last().unwrap().targets.len(), 1);
    assert_eq!(game.object_view(chosen).zone, Zone::Battlefield);
    cast_named(
        &mut game,
        player,
        "Giant Growth",
        Some(Target::Card(chosen)),
    );
    settle(&mut game);
    assert_eq!(game.object_view(victim).damage, 5);
    assert!(
        !game
            .observe(opponent)
            .known_hand
            .contains(&game.object_ref(revealed))
    );
}

fn mana_ability(card: CardId, ability: u16, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability,
        counter_payment: Counters::default(),
        payment: [0; 6],
        production,
    }
}

#[test]
fn overgrown_zealot_can_fund_a_spell_or_produce_two_mana_for_turning_face_up() {
    let player = Player::P0;
    let mut game = scenario(player);
    let zealot = cast_named(&mut game, player, "Overgrown Zealot", None);
    settle(&mut game);
    assert_eq!(stats(&game, zealot), (0, 4));
    next_main(&mut game, player);
    let spell = put(&mut game, player, "Bear Cub", Zone::Hand);
    put(&mut game, player, "Forest", Zone::Library);
    let manifested = put(&mut game, player, "Grizzly Bears", Zone::Library);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 1];
    cast_named(&mut game, player, "Manifest Dread", None);
    progress_until(&mut game, |game| {
        game.observe(player).manifest_choice.is_some()
    });
    let object = game.object_ref(manifested);
    act(&mut game, Action::ChooseManifest { object });
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);

    let mut ordinary = game.clone();
    let opt = put(&mut ordinary, player, "Opt", Zone::Hand);
    act(&mut ordinary, Action::AnnounceCast { card: opt });
    act(&mut ordinary, mana_ability(zealot, 0, [0, 1, 0, 0, 0, 0]));
    pay(&mut ordinary);
    assert_eq!(ordinary.object_view(opt).zone, Zone::Stack);
    progress_until(&mut ordinary, |game| {
        game.observe(player).library_choice.is_some()
    });
    choose(&mut ordinary, |action| {
        matches!(
            action,
            Action::ArrangeLibrary {
                destination: LibraryDestination::Top,
                ..
            }
        )
    });
    settle(&mut ordinary);

    let reference = game.object_ref(manifested);
    let two_green = [0, 0, 0, 0, 2, 0];
    let produce = mana_ability(zealot, 1, two_green);
    let begin = Action::BeginFaceUp {
        object: reference,
        cost: ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 1,
        },
    };
    let mut prefunded = game.clone();
    act(&mut prefunded, produce.clone());
    reject(&mut prefunded, Action::AnnounceCast { card: spell });
    act(&mut prefunded, begin.clone());
    act(&mut prefunded, Action::PayFaceUp { payment: two_green });
    act(&mut game, begin);
    act(&mut game, produce);
    act(&mut game, Action::PayFaceUp { payment: two_green });
    for result in [&game, &prefunded] {
        assert!(!result.is_face_down(reference));
        assert_eq!(result.object_ref(manifested), reference);
        assert_eq!(result.state.players[player.index()].mana, [0; 6]);
    }
}

#[test]
fn rip_reveals_its_power_and_takes_only_distinct_creature_or_vehicle_powers() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = cast_named(&mut game, player, "Rip, Spawn Hunter", None);
    settle(&mut game);
    next_main(&mut game, player);
    let untouched = put(&mut game, player, "Forest", Zone::Library);
    let vehicle = put(&mut game, player, "Thunderous Velocipede", Zone::Library);
    let land = put(&mut game, player, "Island", Zone::Library);
    let duplicate = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let first = put(&mut game, player, "Bear Cub", Zone::Library);
    attack(&mut game, source);
    progress_until(&mut game, |game| {
        game.observe(player).library_selection.is_some()
    });
    assert_eq!(game.state.step, Step::Main2);
    assert_eq!(
        game.observe(player.other())
            .library_selection
            .unwrap()
            .remaining
            .len(),
        4
    );
    let mut declined = game.clone();
    act(&mut declined, Action::FinishLibrarySelection);
    for card in [vehicle, land, duplicate, first] {
        assert!(declined.state.players[player.index()].library[..4].contains(&card));
    }
    reject(&mut game, Action::SelectLibrary { card: land });
    reject(&mut game, Action::SelectLibrary { card: untouched });
    act(&mut game, Action::SelectLibrary { card: first });
    reject(&mut game, Action::SelectLibrary { card: duplicate });
    act(&mut game, Action::SelectLibrary { card: vehicle });
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut game, Action::FinishLibrarySelection);
    }
    settle(&mut game);
    for card in [first, vehicle] {
        assert_eq!(game.object_view(card).zone, Zone::Hand);
    }
    for card in [duplicate, land] {
        assert!(game.state.players[player.index()].library[..2].contains(&card));
    }
}

#[test]
fn sawblade_counts_paid_sacrifices_through_resolution_of_its_end_step_trigger() {
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enchantment = put(&mut game, player, "Doubling Season", Zone::Battlefield);
    let source = cast_named(&mut game, player, "Sawblade Skinripper", None);
    settle(&mut game);
    assert!(game.object_view(source).keywords.contains(Keywords::MENACE));
    let mut no_sacrifice = game.clone();
    progress_until(&mut no_sacrifice, |game| game.state.step == Step::End);
    assert!(no_sacrifice.state.stack.is_empty());
    assert_eq!(no_sacrifice.state.pending, Pending::Priority);

    act(&mut game, activation(source, 0));
    let invalid = additional_cost(&game, source);
    reject(&mut game, invalid);
    let sacrifice = additional_cost(&game, enchantment);
    act(&mut game, sacrifice);
    finish_targets_and_pay(&mut game);
    assert_eq!(game.object_view(enchantment).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);

    progress_until(&mut game, |game| game.state.step == Step::End);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    act(&mut game, activation(source, 0));
    let sacrifice = additional_cost(&game, creature);
    act(&mut game, sacrifice);
    finish_targets_and_pay(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 2);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
    assert_eq!(
        game.observe(player).players[player.other().index()].life,
        18
    );
}

#[test]
fn tyvar_pays_with_another_new_creature_then_samples_the_largest_remaining_power() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let largest = put(
        &mut game,
        player,
        "Ghalta, Primal Hunger",
        Zone::Battlefield,
    );
    let giant = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let tyvar = cast_named(&mut game, player, "Tyvar, the Pummeler", None);
    settle(&mut game);
    let cub = cast_named(&mut game, player, "Bear Cub", None);
    settle(&mut game);
    assert!(game.object_view(tyvar).summoning_sick && game.object_view(cub).summoning_sick);
    act(&mut game, activation(tyvar, 0));
    let source = additional_cost(&game, tyvar);
    reject(&mut game, source);
    let cost = additional_cost(&game, cub);
    act(&mut game, cost);
    finish_targets_and_pay(&mut game);
    assert!(game.object_view(cub).tapped);
    assert!(!game.object_view(tyvar).tapped);
    settle(&mut game);
    assert!(game.object_view(tyvar).tapped);
    assert!(
        game.object_view(tyvar)
            .keywords
            .contains(Keywords::INDESTRUCTIBLE)
    );
    cast_named(&mut game, player, "Murder", Some(Target::Card(tyvar)));
    settle(&mut game);
    assert_eq!(game.object_view(tyvar).zone, Zone::Battlefield);

    game.state.players[player.index()].mana = [0, 0, 0, 0, 2, 3];
    act(&mut game, activation(tyvar, 1));
    finish_targets_and_pay(&mut game);
    act(&mut game, Action::Pass);
    cast_named(&mut game, opponent, "Unsummon", Some(Target::Card(largest)));
    settle(&mut game);
    assert_eq!(game.object_view(largest).zone, Zone::Hand);
    assert_eq!(stats(&game, tyvar), (6, 6));
    assert_eq!(stats(&game, giant), (6, 6));
    assert_eq!(stats(&game, cub), (5, 5));
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 0];
    let later = cast_named(&mut game, player, "Llanowar Elves", None);
    settle(&mut game);
    assert_eq!(stats(&game, later), (1, 1));
}
