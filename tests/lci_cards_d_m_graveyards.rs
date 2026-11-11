//! Card wiring for linked exile, exact entry and restricted mana.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const WHITE: Mana = [1, 0, 0, 0, 0, 0];
const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const RED: Mana = [0, 0, 0, 1, 0, 0];
const COLORLESS: Mana = [0, 0, 0, 0, 0, 1];

fn named(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}

fn tap(card: CardId, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
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

#[test]
fn echoing_deeps_can_remain_itself_or_copy_either_graveyard_land_with_cave() {
    let p = Player::P1;
    let mut game = scenario(p);
    let card = named(&mut game, p, "Echoing Deeps", Zone::Hand);
    let island = put(&mut game, p, "Island", Zone::Graveyard);
    let pylon = put(&mut game, p.other(), "Conduit Pylons", Zone::Graveyard);
    let creature = put(&mut game, p.other(), "Grizzly Bears", Zone::Graveyard);
    let play = Action::PlayLand { card };
    let copy = |game: &Game, id| Action::ChooseEntryCopy {
        object: Some(game.object_ref(id)),
    };
    act(&mut game, play);
    let invalid = copy(&game, creature);
    reject(&mut game, invalid);
    for viewer in [p, p.other()] {
        assert_eq!(
            game.observe(viewer).entry_copy.unwrap().candidates,
            [island, pylon].map(|id| game.object_ref(id))
        );
    }

    let mut declined = game.clone();
    act(&mut declined, Action::ChooseEntryCopy { object: None });
    assert_eq!(declined.object_view(card).name, "Echoing Deeps");
    assert!(!declined.object_view(card).tapped);
    assert!(declined.has_subtype(card, "Cave"));
    declined.state.players[p.index()].mana = [0; 6];
    act(&mut declined, tap(card, COLORLESS));
    assert_eq!(declined.state.players[p.index()].mana, COLORLESS);

    let mut blue = game.clone();
    act(&mut blue, copy(&game, island));
    assert_eq!(blue.object_view(card).name, "Island");
    assert!(blue.object_view(card).tapped);
    assert!(blue.has_subtype(card, "Cave"));
    blue.state.players[p.index()].mana = [0; 6];
    blue.untap_objects(&[card], untap_rules::UntapCause::Effect);
    act(&mut blue, Action::TapMana { card });
    assert_eq!(blue.state.players[p.index()].mana, BLUE);

    let choose_pylon = copy(&game, pylon);
    act(&mut game, choose_pylon);
    assert_eq!(game.object_view(card).name, "Conduit Pylons");
    assert!(game.object_view(card).tapped);
    assert!(game.has_subtype(card, "Cave"));
    pass_pair(&mut game);
    assert!(game.observe(p).library_choice.unwrap().surveil);
    assert!(game.observe(p.other()).library_choice.is_none());
}

#[test]
fn fabrication_foundry_mana_funds_artifacts_and_exact_materials_pay_its_return() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = named(&mut game, p, "Fabrication Foundry", Zone::Battlefield);
    let funder = named(&mut game, p, "Fabrication Foundry", Zone::Battlefield);
    let material = put(&mut game, p, "Candy Trail", Zone::Battlefield);
    let creature = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let own = put(&mut game, p, "Basilisk Collar", Zone::Graveyard);
    let foreign = put(&mut game, p.other(), "Basilisk Collar", Zone::Graveyard);
    let spell = put(&mut game, p, "Basilisk Collar", Zone::Hand);
    let nonartifact = named(&mut game, p, "Helping Hand", Zone::Hand);
    put(&mut game, p, "Bear Cub", Zone::Graveyard);
    game.state.players[p.index()].mana = [0; 6];

    let mut cast_branch = game.clone();
    act(&mut cast_branch, tap(funder, WHITE));
    assert_eq!(cast_branch.state.players[p.index()].mana, WHITE);
    reject(&mut cast_branch, Action::AnnounceCast { card: nonartifact });
    cast(&mut cast_branch, spell, None);
    settle(&mut cast_branch);
    assert_eq!(cast_branch.object_view(spell).zone, Zone::Battlefield);
    assert_eq!(cast_branch.state.players[p.index()].mana, [0; 6]);

    game.state.players[p.index()].mana[5] = 2;
    act(&mut game, activate(source, 1));
    reject(&mut game, target(foreign));
    act(&mut game, target(own));
    reject(&mut game, tap(source, WHITE));
    act(&mut game, tap(funder, WHITE));
    pay(&mut game);
    choose(&mut game, |action| {
        matches!(action, Action::ChooseCastCost { .. })
    });
    let selection = |game: &Game, id| Action::ChooseAdditionalCost {
        object: game.object_ref(id),
    };
    let source_cost = selection(&game, source);
    let creature_cost = selection(&game, creature);
    reject(&mut game, source_cost);
    reject(&mut game, creature_cost);
    reject(&mut game, Action::FinishAdditionalCostGroup);
    let artifact_cost = selection(&game, material);
    act(&mut game, artifact_cost);
    act(&mut game, Action::FinishAdditionalCostGroup);
    assert_eq!(game.object_view(material).zone, Zone::Exile);
    assert!(game.object_view(source).tapped);
    assert_eq!(game.state.stack.last().unwrap().x, Some(1));
    settle(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Battlefield);
    assert_eq!(game.object_view(foreign).zone, Zone::Graveyard);
}

