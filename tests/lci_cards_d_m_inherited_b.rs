//! Printed obligations of previously admitted G–M LCI cards.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn attack(game: &mut Game, attackers: &[CardId]) {
    game.begin_step(Step::BeginCombat);
    pass_pair(game);
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn combat_to_main_two(game: &mut Game) {
    for _ in 0..24 {
        if game.state.step == Step::Main2 && game.state.pending == Pending::Priority {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishBlockers
            )
        });
    }
    panic!("combat did not finish");
}

#[test]
fn glimpse_the_core_searches_only_basic_forest_and_returns_only_an_owned_cave() {
    let p = Player::P1;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Glimpse the Core", Zone::Hand);
    let forest = put(&mut game, p, "Forest", Zone::Library);
    let nonbasic = spawn(
        &mut game,
        p,
        Card::from_name("Lush Portico").unwrap(),
        Zone::Library,
    );
    let cave = put(&mut game, p, "Captivating Cave", Zone::Graveyard);
    let foreign = put(&mut game, p.other(), "Captivating Cave", Zone::Graveyard);
    let not_cave = put(&mut game, p, "Forest", Zone::Graveyard);
    act(&mut game, Action::AnnounceCast { card: spell });
    let mut search = game.clone();
    act(&mut search, Action::ChooseMode { mode: 0 });
    pay(&mut search);
    pass_pair(&mut search);
    reject(&mut search, Action::ChooseSearch { card: nonbasic });
    act(&mut search, Action::ChooseSearch { card: forest });
    if search.legal_actions().contains(&Action::FinishSearch) {
        act(&mut search, Action::FinishSearch);
    }
    assert_eq!(search.object_view(forest).zone, Zone::Battlefield);
    assert!(search.object_view(forest).tapped);

    act(&mut game, Action::ChooseMode { mode: 1 });
    reject(&mut game, target(foreign));
    reject(&mut game, target(not_cave));
    act(&mut game, target(cave));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(cave).zone, Zone::Battlefield);
    assert!(game.object_view(cave).tapped);
}

#[test]
fn goblin_tomb_raider_gains_and_loses_its_artifact_bonus_during_combat() {
    let p = Player::P0;
    let mut game = scenario(p);
    let raider = put(&mut game, p, "Goblin Tomb Raider", Zone::Hand);
    cast(&mut game, raider, None);
    settle(&mut game);
    assert_eq!(stats(&game, raider), (1, 2));
    assert!(!game.keywords(raider).contains(Keywords::HASTE));
    put(&mut game, p.other(), "Short Bow", Zone::Battlefield);
    assert_eq!(stats(&game, raider), (1, 2));
    let artifact = put(&mut game, p, "Short Bow", Zone::Battlefield);
    assert_eq!(stats(&game, raider), (2, 2));
    assert!(game.keywords(raider).contains(Keywords::HASTE));
    attack(&mut game, &[raider]);
    game.move_object(artifact, Zone::Hand);
    assert!(game.object_view(raider).attacking);
    assert!(!game.keywords(raider).contains(Keywords::HASTE));
    assert_eq!(stats(&game, raider), (1, 2));
    combat_to_main_two(&mut game);
    assert_eq!(
        game.state.players[p.other().index()].life,
        STARTING_LIFE - 1
    );
}

#[test]
fn greedy_freebooter_scries_privately_before_creating_treasure() {
    let p = Player::P1;
    let mut game = scenario(p);
    let creature = put(&mut game, p, "Greedy Freebooter", Zone::Battlefield);
    let top = put(&mut game, p, "Opt", Zone::Library);
    let shock = put(&mut game, p, "Shock", Zone::Hand);
    let first_possible_token = game.state.objects.len() as CardId;
    cast(&mut game, shock, Some(Target::Card(creature)));
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
    assert_eq!(game.state.objects.len(), first_possible_token as usize);
    assert_eq!(
        game.observe(p).library_choice.unwrap().remaining[0].reference,
        game.object_ref(top)
    );
    assert!(game.observe(p.other()).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Bottom,
        },
    );
    let token = game.object_view(first_possible_token);
    assert!(token.has_subtype("Treasure") && token.is_token());
    assert!(!token.tapped);
    assert_eq!(game.state.players[p.index()].library[0], top);
}

#[test]
fn hermitic_nautilus_attacks_without_tapping_and_repeated_activation_is_lethal() {
    let p = Player::P0;
    let mut game = scenario(p);
    let nautilus = put(&mut game, p, "Hermitic Nautilus", Zone::Battlefield);
    attack(&mut game, &[nautilus]);
    assert!(!game.object_view(nautilus).tapped);
    game.state.players[p.index()].mana = [0, 2, 0, 0, 0, 2];
    for activation in 1..=2 {
        act(&mut game, activate(nautilus));
        pay(&mut game);
        settle(&mut game);
        if activation == 1 {
            assert_eq!(stats(&game, nautilus), (4, 1));
        }
    }
    assert_eq!(game.object_view(nautilus).zone, Zone::Graveyard);
}

