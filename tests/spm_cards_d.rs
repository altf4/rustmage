//! Paid outcomes for the final Spider-Man card cohort.
mod support;

use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn activate(game: &mut Game, card: CardId, ability: u16, target: Option<Target>) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if let Some(target) = target {
        act(game, Action::ChooseTarget { target });
    }
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

fn choose_cast_clause(game: &mut Game, wanted: CastCostClause) {
    let clause = game
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| !row.paid && row.clause == wanted)
        .unwrap();
    act(
        game,
        Action::ChooseCastCost {
            clause: clause.try_into().unwrap(),
        },
    );
}

fn web_cast(game: &mut Game, card: CardId, returned: CardId) {
    choose(
        game,
        |action| matches!(action, Action::AnnounceWebSlinging { card: id, .. } if *id == card),
    );
    if game.legal_actions().contains(&Action::BeginOrderedCosts) {
        act(game, Action::BeginOrderedCosts);
    }
    choose_cast_clause(game, CastCostClause::Mana);
    pay(game);
    choose_cast_clause(game, CastCostClause::Additional);
    let object = game.object_ref(returned);
    act(game, Action::PayWebSlingingReturn { object });
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().method,
        CastMethod::WebSlinging
    );
}

fn end_step(game: &mut Game) {
    game.begin_step(Step::End);
    game.prepare_triggers();
}

fn attack_with(game: &mut Game, cards: &[CardId]) {
    game.begin_step(Step::BeginCombat);
    pass_pair(game);
    for &card in cards {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn hit_player(game: &mut Game, source: CardId, player: Player, amount: i128, combat: bool) {
    let event = DamageEvent {
        preventable: true,
        source: game.damage_source(source),
        target: Target::Player(player),
        amount,
        combat,
    };
    game.apply_damage_batch(&[event]);
    game.advance_replacements(false);
    game.prepare_triggers();
}

#[test]
fn spider_suit_equips_for_three_and_moves_its_stats_and_both_subtypes() {
    let player = Player::P0;
    let mut game = scenario(player);
    let equipment = put(&mut game, player, "Spider-Suit", Zone::Hand);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    cast(&mut game, equipment, None);
    settle(&mut game);

    activate(&mut game, equipment, 0, Some(Target::Card(first)));
    settle(&mut game);
    assert_eq!(stats(&game, first), (4, 4));
    for subtype in ["Bear", "Spider", "Hero"] {
        assert!(game.has_subtype(first, subtype));
    }
    activate(&mut game, equipment, 0, Some(Target::Card(second)));
    settle(&mut game);
    assert_eq!(stats(&game, first), (2, 2));
    assert!(!game.has_subtype(first, "Spider"));
    assert!(game.has_subtype(second, "Hero"));
}

#[test]
fn spider_uk_counts_its_own_entry_and_a_prior_creature_that_has_left() {
    for prior in [false, true] {
        let player = Player::P0;
        let mut game = scenario(player);
        let donor = put(
            &mut game,
            player,
            "Grizzly Bears",
            if prior { Zone::Hand } else { Zone::Battlefield },
        );
        if prior {
            cast(&mut game, donor, None);
            settle(&mut game);
        }
        game.state.objects[donor as usize].tapped = true;
        let hero = put(&mut game, player, "Spider-UK", Zone::Hand);
        web_cast(&mut game, hero, donor);
        settle(&mut game);
        assert_eq!(game.object_view(donor).zone, Zone::Hand);
        end_step(&mut game);
        settle(&mut game);
        assert_eq!(
            game.state.players[player.index()].life,
            if prior { 22 } else { 20 }
        );
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            u64::from(prior)
        );
    }
}

#[test]
fn spider_verse_exempts_spider_legends_and_only_copies_one_paid_nonhand_spell() {
    let player = Player::P0;
    let mut game = scenario(player);
    let verse = put(&mut game, player, "Spider-Verse", Zone::Hand);
    cast(&mut game, verse, None);
    settle(&mut game);

    for _ in 0..2 {
        let hero = put(&mut game, player, "Spider-Byte, Web Warden", Zone::Hand);
        cast(&mut game, hero, None);
        settle(&mut game);
        assert_eq!(game.object_view(hero).zone, Zone::Battlefield);
    }
    for first in [true, false] {
        let spell = put(&mut game, player, "Grizzly Bears", Zone::Exile);
        game.grant_exile_play_permission(player, game.object_ref(spell), false);
        act(&mut game, Action::AnnounceExile { card: spell });
        pay(&mut game);
        assert_eq!(game.state.stack.len(), if first { 2 } else { 1 });
        if first {
            pass_pair(&mut game);
            act(&mut game, Action::ChooseOptional { accept: true });
        }
        settle(&mut game);
        assert!(!game.object_view(spell).keywords.contains(Keywords::HASTE));
    }
    let copies: Vec<_> = game
        .observe(player)
        .objects
        .into_iter()
        .filter(|view| view.zone == Zone::Battlefield && view.is_token())
        .collect();
    assert_eq!(copies.len(), 1);
    assert!(copies[0].keywords.contains(Keywords::HASTE));
}

#[test]
fn only_web_cast_spiders_man_makes_reach_tokens_and_gains_life() {
    for web in [false, true] {
        let player = Player::P0;
        let mut game = scenario(player);
        let donor = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        game.state.objects[donor as usize].tapped = true;
        let hero = put(&mut game, player, "Spiders-Man, Heroic Horde", Zone::Hand);
        if web {
            web_cast(&mut game, hero, donor);
        } else {
            cast(&mut game, hero, None);
        }
        settle(&mut game);
        let spiders: Vec<_> = game
            .observe(player)
            .objects
            .into_iter()
            .filter(|row| row.zone == Zone::Battlefield && row.is_token())
            .collect();
        assert_eq!(spiders.len(), if web { 2 } else { 0 });
        for token in spiders {
            assert!(token.has_subtype("Spider"));
            assert!(token.keywords.contains(Keywords::REACH));
            assert_eq!(stats(&game, token.id), (2, 1));
        }
        assert_eq!(
            game.state.players[player.index()].life,
            if web { 23 } else { 20 }
        );
    }
}

#[test]
fn spinneret_requires_two_spider_attackers_or_four_actual_damage_for_its_separate_payoffs() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(
        &mut game,
        player,
        "Spinneret and Spiderling",
        Zone::Battlefield,
    );
    let first = put(&mut game, player, "Treetop Snarespinner", Zone::Battlefield);
    let second = put(&mut game, player, "Treetop Snarespinner", Zone::Battlefield);
    let exiled = put(&mut game, player, "Forest", Zone::Library);
    let mut single = game.clone();
    attack_with(&mut single, &[first]);
    settle(&mut single);
    assert_eq!(single.counter_count(source, CounterKind::PlusOne), 0);

    attack_with(&mut game, &[first, second]);
    settle(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);
    hit_player(&mut game, source, player.other(), 3, false);
    settle(&mut game);
    assert_eq!(game.object_view(exiled).zone, Zone::Library);
    hit_player(&mut game, source, player.other(), 4, false);
    settle(&mut game);
    assert_eq!(game.object_view(exiled).zone, Zone::Exile);
    assert_eq!(
        game.observe(player).exile_permissions[0].object,
        game.object_ref(exiled)
    );
}

