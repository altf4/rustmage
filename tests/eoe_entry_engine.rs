//! Shared as-entry sacrifice and land-history rules independent of card admission.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const DEVOUR: CardRules = CardRules {
    effect: Effect::Creature(0, 1),
    extras: &CardExtras {
        devour: Some(DevourSpec {
            kinds: 2,
            counters_per_permanent: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const LAND_HISTORY_PROVIDER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ControlledCreatureEntryCountersForLandsEntered],
    }],
    ..CardRules::EMPTY
};

const POWER_LOOK: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 0,
                count: LibrarySelectionCount::All,
                filter: LibrarySelectionFilter::Kinds(2),
                optional: true,
                reveal: false,
                remainder: LibraryRemainder::Shuffle,
            },
            look: LibraryLook::Amount(AmountValue::SourcePower),
            destination: LibraryPickDestination::BattlefieldTapped,
            public: false,
            maximum_mana_value: None,
        })],
    }],
    ..CardRules::EMPTY
};

fn goal(actor: Player) -> FundingGoal {
    FundingGoal {
        actor,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    }
}

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn choose_devour(game: &mut Game, id: CardId) {
    let object = game.object_ref(id);
    act(game, Action::ChooseEntryDevour { object });
}

#[test]
fn devour_land_paid_cast_and_owned_raw_index_restore() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &DEVOUR)]);
    let player = Player::P0;
    let mut parent = scenario(player);
    let first = put(&mut parent, player, "Forest", Zone::Battlefield);
    let second = put(&mut parent, player, "Plains", Zone::Battlefield);
    let enemy = put(&mut parent, player.other(), "Forest", Zone::Battlefield);
    let nonland = put(&mut parent, player, "Savannah Lions", Zone::Battlefield);
    let entrant = put(&mut parent, player, "Grizzly Bears", Zone::Hand);
    cast(&mut parent, entrant, None);
    resolve_top(&mut parent);
    let choice = parent.observe(player).entry_devour.unwrap();
    assert_eq!(choice.entrant, parent.object_ref(entrant));
    assert_eq!(
        choice.candidates,
        vec![parent.object_ref(first), parent.object_ref(second)]
    );
    for id in [enemy, nonland, entrant] {
        let object = parent.object_ref(id);
        reject(&mut parent, Action::ChooseEntryDevour { object });
    }
    let saved = parent.snapshot();
    let pick = Action::ChooseEntryDevour {
        object: parent.object_ref(first),
    };
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == pick)
        .unwrap();
    drop(parent);

    act(&mut raw, pick.clone());
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.object_view(first).zone, Zone::Battlefield);
    assert_eq!(raw.object_view(entrant).zone, Zone::Stack);
    reject(&mut raw, pick);

    act(&mut raw, Action::FinishEntryDevour);
    settle(&mut raw);
    assert_eq!(raw.object_view(first).zone, Zone::Graveyard);
    assert_eq!(raw.object_view(second).zone, Zone::Battlefield);
    assert_eq!(raw.counter_count(entrant, CounterKind::PlusOne), 3);

    indexed.restore(&saved).unwrap();
    act(&mut indexed, Action::FinishEntryDevour);
    settle(&mut indexed);
    assert_eq!(indexed.object_view(first).zone, Zone::Battlefield);
    assert_eq!(indexed.counter_count(entrant, CounterKind::PlusOne), 0);
}

struct ClearPrevention;

impl Drop for ClearPrevention {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
    }
}

