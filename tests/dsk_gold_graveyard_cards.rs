//! Paid scenarios for the green and multicolored Duskmourn graveyard cards.
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

fn settle_ordered(game: &mut Game) {
    progress_until(game, |game| {
        game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.resolving.is_none()
    });
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

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn cast_named(game: &mut Game, player: Player, name: &str, target: Option<Target>) -> CardId {
    let card = put(game, player, name, Zone::Hand);
    cast(game, card, target);
    card
}

fn activation(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
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

fn exact_cost(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

fn seed_three_types(game: &mut Game, player: Player) {
    for name in ["Forest", "Shock", "Short Bow"] {
        put(game, player, name, Zone::Graveyard);
    }
}

#[test]
fn altanak_discards_for_own_land_and_draws_when_an_opponent_targets_it() {
    let player = Player::P0;
    let mut game = scenario(player);
    let altanak = put(&mut game, player, "Altanak, the Thrice-Called", Zone::Hand);
    let land = put(&mut game, player, "Island", Zone::Graveyard);
    let foreign = put(&mut game, player.other(), "Island", Zone::Graveyard);
    let nonland = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let mut hand_ability = game.clone();
    hand_ability.state.players[player.index()].mana = [0, 0, 0, 0, 1, 1];
    act(&mut hand_ability, activation(altanak));
    reject(&mut hand_ability, target(foreign));
    reject(&mut hand_ability, target(nonland));
    act(&mut hand_ability, target(land));
    finish_targets_and_pay(&mut hand_ability);
    assert_eq!(hand_ability.object_view(altanak).zone, Zone::Graveyard);
    assert_eq!(hand_ability.object_view(land).zone, Zone::Graveyard);
    assert_eq!(hand_ability.state.players[player.index()].mana, [0; 6]);
    settle(&mut hand_ability);
    assert_eq!(hand_ability.object_view(land).zone, Zone::Battlefield);
    assert!(hand_ability.object_view(land).tapped);

    cast(&mut game, altanak, None);
    settle(&mut game);
    assert_eq!(stats(&game, altanak), (9, 9));
    assert!(
        game.object_view(altanak)
            .keywords
            .contains(Keywords::TRAMPLE)
    );
    let drawn = put(&mut game, player, "Forest", Zone::Library);
    cast_named(
        &mut game,
        player,
        "Giant Growth",
        Some(Target::Card(altanak)),
    );
    settle(&mut game);
    assert_eq!(game.object_view(drawn).zone, Zone::Library);
    act(&mut game, Action::Pass);
    cast_named(
        &mut game,
        player.other(),
        "Shock",
        Some(Target::Card(altanak)),
    );
    settle(&mut game);
    assert_eq!(game.object_view(drawn).zone, Zone::Hand);
}

#[test]
fn balustrade_wurm_resists_counters_and_returns_with_finality_only_at_delirium() {
    let player = Player::P1;
    let mut normal = scenario(player);
    let wurm = cast_named(&mut normal, player, "Balustrade Wurm", None);
    act(&mut normal, Action::Pass);
    let counter = cast_named(
        &mut normal,
        player.other(),
        "Cancel",
        Some(Target::Spell(wurm)),
    );
    settle(&mut normal);
    assert_eq!(normal.object_view(counter).zone, Zone::Graveyard);
    assert_eq!(normal.object_view(wurm).zone, Zone::Battlefield);
    assert!(
        normal
            .object_view(wurm)
            .keywords
            .contains(Keywords::HASTE.union(Keywords::TRAMPLE))
    );
    assert_eq!(normal.counter_count(wurm, CounterKind::Finality), 0);

    let mut returned = scenario(player);
    let wurm = put(&mut returned, player, "Balustrade Wurm", Zone::Graveyard);
    put(&mut returned, player, "Forest", Zone::Graveyard);
    put(&mut returned, player, "Shock", Zone::Graveyard);
    returned.state.players[player.index()].mana = [0, 0, 0, 0, 2, 2];
    reject(&mut returned, activation(wurm));
    put(&mut returned, player, "Short Bow", Zone::Graveyard);
    act(&mut returned, activation(wurm));
    finish_targets_and_pay(&mut returned);
    settle(&mut returned);
    assert_eq!(returned.state.players[player.index()].mana, [0; 6]);
    assert_eq!(returned.object_view(wurm).zone, Zone::Battlefield);
    assert_eq!(returned.counter_count(wurm, CounterKind::Finality), 1);
    assert!(!returned.object_view(wurm).tapped);
    returned.state.players[player.index()].mana = [0, 0, 2, 0, 0, 1];
    cast_named(&mut returned, player, "Murder", Some(Target::Card(wurm)));
    settle(&mut returned);
    assert_eq!(returned.object_view(wurm).zone, Zone::Exile);
}

#[test]
fn cathartic_parting_uses_the_permanent_owner_and_a_separate_optional_shuffle() {
    let player = Player::P0;
    let mut game = scenario(player);
    let stolen = put(&mut game, player, "Short Bow", Zone::Battlefield);
    game.record_control(game.object_ref(stolen), player.other(), None);
    let friendly = put(&mut game, player, "Doubling Season", Zone::Battlefield);
    let first = put(&mut game, player, "Shock", Zone::Graveyard);
    let second = put(&mut game, player, "Island", Zone::Graveyard);
    let foreign = put(&mut game, player.other(), "Forest", Zone::Graveyard);
    let spell = put(&mut game, player, "Cathartic Parting", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, target(friendly));
    act(&mut game, target(stolen));
    reject(&mut game, target(foreign));
    act(&mut game, target(first));
    act(&mut game, target(second));
    finish_targets_and_pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.object_view(stolen).zone, Zone::Library);
    assert!(game.state.players[player.index()].library.contains(&stolen));
    assert!(
        !game.state.players[player.other().index()]
            .library
            .contains(&stolen)
    );
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    for card in [first, second] {
        assert_eq!(declined.object_view(card).zone, Zone::Graveyard);
    }
    act(&mut game, Action::ChooseOptional { accept: true });
    settle(&mut game);
    for card in [first, second] {
        assert_eq!(game.object_view(card).zone, Zone::Library);
        assert!(game.state.players[player.index()].library.contains(&card));
    }
}

#[test]
fn disturbing_mirth_draws_for_another_sacrifice_and_manifests_on_its_own_sacrifice() {
    let player = Player::P1;
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let first = cast_named(&mut game, player, "Disturbing Mirth", None);
    progress_until(&mut game, |game| {
        game.observe(player).resolution_payment.is_some()
    });
    for card in [enemy, first] {
        let choice = exact_cost(&game, card);
        reject(&mut game, choice);
    }
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    assert_eq!(
        declined.state.players[player.index()].cards_drawn_this_turn,
        0
    );
    cast_named(
        &mut declined,
        player,
        "Disenchant",
        Some(Target::Card(first)),
    );
    settle(&mut declined);
    assert_eq!(declined.object_view(first).zone, Zone::Graveyard);
    assert!(declined.observe(player).last_manifest_dread.is_none());

    let choice = exact_cost(&game, creature);
    act(&mut game, choice);
    act(&mut game, Action::PayResolutionMana { payment: [0; 6] });
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);

    put(
        &mut game,
        player.other(),
        "Rest in Peace",
        Zone::Battlefield,
    );
    let unchosen = put(&mut game, player, "Shock", Zone::Library);
    let manifested = put(&mut game, player, "Bear Cub", Zone::Library);
    put(&mut game, player, "Forest", Zone::Library);
    put(&mut game, player, "Forest", Zone::Library);
    let second = cast_named(&mut game, player, "Disturbing Mirth", None);
    progress_until(&mut game, |game| {
        game.observe(player).resolution_payment.is_some()
    });
    let choice = exact_cost(&game, first);
    act(&mut game, choice);
    act(&mut game, Action::PayResolutionMana { payment: [0; 6] });
    progress_until(&mut game, |game| {
        game.observe(player).manifest_choice.is_some()
    });
    assert_eq!(game.object_view(first).zone, Zone::Exile);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 4);
    let object = game.object_ref(manifested);
    act(&mut game, Action::ChooseManifest { object });
    settle(&mut game);
    assert!(game.is_face_down(game.object_ref(manifested)));
    assert_eq!(game.object_view(unchosen).zone, Zone::Exile);
    assert_eq!(game.object_view(second).zone, Zone::Battlefield);
}