#[test]
fn stegron_discards_from_hand_and_keeps_the_targets_native_subtypes_until_cleanup() {
    let player = Player::P0;
    let mut game = scenario(player);
    let stegron = put(&mut game, player, "Stegron the Dinosaur Man", Zone::Hand);
    let target = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    assert!(
        game.object_view(stegron)
            .keywords
            .contains(Keywords::MENACE)
    );
    activate(&mut game, stegron, 0, Some(Target::Card(target)));
    settle(&mut game);
    assert_eq!(game.object_view(stegron).zone, Zone::Graveyard);
    assert_eq!(stats(&game, target), (5, 3));
    assert!(game.has_subtype(target, "Bear") && game.has_subtype(target, "Dinosaur"));
    end_step(&mut game);
    game.give_priority();
    pass_pair(&mut game);
    assert_eq!(stats(&game, target), (2, 2));
    assert!(!game.has_subtype(target, "Dinosaur"));
}

#[test]
fn strength_of_will_preserves_a_lethally_damaged_creature_and_counts_actual_damage() {
    let player = Player::P0;
    let mut game = scenario(player);
    let target = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let strength = put(&mut game, player, "Strength of Will", Zone::Hand);
    cast(&mut game, strength, Some(Target::Card(target)));
    settle(&mut game);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(target)));
    settle(&mut game);
    assert_eq!(game.object_view(target).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(target, CounterKind::PlusOne), 2);
    assert!(game.keywords(target).contains(Keywords::INDESTRUCTIBLE));
    end_step(&mut game);
    game.give_priority();
    pass_pair(&mut game);
    assert!(!game.keywords(target).contains(Keywords::INDESTRUCTIBLE));
}