#[test]
fn simultaneous_devour_reserves_old_lands_and_counts_actual_departures() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &DEVOUR)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let a = put(&mut game, player, "Forest", Zone::Battlefield);
    let b = put(&mut game, player, "Island", Zone::Battlefield);
    let incoming_land = put(&mut game, player, "Plains", Zone::Graveyard);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Exile);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Exile);
    put(&mut game, player, "Rest in Peace", Zone::Battlefield);
    let old_a = game.object_ref(a);
    let old_b = game.object_ref(b);
    let _clear = ClearPrevention;
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(old_b)));

    let moves = [first, second, incoming_land]
        .into_iter()
        .map(|id| game.entry_move(id, Zone::Battlefield, false))
        .collect::<Vec<_>>();
    let mut projected = moves.clone();
    for (movement, selected) in projected.iter_mut().zip([old_a, old_b]) {
        movement.devour_preparation = Some(Box::new(entry_devour::Preparation {
            selected: vec![selected],
            closed: true,
            ..Default::default()
        }));
    }
    let world = FundingState::from_game(&game, &goal(player));
    let mut expected_projection = false;
    world.visit_entry_moves(projected, true, None, &mut |next| {
        // With competing replacements the affected player can choose prevention.
        if next.reference(b) == old_b {
            assert_eq!(next.object(a).zone, Zone::Exile);
            assert_eq!(next.counter_count(first, CounterKind::PlusOne), 3);
            assert_eq!(next.counter_count(second, CounterKind::PlusOne), 0);
            expected_projection = true;
        }
        false
    });
    assert!(expected_projection);

    game.stage_entries(moves, None);
    game.advance_entries();
    let incoming = game.object_ref(incoming_land);
    reject(&mut game, Action::ChooseEntryDevour { object: incoming });
    choose_devour(&mut game, a);
    act(&mut game, Action::FinishEntryDevour);
    assert_eq!(game.object_view(a).zone, Zone::Battlefield);
    reject(&mut game, Action::ChooseEntryDevour { object: old_a });
    choose_devour(&mut game, b);
    let prevent = Action::ChooseZoneReplacement {
        effect: ZoneReplacementKind::Prevent,
    };
    if game.legal_actions().contains(&prevent) {
        act(&mut game, prevent);
    }
    settle(&mut game);
    assert_eq!(game.object_view(a).zone, Zone::Exile);
    assert_eq!(game.object_view(b).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(first, CounterKind::PlusOne), 3);
    assert_eq!(game.counter_count(second, CounterKind::PlusOne), 0);
    assert_eq!(game.object_view(incoming_land).zone, Zone::Battlefield);
}

#[test]
fn devour_drops_an_older_entrant_after_a_sacrifice_returns_that_card() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &DEVOUR)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let returning = put(
        &mut game,
        player.other(),
        "Breeding Pool",
        Zone::Battlefield,
    );
    let entrant = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let exiler = game.object_ref(land);
    let victim = game.object_ref(returning);
    game.exile_objects_until_source_leaves_origin(exiler, &[victim], None);
    let exiled = game.object_ref(returning);
    assert_eq!(game.object_view(returning).zone, Zone::Exile);
    let mut moves = vec![
        game.entry_move(entrant, Zone::Battlefield, false),
        game.entry_move(returning, Zone::Battlefield, false),
    ];
    moves[1].controller = player;
    let mut projected = moves.clone();
    projected[0].devour_preparation = Some(Box::new(entry_devour::Preparation {
        selected: vec![exiler],
        closed: true,
        ..Default::default()
    }));
    let world = FundingState::from_game(&game, &goal(player));
    let mut projections = 0;
    world.visit_entry_moves(projected, true, None, &mut |next| {
        projections += 1;
        assert_eq!(next.reference(returning).generation, exiled.generation + 1);
        assert_eq!(next.object(returning).controller, player.other());
        assert_eq!(next.counter_count(entrant, CounterKind::PlusOne), 3);
        false
    });
    assert_eq!(projections, 2);

    game.stage_entries(moves, None);
    game.advance_entries();
    choose_devour(&mut game, land);
    // The last eligible donor closes the selection automatically.
    assert_eq!(game.actor(), Some(player.other()));
    assert_eq!(game.object_view(returning).zone, Zone::Exile);
    act(&mut game, Action::ChooseEntryLife { pay: true });
    settle(&mut game);
    assert_eq!(game.object_ref(returning).generation, exiled.generation + 1);
    assert_eq!(game.object_view(returning).controller, player.other());
    assert!(!game.object_view(returning).tapped);
    assert_eq!(
        game.observe(player).players[player.other().index()].life,
        18
    );
    assert_eq!(game.counter_count(entrant, CounterKind::PlusOne), 3);
}