#[test]
fn drag_to_the_roots_costs_two_less_only_with_four_actual_card_types() {
    let player = Player::P0;
    let mut game = scenario(player);
    seed_three_types(&mut game, player);
    let enemy = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let land = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    let spell = put(&mut game, player, "Drag to the Roots", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 1, 0, 1, 0];
    reject(&mut game, Action::AnnounceCast { card: spell });
    put(&mut game, player, "Oaken Siren", Zone::Graveyard);
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, target(land));
    act(&mut game, target(enemy));
    finish_targets_and_pay(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    settle(&mut game);
    assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
}

#[test]
fn fear_of_infinity_can_return_on_entry_and_full_unlock_then_attack_with_lifelink() {
    let player = Player::P0;
    let mut game = scenario(player);
    let fear = put(&mut game, player, "Fear of Infinity", Zone::Graveyard);
    cast_named(&mut game, player, "Doubling Season", None);
    progress_until(&mut game, |game| {
        matches!(game.state.pending, Pending::EffectOptional { .. })
    });
    let mut entered = game.clone();
    act(&mut entered, Action::ChooseOptional { accept: true });
    settle(&mut entered);
    assert_eq!(entered.object_view(fear).zone, Zone::Hand);
    act(&mut game, Action::ChooseOptional { accept: false });
    settle(&mut game);

    let room = put(&mut game, player, "Smoky Lounge // Misty Salon", Zone::Hand);
    act(
        &mut game,
        Action::AnnounceRoom {
            card: room,
            door: RoomDoor::Left,
            flash_surcharge: false,
        },
    );
    pay(&mut game);
    progress_until(&mut game, |game| {
        matches!(game.state.pending, Pending::EffectOptional { .. })
    });
    act(&mut game, Action::ChooseOptional { accept: false });
    settle(&mut game);
    let object = game.object_ref(room);
    act(
        &mut game,
        Action::BeginRoomUnlock {
            object,
            door: RoomDoor::Right,
            cost: ManaCost {
                colored: [0, 1, 0, 0, 0, 0],
                generic: 3,
            },
        },
    );
    choose(&mut game, |action| {
        matches!(action, Action::PayRoomUnlock { .. })
    });
    progress_until(&mut game, |game| {
        matches!(game.state.pending, Pending::EffectOptional { .. })
    });
    act(&mut game, Action::ChooseOptional { accept: true });
    settle_ordered(&mut game);
    assert_eq!(game.object_view(fear).zone, Zone::Hand);

    cast(&mut game, fear, None);
    settle(&mut game);
    let keywords = game.object_view(fear).keywords;
    assert!(
        keywords.contains(
            Keywords::FLYING
                .union(Keywords::LIFELINK)
                .union(Keywords::CANT_BLOCK)
        )
    );
    next_main(&mut game, player);
    attack(&mut game, fear);
    progress_until(&mut game, |game| game.state.step == Step::Main2);
    assert_eq!(game.observe(player).players[player.index()].life, 22);
    assert_eq!(
        game.observe(player).players[player.other().index()].life,
        18
    );
}