#[test]
fn subway_train_accepts_green_to_find_a_basic_and_crews_with_two_power() {
    let player = Player::P0;
    let mut game = scenario(player);
    let train = put(&mut game, player, "Subway Train", Zone::Hand);
    let forest = put(&mut game, player, "Forest", Zone::Library);
    let crew = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    cast(&mut game, train, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    assert_eq!(declined.object_view(forest).zone, Zone::Library);
    act(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 1, 0],
        },
    );
    act(&mut game, Action::ChooseSearch { card: forest });
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    settle(&mut game);
    assert_eq!(game.object_view(forest).zone, Zone::Hand);
    assert!(!game.is_creature(train));
    activate(&mut game, train, 0, None);
    let object = game.object_ref(crew);
    act(&mut game, Action::ChooseCrewer { object });
    act(&mut game, Action::FinishCrew);
    settle(&mut game);
    assert_eq!(stats(&game, train), (3, 1));
    assert!(game.object_view(crew).tapped);
}

#[test]
fn sun_spider_finds_an_aura_or_equipment_and_flies_only_for_the_live_controllers_turn() {
    let player = Player::P0;
    let mut game = scenario(player);
    let hero = put(&mut game, player, "Sun-Spider, Nimble Webber", Zone::Hand);
    let equipment = put(&mut game, player, "Web-Shooters", Zone::Library);
    let aura = put(&mut game, player, "Robotics Mastery", Zone::Library);
    let artifact = put(&mut game, player, "Hot Dog Cart", Zone::Library);
    cast(&mut game, hero, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert!(
        game.legal_actions()
            .contains(&Action::ChooseSearch { card: aura })
    );
    reject(&mut game, Action::ChooseSearch { card: artifact });
    act(&mut game, Action::ChooseSearch { card: equipment });
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    settle(&mut game);
    assert_eq!(game.object_view(equipment).zone, Zone::Hand);
    assert!(game.keywords(hero).contains(Keywords::FLYING));
    game.record_control(game.object_ref(hero), player.other(), None);
    assert!(!game.keywords(hero).contains(Keywords::FLYING));
}

#[test]
fn superior_foes_requires_four_mana_value_and_a_new_exile_supersedes_the_old_grant() {
    let player = Player::P0;
    let mut game = scenario(player);
    let foe = put(
        &mut game,
        player,
        "Superior Foes of Spider-Man",
        Zone::Battlefield,
    );
    assert!(game.keywords(foe).contains(Keywords::TRAMPLE));
    let small = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, small, None);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    let mut exiled = Vec::new();
    for name in ["Forest", "Island"] {
        let top = put(&mut game, player, name, Zone::Library);
        let spell = put(&mut game, player, "Hill Giant", Zone::Hand);
        cast(&mut game, spell, None);
        pass_pair(&mut game);
        act(&mut game, Action::ChooseOptional { accept: true });
        settle(&mut game);
        exiled.push(top);
    }
    for top in &exiled {
        assert_eq!(game.object_view(*top).zone, Zone::Exile);
    }
    let permissions = game.observe(player).exile_permissions;
    assert_eq!(permissions.len(), 1);
    assert_eq!(permissions[0].object, game.object_ref(exiled[1]));
}

fn pick_library(game: &mut Game, card: CardId) {
    act(game, Action::SelectLibrary { card });
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(game, Action::FinishLibrarySelection);
    }
    settle(game);
}

#[test]
fn superior_spider_man_copies_an_opposing_graveyard_creature_with_exact_exceptions() {
    let player = Player::P0;
    let mut game = scenario(player);
    let original = put(&mut game, player.other(), "Llanowar Elves", Zone::Graveyard);
    let card = put(&mut game, player, "Superior Spider-Man", Zone::Hand);
    cast(&mut game, card, None);
    pass_pair(&mut game);
    let object = Some(game.object_ref(original));
    act(&mut game, Action::ChooseEntryCopy { object });
    settle(&mut game);
    let view = game.object_view(card);
    assert_eq!(view.name, "Superior Spider-Man");
    assert_eq!(stats(&game, card), (4, 4));
    assert!(!view.is_legendary());
    for subtype in ["Elf", "Spider", "Human", "Hero"] {
        assert!(view.has_subtype(subtype), "{subtype}");
    }
    assert_eq!(game.object_view(original).zone, Zone::Exile);
}