#[test]
fn hoverstone_pilgrim_bottoms_either_graveyard_card_and_requires_two_for_ward() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Hoverstone Pilgrim", Zone::Battlefield);
    let opposing = put(&mut game, p.other(), "Forest", Zone::Graveyard);
    let own = put(&mut game, p, "Opt", Zone::Graveyard);
    let bounce = put(&mut game, p.other(), "Unsummon", Zone::Hand);
    assert!(game.keywords(source).contains(Keywords::FLYING));
    for (owner, card) in [(p.other(), opposing), (p, own)] {
        game.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        act(&mut game, activate(source));
        act(&mut game, target(card));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.state.players[owner.index()].library[0], card);
        assert_eq!(game.object_view(card).zone, Zone::Library);
    }
    game.state.players[p.other().index()].mana = [0, 1, 0, 0, 0, 0];
    act(&mut game, Action::Pass);
    cast(&mut game, bounce, Some(Target::Card(source)));
    pass_pair(&mut game);
    if game
        .legal_actions()
        .contains(&Action::DeclineResolutionMana)
    {
        act(&mut game, Action::DeclineResolutionMana);
    }
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Battlefield);
}

#[test]
fn huatlis_final_strike_grows_the_first_target_then_uses_its_boosted_power() {
    let p = Player::P0;
    let mut game = scenario(p);
    let yours = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let theirs = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, p, "Huatli's Final Strike", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, target(theirs));
    act(&mut game, target(yours));
    reject(&mut game, target(yours));
    act(&mut game, target(theirs));
    pay(&mut game);

    let mut no_source = game.clone();
    no_source.move_object(yours, Zone::Hand);
    settle(&mut no_source);
    assert_eq!(no_source.object_view(theirs).damage, 0);
    assert_eq!(stats(&no_source, theirs), (3, 3));
    let mut no_recipient = game.clone();
    no_recipient.move_object(theirs, Zone::Hand);
    settle(&mut no_recipient);
    assert_eq!(stats(&no_recipient, yours), (3, 2));

    settle(&mut game);
    assert_eq!(game.object_view(theirs).zone, Zone::Graveyard);
    assert_eq!(game.object_view(yours).damage, 0);
    assert_eq!(stats(&game, yours), (3, 2));
}

#[test]
fn hulking_raptor_adds_two_green_during_only_its_controllers_first_main_phase() {
    let p = Player::P1;
    let mut game = scenario(p);
    put(&mut game, p, "Hulking Raptor", Zone::Battlefield);
    game.state.players[p.index()].mana = [0; 6];
    game.begin_step(Step::Main1);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].mana, [0, 0, 0, 0, 2, 0]);
    game.clear_step_mana(false, false);
    game.begin_step(Step::Main2);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].mana, [0; 6]);
    game.state.active = p.other();
    game.begin_step(Step::Main1);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].mana, [0; 6]);
}

#[test]
fn inti_offers_one_discard_for_the_attack_and_play_permission_ends_at_own_end_step() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = put(
        &mut game,
        p,
        "Inti, Seneschal of the Sun",
        Zone::Battlefield,
    );
    let first = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let discard = put(&mut game, p, "Shock", Zone::Hand);
    let land = put(&mut game, p, "Forest", Zone::Library);
    attack(&mut game, &[first, second]);
    assert_eq!(game.state.stack.len(), 1);
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    settle(&mut declined);
    assert_eq!(declined.top_library_card(p), Some(land));
    assert_eq!(declined.object_view(discard).zone, Zone::Hand);

    act(&mut game, Action::Discard { card: discard });
    for _ in 0..12 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
        {
            break;
        }
        if game.legal_actions().contains(&target(first)) {
            reject(&mut game, target(source));
            act(&mut game, target(first));
        } else {
            choose(&mut game, |action| {
                matches!(action, Action::Pass | Action::PutTrigger { .. })
            });
        }
    }
    assert_eq!(game.counter_count(first, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(second, CounterKind::PlusOne), 0);
    assert!(game.keywords(first).contains(Keywords::TRAMPLE));
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    reject(&mut game, Action::PlayLand { card: land });
    let mut expired = game.clone();
    expired.begin_step(Step::End);
    expired.state.step = Step::Main2;
    reject(&mut expired, Action::PlayLand { card: land });
    game.state.step = Step::Main2;
    act(&mut game, Action::PlayLand { card: land });
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
}