#[test]
fn hedge_shredder_crews_and_recovers_only_the_exact_remaining_milled_land() {
    let player = Player::P0;
    let mut game = scenario(player);
    let shredder = cast_named(&mut game, player, "Hedge Shredder", None);
    settle(&mut game);
    assert!(!game.object_view(shredder).is_creature());
    next_main(&mut game, player);
    let crew = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    let ooze = put(
        &mut game,
        player.other(),
        "Scavenging Ooze",
        Zone::Battlefield,
    );
    let older = put(&mut game, player, "Forest", Zone::Graveyard);
    let returned = put(&mut game, player, "Island", Zone::Library);
    let lost = put(&mut game, player, "Forest", Zone::Library);
    act(&mut game, activation(shredder));
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    let object = game.object_ref(crew);
    act(&mut game, Action::ChooseCrewer { object });
    act(&mut game, Action::FinishCrew);
    settle(&mut game);
    assert_eq!(stats(&game, shredder), (5, 5));
    assert!(game.object_view(crew).tapped);
    attack(&mut game, shredder);
    progress_until(&mut game, |game| {
        matches!(game.state.pending, Pending::EffectOptional { .. })
    });
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(declined.object_view(lost).zone, Zone::Library);
    assert_eq!(declined.object_view(returned).zone, Zone::Library);

    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.object_view(lost).zone, Zone::Graveyard);
    assert_eq!(game.object_view(returned).zone, Zone::Graveyard);
    assert_eq!(game.state.stack.len(), 1);
    assert!(game.state.stack[0].targets.is_empty());
    act(&mut game, Action::Pass);
    game.state.players[player.other().index()].mana = [0, 0, 0, 0, 1, 0];
    act(&mut game, activation(ooze));
    act(&mut game, target(lost));
    finish_targets_and_pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(lost).zone, Zone::Exile);
    assert_eq!(game.object_view(older).zone, Zone::Graveyard);
    assert_eq!(game.object_view(returned).zone, Zone::Battlefield);
    assert!(game.object_view(returned).tapped);
}