#[test]
fn symbiote_spider_mans_paid_graveyard_ability_grants_its_private_combat_payoff() {
    let player = Player::P0;
    let mut game = scenario(player);
    let grave = put(&mut game, player, "Symbiote Spider-Man", Zone::Graveyard);
    let host = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let discarded = put(&mut game, player, "Mountain", Zone::Library);
    let selected = put(&mut game, player, "Island", Zone::Library);
    activate(&mut game, grave, 0, Some(Target::Card(host)));
    settle(&mut game);
    assert_eq!(game.object_view(grave).zone, Zone::Exile);
    assert_eq!(game.counter_count(host, CounterKind::PlusOne), 1);
    hit_player(&mut game, host, player.other(), 2, true);
    pass_pair(&mut game);
    assert!(game.observe(player.other()).library_selection.is_none());
    pick_library(&mut game, selected);
    assert_eq!(game.object_view(selected).zone, Zone::Hand);
    assert_eq!(game.object_view(discarded).zone, Zone::Graveyard);
}

#[test]
fn terrific_team_up_costs_two_less_with_a_four_mana_permanent_and_grows_both_sources() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Terrific Team-Up", Zone::Hand);
    let allies = [
        put(&mut game, player, "Grizzly Bears", Zone::Battlefield),
        put(&mut game, player, "Bear Cub", Zone::Battlefield),
    ];
    let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 1];
    reject(&mut game, Action::AnnounceCast { card: spell });
    put(&mut game, player, "Hedron Archive", Zone::Battlefield);
    announce(
        &mut game,
        spell,
        &[allies[0], allies[1], enemy].map(Target::Card),
    );
    assert_eq!(
        game.state
            .stack
            .last()
            .unwrap()
            .cast
            .unwrap()
            .payment
            .iter()
            .sum::<u32>(),
        2
    );
    settle(&mut game);
    for ally in allies {
        assert_eq!(stats(&game, ally), (3, 2));
    }
    assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
}

fn next_chapter(game: &mut Game) {
    game.state.turn += 1;
    game.begin_step(Step::Main1);
    game.prepare_triggers();
}

#[test]
fn the_clone_saga_surveils_copies_the_next_creature_and_remembers_a_name_after_sacrifice() {
    let player = Player::P0;
    let mut game = scenario(player);
    let saga = put(&mut game, player, "The Clone Saga", Zone::Hand);
    cast(&mut game, saga, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    for _ in 0..3 {
        choose(&mut game, |action| {
            matches!(action, Action::ArrangeLibrary { .. })
        });
    }
    settle(&mut game);
    assert_eq!(game.counter_count(saga, CounterKind::Lore), 1);

    next_chapter(&mut game);
    settle(&mut game);
    let legend = put(&mut game, player, "Spider-Byte, Web Warden", Zone::Hand);
    cast(&mut game, legend, None);
    settle(&mut game);
    let copy = game
        .observe(player)
        .objects
        .into_iter()
        .find(|row| {
            row.is_token() && row.zone == Zone::Battlefield && row.name == "Spider-Byte, Web Warden"
        })
        .unwrap();
    assert!(!copy.is_legendary());
    assert!(game.object_view(legend).is_legendary());

    next_chapter(&mut game);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::CardName(
                rustmage_core::CardName::from_name("Grizzly Bears").unwrap(),
            ),
        },
    );
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);
    let attacker = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let before = game.hand_count(player);
    hit_player(&mut game, attacker, player, 2, true);
    settle(&mut game);
    assert_eq!(game.hand_count(player), before + 1);
}

#[test]
fn the_death_of_gwen_stacy_destroys_then_resolves_each_players_discard_and_exiles_both_graveyards()
{
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let target = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let card = put(&mut game, player, "Mountain", Zone::Hand);
    let held = put(&mut game, opponent, "Island", Zone::Hand);
    let saga = put(&mut game, player, "The Death of Gwen Stacy", Zone::Hand);
    cast(&mut game, saga, None);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(target).zone, Zone::Graveyard);

    next_chapter(&mut game);
    pass_pair(&mut game);
    act(&mut game, Action::Discard { card });
    assert_eq!(game.object_view(card).zone, Zone::Hand);
    act(&mut game, Action::ChooseOptional { accept: false });
    assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    assert_eq!(game.object_view(held).zone, Zone::Hand);
    assert_eq!(game.state.players[opponent.index()].life, 17);

    next_chapter(&mut game);
    for target in [player, opponent] {
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(target),
            },
        );
    }
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    settle(&mut game);
    assert_eq!(game.object_view(target).zone, Zone::Exile);
    assert_eq!(game.object_view(card).zone, Zone::Exile);
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);
}

