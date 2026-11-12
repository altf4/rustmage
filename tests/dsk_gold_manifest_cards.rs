//! Paid scenarios for the green and multicolored Duskmourn manifest family.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardRules, object};
use rustmage_core::{cards::*, engine::*};
use support::*;

fn cast_named(game: &mut Game, player: Player, name: &str) -> CardId {
    let card = put(game, player, name, Zone::Hand);
    cast(game, card, None);
    card
}

fn progress_until(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
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

fn settle_triggers(game: &mut Game) {
    progress_until(game, |game| {
        game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.resolving.is_none()
    });
}

fn reach_manifest(game: &mut Game, player: Player) {
    progress_until(game, |game| game.observe(player).manifest_choice.is_some());
    assert_eq!(game.actor(), Some(player));
}

fn manifest(game: &mut Game, card: CardId) {
    let object = game.object_ref(card);
    act(game, Action::ChooseManifest { object });
}

fn turn_face_up(game: &mut Game, card: CardId) {
    let object = game.object_ref(card);
    choose(
        game,
        |action| matches!(action, Action::BeginFaceUp { object: offered, .. } if *offered == object),
    );
    choose(game, |action| matches!(action, Action::PayFaceUp { .. }));
}

fn top_pair(game: &mut Game, player: Player, lower: &str, upper: &str) -> [CardId; 2] {
    [lower, upper].map(|name| put(game, player, name, Zone::Library))
}

#[test]
fn manifest_dread_cast_owns_the_private_choice_and_turns_up_for_mana() {
    let player = Player::P1;
    let mut game = scenario(player);
    let [lower, upper] = top_pair(&mut game, player, "Shock", "Bear Cub");
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 1];
    let spell = cast_named(&mut game, player, "Manifest Dread");
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    reach_manifest(&mut game, player);
    let choice = game.observe(player).manifest_choice.unwrap();
    assert_eq!(
        choice
            .candidates
            .iter()
            .map(|(reference, _)| reference.card)
            .collect::<Vec<_>>(),
        [upper, lower]
    );
    assert!(game.observe(player.other()).manifest_choice.is_none());

    let action = Action::ChooseManifest {
        object: game.object_ref(upper),
    };
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let saved = game.snapshot();
    let mut alternate = game.clone();
    let mut raw = game.clone();
    act(&mut raw, action);
    game.apply_index(player, game.revision(), index).unwrap();
    assert_eq!(game.snapshot(), raw.snapshot());
    alternate.restore(&saved).unwrap();
    manifest(&mut alternate, lower);
    assert_eq!(alternate.object_view(upper).zone, Zone::Graveyard);
    assert!(alternate.is_face_down(alternate.object_ref(lower)));

    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(game.object_view(lower).zone, Zone::Graveyard);
    let reference = game.object_ref(upper);
    assert_eq!(stats(&game, upper), (2, 2));
    assert_eq!(
        game.observe(player.other()).face_down_permanents[0].underlying,
        None
    );
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 1];
    turn_face_up(&mut game, upper);
    assert_eq!(game.object_ref(upper), reference);
    assert!(!game.is_face_down(reference));
    assert_eq!(game.object_view(upper).name, "Bear Cub");
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn bashful_beastie_manifests_on_death_and_has_no_trigger_on_exile() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = cast_named(&mut game, player, "Bashful Beastie");
    settle(&mut game);
    assert_eq!(stats(&game, source), (5, 4));
    let [milled, facedown] = top_pair(&mut game, player, "Shock", "Bear Cub");
    let removal = put(&mut game, player, "Murder", Zone::Hand);
    let mut exile = game.clone();
    exile.add_counter(source, CounterKind::Finality, 1);
    cast(&mut exile, removal, Some(Target::Card(source)));
    settle(&mut exile);
    assert_eq!(exile.object_view(source).zone, Zone::Exile);
    assert_eq!(exile.object_view(facedown).zone, Zone::Library);

    cast(&mut game, removal, Some(Target::Card(source)));
    reach_manifest(&mut game, player);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    manifest(&mut game, facedown);
    assert_eq!(game.object_view(milled).zone, Zone::Graveyard);
    assert!(game.is_face_down(game.object_ref(facedown)));
}