#[test]
fn nashi_mills_actual_combat_damage_and_counts_only_cards_moved_to_hand() {
    let player = Player::P1;
    let mut game = scenario(player);
    let nashi = cast_named(&mut game, player, "Nashi, Searcher in the Dark", None);
    settle(&mut game);
    assert!(game.object_view(nashi).keywords.contains(Keywords::MENACE));
    next_main(&mut game, player);
    let old = put(&mut game, player, "Banishing Light", Zone::Graveyard);
    let outside_batch = put(&mut game, player, "Shock", Zone::Library);
    let enchantment = put(&mut game, player, "Doubling Season", Zone::Library);
    let legendary = put(&mut game, player, "Ghalta, Primal Hunger", Zone::Library);
    attack(&mut game, nashi);
    progress_until(&mut game, |game| {
        game.observe(player).milled_choice.is_some()
    });
    assert_eq!(
        game.observe(player).players[player.other().index()].life,
        18
    );
    assert_eq!(game.object_view(outside_batch).zone, Zone::Library);
    assert_eq!(game.object_view(enchantment).zone, Zone::Graveyard);
    assert_eq!(game.object_view(legendary).zone, Zone::Graveyard);
    let old_ref = game.object_ref(old);
    reject(&mut game, Action::SelectMilled { object: old_ref });
    let mut none = game.clone();
    act(&mut none, Action::FinishMilledSelection);
    assert_eq!(none.counter_count(nashi, CounterKind::PlusOne), 1);
    for card in [enchantment, legendary] {
        let object = game.object_ref(card);
        act(&mut game, Action::SelectMilled { object });
    }
    settle(&mut game);
    assert_eq!(game.object_view(enchantment).zone, Zone::Hand);
    assert_eq!(game.object_view(legendary).zone, Zone::Hand);
    assert_eq!(game.counter_count(nashi, CounterKind::PlusOne), 0);
}