#[test]
fn land_history_uses_prior_same_controller_entries_and_counter_replacements() {
    let provider = Card::from_name("Impact Tremors").unwrap();
    let _rules = fixture_rules::activate(&[(provider, &LAND_HISTORY_PROVIDER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Forest", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    let opponent_land = put(&mut game, player.other(), "Forest", Zone::Graveyard);
    game.move_object(opponent_land, Zone::Battlefield);
    assert_eq!(
        game.observe(player).players[player.index()].lands_entered_this_turn,
        1
    );
    assert_eq!(
        game.observe(player).players[player.other().index()].lands_entered_this_turn,
        1
    );

    let provider = spawn(&mut game, player, provider, Zone::Hand);
    let simultaneous = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    game.stage_entries(
        vec![
            game.entry_move(provider, Zone::Battlefield, false),
            game.entry_move(simultaneous, Zone::Battlefield, false),
        ],
        None,
    );
    game.advance_entries();
    assert_eq!(game.counter_count(simultaneous, CounterKind::PlusOne), 0);

    put(&mut game, player, "Loading Zone", Zone::Battlefield);
    let land = put(&mut game, player, "Plains", Zone::Graveyard);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    game.stage_entries(
        vec![
            game.entry_move(land, Zone::Battlefield, false),
            game.entry_move(creature, Zone::Battlefield, false),
        ],
        None,
    );
    game.advance_entries();
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 2);

    let next = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, next, None);
    settle(&mut game);
    assert_eq!(game.counter_count(next, CounterKind::PlusOne), 4);
    game.begin_turn();
    assert_eq!(
        game.observe(player).players[player.index()].lands_entered_this_turn,
        0
    );
}

#[test]
fn projected_entry_lands_are_visible_to_subsequent_entry_batches_only() {
    let provider = Card::from_name("Impact Tremors").unwrap();
    let _rules = fixture_rules::activate(&[(provider, &LAND_HISTORY_PROVIDER)]);
    let player = Player::P1;
    let mut game = scenario(player);
    spawn(&mut game, player, provider, Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Exile);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Exile);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Exile);
    let world = FundingState::from_game(&game, &goal(player));
    let actual_before = game.snapshot();
    let mut verified = false;
    world.visit_entry_moves(
        vec![
            game.entry_move(land, Zone::Battlefield, false),
            game.entry_move(first, Zone::Battlefield, false),
        ],
        true,
        None,
        &mut |first_batch| {
            assert_eq!(first_batch.counter_count(first, CounterKind::PlusOne), 0);
            first_batch.visit_entry_moves(
                vec![game.entry_move(second, Zone::Battlefield, false)],
                true,
                None,
                &mut |second_batch| {
                    assert_eq!(second_batch.counter_count(second, CounterKind::PlusOne), 1);
                    verified = true;
                    true
                },
            )
        },
    );
    assert!(verified);
    assert_eq!(game.snapshot(), actual_before);
}

#[test]
fn power_scaled_private_library_look_uses_latest_departure_power() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &POWER_LOOK)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    let boost = put(&mut game, player, "Giant Growth", Zone::Hand);
    let removal = put(&mut game, player.other(), "Murder", Zone::Hand);
    cast(&mut game, creature, None);
    resolve_top(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    cast(&mut game, boost, Some(Target::Card(creature)));
    resolve_top(&mut game);
    assert_eq!(stats(&game, creature), (5, 5));
    act(&mut game, Action::Pass);
    cast(&mut game, removal, Some(Target::Card(creature)));
    resolve_top(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
    resolve_top(&mut game);

    let view = game.observe(player).library_selection.unwrap();
    assert_eq!(view.remaining.len(), 5);
    assert!(game.observe(player.other()).library_selection.is_none());
    let chosen = view.candidates[0];
    act(&mut game, Action::SelectLibrary { card: chosen.card });
    act(&mut game, Action::FinishLibrarySelection);
    assert_eq!(game.object_view(chosen.card).zone, Zone::Battlefield);
    assert!(game.object_view(chosen.card).tapped);
}