#[test]
fn inti_exiles_one_card_for_a_simultaneous_discard_and_the_spell_pays_its_own_cost() {
    let p = Player::P1;
    let mut game = scenario(p);
    put(
        &mut game,
        p,
        "Inti, Seneschal of the Sun",
        Zone::Battlefield,
    );
    let first = put(&mut game, p, "Forest", Zone::Hand);
    let second = put(&mut game, p, "Island", Zone::Hand);
    let untouched = put(&mut game, p, "Forest", Zone::Library);
    let spell = put(&mut game, p, "Shock", Zone::Library);
    game.discard_batch(&[first, second], None);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.top_library_card(p), Some(untouched));
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    game.state.players[p.index()].mana = [0; 6];
    reject(&mut game, Action::AnnounceExile { card: spell });
    game.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
    act(&mut game, Action::AnnounceExile { card: spell });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(
        game.state.players[p.other().index()].life,
        STARTING_LIFE - 2
    );
}

#[test]
fn ironpaw_aspirant_can_put_its_counter_on_itself_or_an_opposing_creature() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Ironpaw Aspirant", Zone::Hand);
    let opponent = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    let artifact = put(&mut game, p, "Short Bow", Zone::Battlefield);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    reject(&mut game, target(artifact));
    for recipient in [source, opponent] {
        let mut branch = game.clone();
        act(&mut branch, target(recipient));
        settle(&mut branch);
        assert_eq!(branch.counter_count(recipient, CounterKind::PlusOne), 1);
    }
}

#[test]
fn kinjallis_dawnrunner_explores_then_hits_in_both_combat_damage_steps() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Kinjalli's Dawnrunner", Zone::Hand);
    let top = put(&mut game, p, "Shock", Zone::Library);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert!(game.keywords(source).contains(Keywords::DOUBLE_STRIKE));
    let reveal = game.observe(p.other()).explore_choice.unwrap();
    assert_eq!(reveal.revealed.object, game.object_ref(top));
    act(
        &mut game,
        Action::ChooseExplore {
            object: reveal.revealed.object,
            graveyard: true,
        },
    );
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);

    game.state.objects[source as usize].summoning_sick = false;
    attack(&mut game, &[source]);
    combat_to_main_two(&mut game);
    assert_eq!(
        game.state.players[p.other().index()].life,
        STARTING_LIFE - 4
    );
}

#[test]
fn kutzil_blocks_enemy_spells_but_allows_activations_and_draws_once_per_damage_step() {
    let p = Player::P0;
    let opponent = p.other();
    let mut game = scenario(p);
    put(&mut game, p, "Kutzil, Malamet Exemplar", Zone::Battlefield);
    let first = put(&mut game, p, "Kinjalli's Dawnrunner", Zone::Battlefield);
    let second = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    for creature in [first, second] {
        game.add_counter(creature, CounterKind::PlusOne, 1);
    }
    let shock = put(&mut game, opponent, "Shock", Zone::Hand);
    let nautilus = put(&mut game, opponent, "Hermitic Nautilus", Zone::Battlefield);
    let mut enemy_priority = game.clone();
    act(&mut enemy_priority, Action::Pass);
    reject(&mut enemy_priority, Action::AnnounceCast { card: shock });
    act(&mut enemy_priority, activate(nautilus));
    pay(&mut enemy_priority);
    settle(&mut enemy_priority);
    assert_eq!(stats(&enemy_priority, nautilus), (4, 1));
    let mut opponent_turn = game.clone();
    opponent_turn.state.active = opponent;
    opponent_turn.state.priority = opponent;
    assert!(
        opponent_turn
            .legal_actions()
            .contains(&Action::AnnounceCast { card: shock })
    );

    attack(&mut game, &[first, second]);
    combat_to_main_two(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, STARTING_LIFE - 7);
    assert_eq!(game.hand_count(p), 2);
}

#[test]
fn kutzil_compares_damage_sources_against_their_effective_base_power() {
    let p = Player::P1;
    let mut game = scenario(p);
    put(&mut game, p, "Kutzil, Malamet Exemplar", Zone::Battlefield);
    let yearling = put(&mut game, p, "Belligerent Yearling", Zone::Battlefield);
    let dinosaur = put(&mut game, p, "Hulking Raptor", Zone::Hand);
    cast(&mut game, dinosaur, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(stats(&game, yearling), (5, 2));
    attack(&mut game, &[yearling]);
    combat_to_main_two(&mut game);
    assert_eq!(
        game.state.players[p.other().index()].life,
        STARTING_LIFE - 5
    );
    assert_eq!(game.hand_count(p), 0);
}