#[test]
fn flytrap_announces_divisions_and_doubles_the_new_totals_at_six_types() {
    let player = Player::P0;
    let mut game = scenario(player);
    seed_three_types(&mut game, player);
    let flytrap = put(&mut game, player, "Omnivorous Flytrap", Zone::Hand);
    let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let ooze = put(
        &mut game,
        player.other(),
        "Scavenging Ooze",
        Zone::Battlefield,
    );
    let mut three = game.clone();
    cast(&mut three, flytrap, None);
    settle(&mut three);
    assert_eq!(three.counter_count(first, CounterKind::PlusOne), 0);

    let fourth = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    cast(&mut game, flytrap, None);
    pass_pair(&mut game);
    act(&mut game, target(first));
    act(&mut game, target(second));
    finish_targets_and_pay(&mut game);
    reject(&mut game, Action::AllocateTargetAmount { amount: 0 });
    act(&mut game, Action::AllocateTargetAmount { amount: 1 });
    act(&mut game, Action::AllocateTargetAmount { amount: 1 });
    let mut below_delirium = game.clone();
    act(&mut below_delirium, Action::Pass);
    act(&mut below_delirium, activation(ooze));
    act(&mut below_delirium, target(fourth));
    finish_targets_and_pay(&mut below_delirium);
    settle(&mut below_delirium);
    assert_eq!(below_delirium.object_view(fourth).zone, Zone::Exile);
    assert_eq!(below_delirium.counter_count(first, CounterKind::PlusOne), 0);
    assert_eq!(
        below_delirium.counter_count(second, CounterKind::PlusOne),
        0
    );
    settle(&mut game);
    assert_eq!(game.counter_count(first, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(second, CounterKind::PlusOne), 1);

    next_main(&mut game, player);
    put(&mut game, player, "Doubling Season", Zone::Graveyard);
    put(&mut game, player, "Terror Tide", Zone::Graveyard);
    attack(&mut game, flytrap);
    act(&mut game, target(first));
    finish_targets_and_pay(&mut game);
    act(&mut game, Action::AllocateTargetAmount { amount: 2 });
    settle_ordered(&mut game);
    assert_eq!(game.counter_count(first, CounterKind::PlusOne), 6);
    assert_eq!(game.counter_count(second, CounterKind::PlusOne), 1);
}

#[test]
fn patchwork_beastie_can_decline_mill_and_requires_delirium_for_combat() {
    let player = Player::P1;
    let mut game = scenario(player);
    seed_three_types(&mut game, player);
    let beastie = cast_named(&mut game, player, "Patchwork Beastie", None);
    settle(&mut game);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let attacker = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    next_main(&mut game, player.other());
    attack(&mut game, attacker);
    progress_until(&mut game, |game| game.state.step == Step::Blockers);
    reject(
        &mut game,
        Action::Block {
            blocker: beastie,
            attacker,
        },
    );
    act(&mut game, Action::FinishBlockers);

    let top = put(&mut game, player, "Bear Cub", Zone::Library);
    progress_until(&mut game, |game| {
        matches!(game.state.pending, Pending::EffectOptional { .. })
    });
    assert_eq!(game.state.step, Step::Upkeep);
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(declined.object_view(top).zone, Zone::Library);
    progress_until(&mut declined, |game| game.state.step == Step::Attackers);
    reject(&mut declined, Action::Attack { card: beastie });

    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
    attack(&mut game, beastie);
    assert!(game.object_view(beastie).attacking);
}

#[test]
fn rite_of_the_moth_returns_its_targets_with_finality_and_exiles_after_flashback() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Rite of the Moth", Zone::Hand);
    let first = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let second = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let wrong = put(&mut game, player.other(), "Bear Cub", Zone::Graveyard);
    game.state.players[player.index()].mana = [1, 0, 2, 0, 0, 1];
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, target(wrong));
    act(&mut game, target(first));
    finish_targets_and_pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(first, CounterKind::Finality), 1);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);

    game.state.players[player.index()].mana = [0, 0, 2, 0, 0, 1];
    cast_named(&mut game, player, "Murder", Some(Target::Card(first)));
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Exile);
    game.state.players[player.index()].mana = [2, 0, 1, 0, 0, 3];
    act(&mut game, Action::AnnounceFlashback { card: spell });
    act(&mut game, target(second));
    finish_targets_and_pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(game.object_view(second).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(second, CounterKind::Finality), 1);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
}

#[test]
fn say_its_name_mills_then_recovers_and_pays_exact_three_card_search_cost() {
    let player = Player::P1;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Say Its Name", Zone::Hand);
    let copies = [(); 2].map(|_| put(&mut game, player, "Say Its Name", Zone::Graveyard));
    let wrong = put(&mut game, player.other(), "Say Its Name", Zone::Graveyard);
    let altanak = put(&mut game, player, "Altanak, the Thrice-Called", Zone::Hand);
    let picked = put(&mut game, player, "Forest", Zone::Library);
    let creature = put(&mut game, player, "Bear Cub", Zone::Library);
    let instant = put(&mut game, player, "Shock", Zone::Library);
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    for card in [picked, creature, instant] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    let object = game.object_ref(instant);
    reject(&mut game, Action::SelectGraveyardReturn { object });
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(declined.object_view(picked).zone, Zone::Graveyard);
    let object = game.object_ref(picked);
    act(&mut game, Action::SelectGraveyardReturn { object });
    settle(&mut game);
    assert_eq!(game.object_view(picked).zone, Zone::Hand);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);

    act(&mut game, activation(spell));
    for card in [spell, wrong] {
        let choice = exact_cost(&game, card);
        reject(&mut game, choice);
    }
    for card in copies {
        let choice = exact_cost(&game, card);
        act(&mut game, choice);
    }
    finish_targets_and_pay(&mut game);
    for card in [spell, copies[0], copies[1]] {
        assert_eq!(game.object_view(card).zone, Zone::Exile);
    }
    assert_eq!(game.state.stack.len(), 1);
    let before = game.state.players[player.index()].library.clone();
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseGraveyardHandLibrarySearchZones {
            graveyard: false,
            hand: true,
            library: false,
        },
    );
    assert!(
        game.observe(player.other())
            .hand_library_search
            .unwrap()
            .cards
            .is_none()
    );
    act(&mut game, Action::ChooseSearch { card: altanak });
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert_eq!(game.object_view(altanak).zone, Zone::Battlefield);
    assert_eq!(stats(&game, altanak), (9, 9));
    assert_eq!(game.state.players[player.index()].library, before);
    assert_eq!(game.object_view(wrong).zone, Zone::Graveyard);
}