#[test]
fn forgotten_monument_grants_the_life_paid_mana_ability_to_other_owned_caves() {
    let p = Player::P1;
    let mut game = scenario(p);
    let monument = named(&mut game, p, "Forgotten Monument", Zone::Hand);
    let cave = put(&mut game, p, "Captivating Cave", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Captivating Cave", Zone::Battlefield);
    let spell = put(&mut game, p, "Spyglass Siren", Zone::Hand);
    game.state.players[p.index()].mana = [0; 6];
    act(&mut game, Action::PlayLand { card: monument });
    assert!(!game.object_view(monument).tapped);
    let source = game.object_ref(monument);
    let grant = |game: &Game, id| Action::AnnounceGrantedManaTemplate {
        source: game.object_ref(id),
        grantor: source,
        ability: 1,
        production: BLUE,
    };
    let self_grant = grant(&game, monument);
    let enemy_grant = grant(&game, enemy);
    reject(&mut game, self_grant);
    reject(&mut game, enemy_grant);
    let mut base = game.clone();
    act(&mut base, tap(monument, COLORLESS));
    assert_eq!(base.state.players[p.index()].mana, COLORLESS);

    let mut removed = game.clone();
    removed.move_object(monument, Zone::Graveyard);
    reject(&mut removed, Action::AnnounceCast { card: spell });
    act(&mut game, Action::AnnounceCast { card: spell });
    let blue = grant(&game, cave);
    act(&mut game, blue);
    if game.state.players[p.index()].mana[1] == 0 {
        pay(&mut game);
    }
    assert_eq!(game.state.players[p.index()].life, STARTING_LIFE - 1);
    assert!(game.object_view(cave).tapped);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
}

#[test]
fn fungal_fortitude_flashes_in_and_returns_a_stolen_creature_tapped_to_its_owner() {
    let p = Player::P0;
    let opponent = p.other();
    let mut game = scenario(p);
    let creature = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    game.record_control(game.object_ref(creature), p, None);
    let aura = named(&mut game, p, "Fungal Fortitude", Zone::Hand);
    let shock = put(&mut game, p, "Shock", Zone::Hand);
    game.state.active = opponent;
    game.state.step = Step::Upkeep;
    cast(&mut game, aura, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(
        game.object_view(aura).attached_to,
        Some(game.object_ref(creature))
    );
    assert_eq!(stats(&game, creature), (4, 2));
    let original = game.object_ref(creature);
    game.state.priority = p;
    cast(&mut game, shock, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(game.object_view(aura).zone, Zone::Graveyard);
    assert_ne!(game.object_ref(creature), original);
    assert_eq!(game.object_view(creature).controller, opponent);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    assert!(game.object_view(creature).tapped);
    assert_eq!(stats(&game, creature), (2, 2));
}

#[test]
fn helping_hand_returns_only_the_exact_owned_creature_up_to_mana_value_three() {
    let p = Player::P1;
    let mut game = scenario(p);
    let own = put(&mut game, p, "Kinjalli's Dawnrunner", Zone::Graveyard);
    let foreign = put(
        &mut game,
        p.other(),
        "Kinjalli's Dawnrunner",
        Zone::Graveyard,
    );
    let expensive = put(&mut game, p, "Hill Giant", Zone::Graveyard);
    let spell = named(&mut game, p, "Helping Hand", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, target(foreign));
    reject(&mut game, target(expensive));
    act(&mut game, target(own));
    pay(&mut game);
    let mut stale = game.clone();
    stale.move_object(own, Zone::Hand);
    stale.move_object(own, Zone::Graveyard);
    settle(&mut stale);
    assert_eq!(stale.object_view(own).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Battlefield);
    assert!(game.object_view(own).tapped);
}

#[test]
fn intrepid_paleontologist_exiles_any_card_and_casts_only_linked_owned_dinosaurs() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = named(&mut game, p, "Intrepid Paleontologist", Zone::Battlefield);
    let own = put(&mut game, p, "Belligerent Yearling", Zone::Graveyard);
    let foreign = put(
        &mut game,
        p.other(),
        "Belligerent Yearling",
        Zone::Graveyard,
    );
    let other_type = put(&mut game, p, "Shock", Zone::Graveyard);
    game.state.players[p.index()].mana = [0; 6];
    act(&mut game, tap(source, BLUE));
    assert_eq!(game.state.players[p.index()].mana, BLUE);
    game.clear_step_mana(false, false);
    for card in [own, foreign, other_type] {
        game.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        act(&mut game, activate(source, 1));
        act(&mut game, target(card));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(card).zone, Zone::Exile);
        assert_eq!(game.state.players[p.index()].mana, [0; 6]);
    }

    let permission = |game: &Game, card| Action::AnnounceLinkedPaid {
        object: game.object_ref(card),
        source: game.object_ref(source),
        face: CastFace::Normal,
        flash_surcharge: false,
    };
    game.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
    let foreign_cast = permission(&game, foreign);
    let non_dinosaur_cast = permission(&game, other_type);
    reject(&mut game, foreign_cast);
    reject(&mut game, non_dinosaur_cast);
    let cast_owned = permission(&game, own);
    game.state.step = Step::Upkeep;
    reject(&mut game, cast_owned.clone());
    game.state.step = Step::Main1;
    act(&mut game, cast_owned);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(own, CounterKind::Finality), 1);
    let shock = put(&mut game, p, "Shock", Zone::Hand);
    game.state.players[p.index()].mana = RED;
    cast(&mut game, shock, Some(Target::Card(own)));
    settle(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Exile);
}