#[test]
fn defiant_survivor_checks_tapped_status_again_in_the_second_main_phase() {
    let player = Player::P0;
    let mut game = scenario(player);
    game.begin_step(Step::Main1);
    let survivor = cast_named(&mut game, player, "Defiant Survivor");
    settle(&mut game);
    let [milled, facedown] = top_pair(&mut game, player, "Forest", "Bear Cub");
    let vehicle = put(
        &mut game,
        player,
        "Thunderous Velocipede",
        Zone::Battlefield,
    );
    act(
        &mut game,
        Action::AnnounceActivate {
            card: vehicle,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    let source = game.object_ref(survivor);
    act(&mut game, Action::ChooseCrewer { object: source });
    act(&mut game, Action::FinishCrew);
    settle(&mut game);
    assert!(game.object_view(survivor).tapped);

    progress_until(&mut game, |game| game.state.step == Step::Main2);
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(game.state.stack[0].source.unwrap().object, source);
    let mut untapped = game.clone();
    untapped.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
    let trick = put(&mut untapped, player, "Shore Up", Zone::Hand);
    cast(&mut untapped, trick, Some(Target::Card(survivor)));
    settle(&mut untapped);
    assert!(!untapped.object_view(survivor).tapped);
    assert_eq!(untapped.object_view(facedown).zone, Zone::Library);

    reach_manifest(&mut game, player);
    manifest(&mut game, facedown);
    assert_eq!(game.object_view(milled).zone, Zone::Graveyard);
    assert!(game.is_face_down(game.object_ref(facedown)));
}

#[test]
fn growing_dread_checks_the_actor_while_cryptid_inspector_checks_control() {
    // A paid fixture effect lets one player turn the other player's permanent up.
    const TURN_OPPONENT_UP: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                base: &object(0),
                predicate: &TargetPredicate::FaceDown,
            })),
            &[
                Instruction::RevealFaceDownTargetAndMayTurnUp,
                Instruction::TurnTargetFaceUp,
            ],
        ),
        ..CardRules::EMPTY
    };
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(opt, &TURN_OPPONENT_UP)]);
    let controller = Player::P1;
    let actor = controller.other();
    let mut game = scenario(controller);
    let inspector = cast_named(&mut game, controller, "Cryptid Inspector");
    settle(&mut game);
    assert!(
        game.object_view(inspector)
            .keywords
            .contains(Keywords::VIGILANCE)
    );

    let [_, second] = top_pair(&mut game, controller, "Shock", "Cryptid Inspector");
    cast_named(&mut game, controller, "Manifest Dread");
    reach_manifest(&mut game, controller);
    manifest(&mut game, second);
    settle_triggers(&mut game);
    assert_eq!(game.counter_count(inspector, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(second, CounterKind::PlusOne), 0);

    let [milled, revealed] = top_pair(&mut game, actor, "Forest", "Llanowar Elves");
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(actor));
    let dread = cast_named(&mut game, actor, "Growing Dread");
    reach_manifest(&mut game, actor);
    manifest(&mut game, revealed);
    settle_triggers(&mut game);
    assert_eq!(game.object_view(dread).zone, Zone::Battlefield);
    assert_eq!(game.object_view(milled).zone, Zone::Graveyard);
    assert!(game.is_face_down(game.object_ref(revealed)));
    assert_eq!(game.counter_count(inspector, CounterKind::PlusOne), 1);

    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(actor));
    let turn_up = put(&mut game, actor, "Opt", Zone::Hand);
    cast(&mut game, turn_up, Some(Target::Card(second)));
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    act(&mut game, Action::ChooseOptional { accept: true });
    settle_triggers(&mut game);
    assert_eq!(game.object_view(second).controller, controller);
    assert_eq!(game.object_view(second).name, "Cryptid Inspector");
    assert_eq!(game.counter_count(inspector, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(second, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(revealed, CounterKind::PlusOne), 0);
}

fn bookworm_prompt(game: &mut Game, player: Player) {
    progress_until(game, |game| {
        matches!(
            game.state.pending,
            Pending::EffectOptional { player: actor, .. } if actor == player
        )
    });
    assert_eq!(game.state.step, Step::End);
}

#[test]
fn oblivious_bookworm_can_decline_or_draw_and_discard_without_history() {
    let player = Player::P0;
    let mut game = scenario(player);
    cast_named(&mut game, player, "Oblivious Bookworm");
    settle(&mut game);
    let old_hand = put(&mut game, player, "Shock", Zone::Hand);
    let drawn = put(&mut game, player, "Forest", Zone::Library);
    bookworm_prompt(&mut game, player);
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    settle(&mut decline);
    assert_eq!(decline.object_view(drawn).zone, Zone::Library);
    assert_eq!(decline.object_view(old_hand).zone, Zone::Hand);

    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.object_view(drawn).zone, Zone::Hand);
    assert_eq!(game.actor(), Some(player));
    act(&mut game, Action::Discard { card: old_hand });
    settle(&mut game);
    assert_eq!(game.object_view(old_hand).zone, Zone::Graveyard);
}