#[test]
fn undead_sprinter_requires_a_non_zombie_death_for_its_countered_cast_route() {
    let player = Player::P0;
    let mut game = scenario(player);
    let sprinter = put(&mut game, player, "Undead Sprinter", Zone::Graveyard);
    let hand = put(&mut game, player, "Undead Sprinter", Zone::Hand);
    let zombie = put(
        &mut game,
        player.other(),
        "Three Tree Mascot",
        Zone::Battlefield,
    );
    let beast = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let announce = Action::AnnounceSelfGraveyardWithCounter {
        object: game.object_ref(sprinter),
        method: CastMethod::Normal,
        flash_surcharge: false,
    };
    reject(&mut game, announce.clone());
    cast_named(&mut game, player, "Murder", Some(Target::Card(zombie)));
    settle(&mut game);
    assert_eq!(game.object_view(zombie).zone, Zone::Graveyard);
    reject(&mut game, announce.clone());
    cast_named(&mut game, player, "Murder", Some(Target::Card(beast)));
    settle(&mut game);
    assert!(game.observe(player).non_zombie_creature_died_this_turn);
    game.state.players[player.index()].mana = [0, 0, 1, 1, 0, 0];
    let mut normal = game.clone();
    cast(&mut normal, hand, None);
    settle(&mut normal);
    assert_eq!(normal.counter_count(hand, CounterKind::PlusOne), 0);

    act(&mut game, announce);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(game.object_view(sprinter).zone, Zone::Battlefield);
    assert_eq!(stats(&game, sprinter), (3, 3));
    assert!(
        game.object_view(sprinter)
            .keywords
            .contains(Keywords::HASTE.union(Keywords::TRAMPLE))
    );
    attack(&mut game, sprinter);
    assert!(game.object_view(sprinter).attacking);
}

#[test]
fn wickerfolk_thresher_has_a_private_land_choice_and_rechecks_delirium() {
    let player = Player::P1;
    let mut game = scenario(player);
    seed_three_types(&mut game, player);
    let fourth = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let ooze = put(
        &mut game,
        player.other(),
        "Scavenging Ooze",
        Zone::Battlefield,
    );
    let thresher = cast_named(&mut game, player, "Wickerfolk Thresher", None);
    settle(&mut game);
    next_main(&mut game, player);
    let land = put(&mut game, player, "Island", Zone::Library);
    attack(&mut game, thresher);
    assert_eq!(game.state.stack.len(), 1);
    let mut no_delirium = game.clone();
    act(&mut no_delirium, Action::Pass);
    no_delirium.state.players[player.other().index()].mana = [0, 0, 0, 0, 1, 0];
    act(&mut no_delirium, activation(ooze));
    act(&mut no_delirium, target(fourth));
    finish_targets_and_pay(&mut no_delirium);
    settle(&mut no_delirium);
    assert_eq!(no_delirium.object_view(fourth).zone, Zone::Exile);
    assert_eq!(no_delirium.object_view(land).zone, Zone::Library);

    progress_until(&mut game, |game| {
        game.observe(player).library_selection.is_some()
    });
    assert!(game.observe(player.other()).library_selection.is_none());
    let mut hand = game.clone();
    act(&mut hand, Action::FinishLibrarySelection);
    assert_eq!(hand.object_view(land).zone, Zone::Hand);
    assert!(
        !hand
            .observe(player.other())
            .known_hand
            .contains(&hand.object_ref(land))
    );
    act(&mut game, Action::SelectLibrary { card: land });
    act(&mut game, Action::FinishLibrarySelection);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert!(!game.object_view(land).tapped);
}