#[test]
fn the_soul_stone_exiles_a_creature_to_harness_and_reanimates_at_its_controllers_upkeep() {
    let player = Player::P0;
    let mut game = scenario(player);
    let stone = put(&mut game, player, "The Soul Stone", Zone::Battlefield);
    let grave = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let offering = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    assert!(game.keywords(stone).contains(Keywords::INDESTRUCTIBLE));
    game.begin_step(Step::Upkeep);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());
    game.state.step = Step::Main1;
    let object = game.object_ref(offering);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: stone,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    act(&mut game, Action::ChooseAdditionalCost { object });
    pay(&mut game);
    assert_eq!(game.object_view(offering).zone, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.object_view(stone).harnessed, Some(true));

    game.begin_step(Step::Upkeep);
    game.prepare_triggers();
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(grave),
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(grave).zone, Zone::Battlefield);
}

#[test]
fn the_spot_exiles_an_artifact_and_a_graveyard_creature_then_returns_both_on_death() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spot = put(&mut game, player, "The Spot, Living Portal", Zone::Hand);
    let artifact = put(
        &mut game,
        player.other(),
        "Swiftfoot Boots",
        Zone::Battlefield,
    );
    let grave = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    cast(&mut game, spot, None);
    pass_pair(&mut game);
    for card in [artifact, grave] {
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
    settle(&mut game);
    for card in [artifact, grave] {
        assert_eq!(game.object_view(card).zone, Zone::Exile);
    }
    let wrath = put(&mut game, player, "Day of Judgment", Zone::Hand);
    cast(&mut game, wrath, None);
    settle(&mut game);
    assert_eq!(game.object_view(spot).zone, Zone::Library);
    assert_eq!(game.state.players[player.index()].library[0], spot);
    for card in [artifact, grave] {
        assert_eq!(game.object_view(card).zone, Zone::Hand);
    }
}

#[test]
fn urban_retreat_enters_tapped_through_its_hand_activation_without_using_a_land_play() {
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Urban Retreat", Zone::Hand);
    let donor = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[donor as usize].tapped = true;
    act(
        &mut game,
        Action::AnnounceActivate {
            card: land,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    if game.legal_actions().contains(&Action::BeginOrderedCosts) {
        act(&mut game, Action::BeginOrderedCosts);
        choose_cast_clause(&mut game, CastCostClause::Mana);
        pay(&mut game);
        choose_cast_clause(&mut game, CastCostClause::Additional);
    }
    let object = game.object_ref(donor);
    act(&mut game, Action::ChooseAdditionalCost { object });
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    assert_eq!(game.object_view(donor).zone, Zone::Hand);
    settle(&mut game);
    assert!(game.object_view(land).tapped);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert!(!game.state.land_played);
    game.state.objects[land as usize].tapped = false;
    let colors: std::collections::BTreeSet<_> = game
        .legal_actions()
        .into_iter()
        .filter_map(|a| match a {
            Action::ActivateMana {
                card, production, ..
            } if card == land => Some(production),
            _ => None,
        })
        .collect();
    assert_eq!(
        colors,
        [[1, 0, 0, 0, 0, 0], [0, 1, 0, 0, 0, 0], [0, 0, 0, 0, 1, 0]]
            .into_iter()
            .collect()
    );
}

#[test]
fn villainous_wrath_counts_the_opponents_creatures_before_destroying_both_sides() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let targets = [
        put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield),
        put(&mut game, opponent, "Bear Cub", Zone::Battlefield),
        put(&mut game, player, "Hill Giant", Zone::Battlefield),
    ];
    let wrath = put(&mut game, player, "Villainous Wrath", Zone::Hand);
    cast(&mut game, wrath, Some(Target::Player(opponent)));
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 18);
    for id in targets {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
}