#[test]
fn oblivious_bookworm_skips_discard_for_either_independent_turn_history() {
    let player = Player::P1;
    for prior_turn in [false, true] {
        let mut game = scenario(player);
        let [_, facedown] = top_pair(&mut game, player, "Shock", "Bear Cub");
        cast_named(&mut game, player, "Manifest Dread");
        reach_manifest(&mut game, player);
        manifest(&mut game, facedown);
        settle(&mut game);
        if prior_turn {
            let turn = game.state.turn;
            progress_until(&mut game, |game| {
                game.state.turn > turn
                    && game.state.active == player
                    && game.state.step == Step::Main1
            });
            assert!(
                !game.state.players[player.index()]
                    .turn_facts
                    .face_down_entry_or_turn_up
            );
            game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 1];
            turn_face_up(&mut game, facedown);
            game.state.players[player.index()].mana = [0, 1, 0, 0, 1, 0];
        }
        cast_named(&mut game, player, "Oblivious Bookworm");
        settle(&mut game);
        let old_hand = put(&mut game, player, "Shock", Zone::Hand);
        let drawn = put(&mut game, player, "Forest", Zone::Library);
        bookworm_prompt(&mut game, player);
        act(&mut game, Action::ChooseOptional { accept: true });
        settle(&mut game);
        assert_eq!(game.object_view(drawn).zone, Zone::Hand);
        assert_eq!(game.object_view(old_hand).zone, Zone::Hand);
    }
}

#[test]
fn under_the_skin_offers_any_own_permanent_in_the_graveyard_and_can_decline() {
    let player = Player::P0;
    let mut game = scenario(player);
    let old = put(&mut game, player, "Short Bow", Zone::Graveyard);
    let opponent = put(&mut game, player.other(), "Bear Cub", Zone::Graveyard);
    let instant = put(&mut game, player, "Shock", Zone::Graveyard);
    let [new, facedown] = top_pair(&mut game, player, "Forest", "Llanowar Elves");
    cast_named(&mut game, player, "Under the Skin");
    reach_manifest(&mut game, player);
    manifest(&mut game, facedown);
    assert_eq!(game.object_view(new).zone, Zone::Graveyard);
    let select = |card| Action::SelectGraveyardReturn {
        object: game.object_ref(card),
    };
    let first = select(old);
    let second = select(new);
    let illegal = [select(opponent), select(instant)];
    for action in illegal {
        reject(&mut game, action);
    }

    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    assert_eq!(decline.object_view(old).zone, Zone::Graveyard);
    assert_eq!(decline.object_view(new).zone, Zone::Graveyard);
    let mut older = game.clone();
    act(&mut older, first);
    assert_eq!(older.object_view(old).zone, Zone::Hand);
    assert_eq!(older.object_view(new).zone, Zone::Graveyard);
    act(&mut game, second);
    assert_eq!(game.object_view(new).zone, Zone::Hand);
    assert_eq!(game.object_view(old).zone, Zone::Graveyard);
    assert!(game.is_face_down(game.object_ref(facedown)));
}