#[test]
fn winter_draws_for_both_players_sets_live_hand_size_and_charges_ward() {
    let player = Player::P0;
    let mut game = scenario(player);
    seed_three_types(&mut game, player);
    let victim = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let winter = cast_named(&mut game, player, "Winter, Misanthropic Guide", None);
    settle(&mut game);
    let maximum = |game: &Game| game.observe(player).hand_size_timeline.unwrap().maximum;
    assert_eq!(maximum(&game), [Some(7), Some(7)]);
    cast_named(&mut game, player, "Murder", Some(Target::Card(victim)));
    settle(&mut game);
    assert_eq!(maximum(&game), [Some(7), Some(3)]);
    put(&mut game, player, "Divination", Zone::Graveyard);
    assert_eq!(maximum(&game), [Some(7), Some(2)]);

    let mut ward = game.clone();
    act(&mut ward, Action::Pass);
    let opponent = player.other();
    ward.state.players[opponent.index()].mana = [0, 0, 2, 0, 0, 3];
    let spell = cast_named(&mut ward, opponent, "Murder", Some(Target::Card(winter)));
    progress_until(&mut ward, |game| {
        game.observe(opponent).resolution_payment.is_some()
    });
    assert_eq!(
        ward.observe(opponent)
            .resolution_payment
            .unwrap()
            .cost
            .generic,
        2
    );
    let mut decline = ward.clone();
    act(&mut decline, Action::DeclineResolutionMana);
    settle(&mut decline);
    assert_eq!(decline.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(decline.object_view(winter).zone, Zone::Battlefield);
    act(
        &mut ward,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 2],
        },
    );
    settle(&mut ward);
    assert_eq!(ward.object_view(winter).zone, Zone::Graveyard);

    let turn = game.state.turn;
    progress_until(&mut game, |game| {
        game.state.turn > turn && game.state.active == player && game.state.step == Step::Upkeep
    });
    let hands = [player, opponent].map(|p| game.hand_count(p));
    settle(&mut game);
    assert_eq!(
        [player, opponent].map(|p| game.hand_count(p)),
        hands.map(|size| size + 2)
    );
}

#[test]
fn victor_shares_one_resolution_count_between_room_unlock_and_enchantment_entry() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let victor = cast_named(&mut game, player, "Victor, Valgavoth's Seneschal", None);
    settle(&mut game);
    assert_eq!(stats(&game, victor), (3, 3));
    let creature = put(&mut game, opponent, "Hill Giant", Zone::Graveyard);
    let own_creature = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let to_discard = put(&mut game, opponent, "Forest", Zone::Hand);
    let kept = put(&mut game, player, "Island", Zone::Library);
    let milled = put(&mut game, player, "Shock", Zone::Library);
    let room = put(&mut game, player, "Smoky Lounge // Misty Salon", Zone::Hand);
    act(
        &mut game,
        Action::AnnounceRoom {
            card: room,
            door: RoomDoor::Left,
            flash_surcharge: false,
        },
    );
    pay(&mut game);
    progress_until(&mut game, |game| {
        game.observe(player).library_choice.is_some()
    });
    assert_eq!(
        game.observe(player).library_choice.unwrap().remaining.len(),
        2
    );
    assert!(game.observe(opponent).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: milled,
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
    settle(&mut game);
    assert_eq!(game.object_view(milled).zone, Zone::Graveyard);

    let object = game.object_ref(room);
    act(
        &mut game,
        Action::BeginRoomUnlock {
            object,
            door: RoomDoor::Right,
            cost: ManaCost {
                colored: [0, 1, 0, 0, 0, 0],
                generic: 3,
            },
        },
    );
    choose(&mut game, |action| {
        matches!(action, Action::PayRoomUnlock { .. })
    });
    let discard = Action::Discard { card: to_discard };
    progress_until(&mut game, |game| game.legal_actions().contains(&discard));
    assert_eq!(game.actor(), Some(opponent));
    act(&mut game, discard);
    settle_ordered(&mut game);
    assert_eq!(game.object_view(to_discard).zone, Zone::Graveyard);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);

    cast_named(&mut game, player, "Impact Tremors", None);
    let return_creature = Action::SelectGraveyardReturn {
        object: game.object_ref(creature),
    };
    progress_until(&mut game, |game| {
        game.legal_actions().contains(&return_creature)
    });
    assert_eq!(game.actor(), Some(player));
    reject(&mut game, Action::FinishGraveyardReturn);
    act(&mut game, return_creature);
    settle_ordered(&mut game);
    let returned = game.object_view(creature);
    assert_eq!(returned.zone, Zone::Battlefield);
    assert_eq!((returned.owner, returned.controller), (opponent, player));
    assert_eq!(game.object_view(own_creature).zone, Zone::Graveyard);

    let untouched = put(&mut game, opponent, "Island", Zone::Hand);
    cast_named(&mut game, player, "Doubling Season", None);
    settle(&mut game);
    assert_eq!(game.object_view(untouched).zone, Zone::Hand);
    assert_eq!(game.object_view(kept).zone, Zone::Library);
    assert_eq!(game.object_view(own_creature).zone, Zone::Graveyard);
}