#[test]
fn wall_crawl_counts_its_new_spider_and_only_defenders_are_barred_from_blocking_spiders() {
    let player = Player::P0;
    let mut game = scenario(player);
    let wall = put(&mut game, player, "Wall Crawl", Zone::Hand);
    let spider = put(&mut game, player, "Treetop Snarespinner", Zone::Battlefield);
    let defender = put(
        &mut game,
        player.other(),
        "Sanctuary Wall",
        Zone::Battlefield,
    );
    let ordinary = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let before = stats(&game, spider);
    cast(&mut game, wall, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 22);
    assert_eq!(stats(&game, spider), (before.0 + 1, before.1 + 1));
    attack_with(&mut game, &[spider]);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::Block {
            blocker: defender,
            attacker: spider,
        },
    );
    act(
        &mut game,
        Action::Block {
            blocker: ordinary,
            attacker: spider,
        },
    );
}

#[test]
fn web_of_life_and_destiny_accepts_colored_convoke_and_privately_deploys_a_top_five_creature() {
    let player = Player::P0;
    let mut game = scenario(player);
    let creatures = [
        put(&mut game, player, "Bear Cub", Zone::Battlefield),
        put(&mut game, player, "Grizzly Bears", Zone::Battlefield),
    ];
    let enchantment = put(&mut game, player, "Web of Life and Destiny", Zone::Hand);
    let candidate = put(&mut game, player, "Hill Giant", Zone::Library);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 6];
    act(&mut game, Action::AnnounceCast { card: enchantment });
    act(&mut game, Action::BeginConvoke);
    for card in creatures {
        let object = game.object_ref(card);
        act(
            &mut game,
            Action::ChooseConvoker {
                object,
                contribution: ConvokeContribution::Green,
            },
        );
    }
    pay(&mut game);
    settle(&mut game);
    for card in creatures {
        assert!(game.object_view(card).tapped);
    }
    game.begin_step(Step::BeginCombat);
    game.prepare_triggers();
    pass_pair(&mut game);
    assert!(game.observe(player.other()).library_selection.is_none());
    pick_library(&mut game, candidate);
    assert_eq!(game.object_view(candidate).zone, Zone::Battlefield);
}

#[test]
fn wild_pack_squad_targets_either_seats_creature_and_both_grants_expire() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Wild Pack Squad", Zone::Battlefield);
    let chosen = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    game.begin_step(Step::BeginCombat);
    game.prepare_triggers();
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(chosen),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    settle(&mut game);
    let keywords = Keywords::FIRST_STRIKE.union(Keywords::VIGILANCE);
    assert!(game.keywords(chosen).contains(keywords));
    end_step(&mut game);
    game.give_priority();
    pass_pair(&mut game);
    assert!(!game.keywords(chosen).intersects(keywords));
}

#[test]
fn wisecrack_adds_two_player_damage_only_for_an_attacking_creature() {
    for attacking in [false, true] {
        let player = Player::P0;
        let mut game = scenario(player);
        let creature = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let spell = put(&mut game, player, "Wisecrack", Zone::Hand);
        if attacking {
            attack_with(&mut game, &[creature]);
            game.state.players[player.index()].mana = [0, 0, 0, 3, 0, 0];
        }
        cast(&mut game, spell, Some(Target::Card(creature)));
        settle(&mut game);
        assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
        assert_eq!(
            game.state.players[player.index()].life,
            if attacking { 18 } else { 20 }
        );
    }
}

#[test]
fn with_great_power_counts_its_own_aura_and_equipment_and_redirects_player_damage() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bearer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let aura = put(&mut game, player, "With Great Power . . .", Zone::Hand);
    let equipment = put(&mut game, player, "Web-Shooters", Zone::Battlefield);
    let foe = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    cast(&mut game, aura, Some(Target::Card(bearer)));
    settle(&mut game);
    assert_eq!(stats(&game, bearer), (4, 4));
    activate(&mut game, equipment, 0, Some(Target::Card(bearer)));
    settle(&mut game);
    assert_eq!(stats(&game, bearer), (7, 7));
    hit_player(&mut game, foe, player, 3, false);
    assert_eq!(game.state.players[player.index()].life, 20);
    assert_eq!(game.object_view(bearer).damage, 3);
}

#[test]
fn wraith_cannot_be_blocked_and_deals_damage_in_both_steps() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let wraith = put(
        &mut game,
        player,
        "Wraith, Vicious Vigilante",
        Zone::Battlefield,
    );
    let blocker = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    attack_with(&mut game, &[wraith]);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::Block {
            blocker,
            attacker: wraith,
        },
    );
    act(&mut game, Action::FinishBlockers);
    while game.state.step != Step::EndCombat {
        pass_pair(&mut game);
    }
    assert_eq!(game.state.players[opponent.index()].life, 18);
}