#[test]
fn ixallis_lorekeeper_mana_pays_for_a_dinosaur_spell_and_rejects_other_spells() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = named(&mut game, p, "Ixalli's Lorekeeper", Zone::Battlefield);
    let dinosaur = put(&mut game, p, "Belligerent Yearling", Zone::Hand);
    let instant = put(&mut game, p, "Shock", Zone::Hand);
    put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    game.state.players[p.index()].mana = COLORLESS;
    act(&mut game, tap(source, RED));
    assert_eq!(game.state.players[p.index()].mana, [0, 0, 0, 1, 0, 1]);
    reject(&mut game, Action::AnnounceCast { card: instant });
    let mut cycling = game.clone();
    let brinefang = put(&mut cycling, p, "Marauding Brinefang", Zone::Hand);
    let island = put(&mut cycling, p, "Island", Zone::Library);
    act(&mut cycling, activate(brinefang, 0));
    pay(&mut cycling);
    assert_eq!(cycling.object_view(brinefang).zone, Zone::Graveyard);
    assert_eq!(cycling.state.players[p.index()].mana, [0; 6]);
    pass_pair(&mut cycling);
    act(&mut cycling, Action::ChooseSearch { card: island });
    if cycling.legal_actions().contains(&Action::FinishSearch) {
        act(&mut cycling, Action::FinishSearch);
    }
    assert_eq!(cycling.object_view(island).zone, Zone::Hand);
    cast(&mut game, dinosaur, None);
    settle(&mut game);
    assert_eq!(game.object_view(dinosaur).zone, Zone::Battlefield);
    assert_eq!(game.state.players[p.index()].mana, [0; 6]);
}

#[test]
fn jadelight_spelunker_resolves_each_paid_explore_before_priority_returns() {
    let p = Player::P0;
    let mut game = scenario(p);
    let top = put(&mut game, p, "Shock", Zone::Library);
    let spelunker = named(&mut game, p, "Jadelight Spelunker", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spelunker });
    let mut zero = game.clone();
    act(&mut zero, Action::ChooseX { value: 0 });
    pay(&mut zero);
    settle(&mut zero);
    assert_eq!(zero.counter_count(spelunker, CounterKind::PlusOne), 0);
    assert_eq!(zero.top_library_card(p), Some(top));

    act(&mut game, Action::ChooseX { value: 2 });
    pay(&mut game);
    pass_pair(&mut game);
    pass_pair(&mut game);
    for (graveyard, counters) in [(false, 1), (true, 2)] {
        let choice = game.observe(p.other()).explore_choice.unwrap();
        assert_eq!(choice.revealed.object, game.object_ref(top));
        assert_eq!(
            game.counter_count(spelunker, CounterKind::PlusOne),
            counters
        );
        act(
            &mut game,
            Action::ChooseExplore {
                object: choice.revealed.object,
                graveyard,
            },
        );
    }
    assert_eq!(game.state.pending, Pending::Priority);
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
    assert_eq!(stats(&game, spelunker), (3, 3));
}
