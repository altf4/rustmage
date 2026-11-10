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
            u32::from(prior).into()
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