#[test]
fn threats_around_every_corner_searches_only_for_controlled_face_down_entries() {
    let player = Player::P0;
    let mut game = scenario(player);
    let found = put(&mut game, player, "Island", Zone::Library);
    let ineligible = put(&mut game, player, "Thornwood Falls", Zone::Library);
    let [milled, facedown] = top_pair(&mut game, player, "Shock", "Bear Cub");
    cast_named(&mut game, player, "Threats Around Every Corner");
    reach_manifest(&mut game, player);
    manifest(&mut game, facedown);
    progress_until(&mut game, |game| {
        game.observe(player).search_choice.is_some()
    });
    assert!(game.observe(player.other()).search_choice.is_none());
    assert_eq!(game.object_view(milled).zone, Zone::Graveyard);
    reject(&mut game, Action::ChooseSearch { card: ineligible });
    let library_before = game.state.players[player.index()]
        .library
        .iter()
        .copied()
        .filter(|card| *card != found)
        .collect::<Vec<_>>();
    act(&mut game, Action::ChooseSearch { card: found });
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert_eq!(game.object_view(found).zone, Zone::Battlefield);
    assert!(game.object_view(found).tapped);
    assert_ne!(
        game.state.players[player.index()].library.as_slice(),
        library_before.as_slice()
    );

    turn_face_up(&mut game, facedown);
    settle(&mut game);
    let [_, theirs] = top_pair(&mut game, player.other(), "Forest", "Bear Cub");
    act(&mut game, Action::Pass);
    cast_named(&mut game, player.other(), "Growing Dread");
    reach_manifest(&mut game, player.other());
    manifest(&mut game, theirs);
    settle(&mut game);
    assert!(game.is_face_down(game.object_ref(theirs)));

    let [_, another] = top_pair(&mut game, player, "Shock", "Bear Cub");
    cast_named(&mut game, player, "Manifest Dread");
    reach_manifest(&mut game, player);
    manifest(&mut game, another);
    progress_until(&mut game, |game| {
        game.observe(player).search_choice.is_some()
    });
    let library_before = game.state.players[player.index()].library.clone();
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert_ne!(game.state.players[player.index()].library, library_before);
    assert_eq!(game.object_view(ineligible).zone, Zone::Library);
}

#[test]
fn valgavoths_onslaught_pays_double_x_and_counters_only_its_two_results() {
    let player = Player::P1;
    let mut game = scenario(player);
    let [_, old] = top_pair(&mut game, player, "Forest", "Bear Cub");
    cast_named(&mut game, player, "Manifest Dread");
    reach_manifest(&mut game, player);
    manifest(&mut game, old);
    settle(&mut game);

    let [bottom, second] = top_pair(&mut game, player, "Forest", "Llanowar Elves");
    let [middle, first] = top_pair(&mut game, player, "Shock", "Grizzly Bears");
    let spell = put(&mut game, player, "Valgavoth's Onslaught", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 4];
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, Action::ChooseX { value: 3 });
    act(&mut game, Action::ChooseX { value: 2 });
    pay(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);

    reach_manifest(&mut game, player);
    manifest(&mut game, first);
    assert_eq!(game.counter_count(first, CounterKind::PlusOne), 0);
    assert_eq!(game.object_view(middle).zone, Zone::Graveyard);
    assert!(game.observe(player.other()).manifest_choice.is_none());
    reject(&mut game, Action::Pass);
    manifest(&mut game, second);
    settle(&mut game);
    assert_eq!(game.object_view(bottom).zone, Zone::Graveyard);
    assert_eq!(game.counter_count(old, CounterKind::PlusOne), 0);
    for result in [first, second] {
        assert!(game.is_face_down(game.object_ref(result)));
        assert_eq!(game.counter_count(result, CounterKind::PlusOne), 2);
        assert_eq!(stats(&game, result), (4, 4));
    }
}
