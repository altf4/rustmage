//! Optional land copies use the owned entry pipeline and flat copiable characteristics.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::{cards::*, engine::*};
use support::*;

const COLORLESS: Mana = [0, 0, 0, 0, 0, 1];
const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const COPIER: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed(COLORLESS))],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        entry_copy: Some(EntryCopySpec {
            filters: &[],
            optional: true,
        }),
        entry_copy_options: Some(&EntryCopyOptions {
            source: EntryCopySource::GraveyardLand { tapped: true },
            exceptions: CopyExceptions::CAVE,
            ..EntryCopyOptions::DEFAULT
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn definition() -> Card {
    Card::from_name("Promising Vein").unwrap()
}

fn copy(object: ObjectRef) -> Action {
    Action::ChooseEntryCopy {
        object: Some(object),
    }
}

#[test]
fn the_land_play_can_decline_or_copy_from_either_graveyard_with_all_entry_abilities() {
    let _rules = fixture_rules::activate(&[(definition(), &COPIER)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let land = put(&mut game, player, "Promising Vein", Zone::Hand);
    let island = put(&mut game, opponent, "Island", Zone::Graveyard);
    let shock_land = put(&mut game, player, "Watery Grave", Zone::Graveyard);
    let surveil_land = put(&mut game, opponent, "Conduit Pylons", Zone::Graveyard);
    let invalid = put(&mut game, opponent, "Grizzly Bears", Zone::Graveyard);
    let follower = put(&mut game, player, "Forest", Zone::Battlefield);

    let island_ref = game.object_ref(island);
    let mut spelunking = game.clone();
    put(&mut spelunking, player, "Spelunking", Zone::Battlefield);
    act(&mut spelunking, Action::PlayLand { card: land });
    act(&mut spelunking, copy(island_ref));
    assert!(spelunking.has_subtype(land, "Cave"));
    assert!(!spelunking.object_view(land).tapped);

    act(&mut game, Action::PlayLand { card: land });
    let original = game.object_ref(land);
    let shock_ref = game.object_ref(shock_land);
    let surveil_ref = game.object_ref(surveil_land);
    for viewer in [player, opponent] {
        let choice = game.observe(viewer).entry_copy.unwrap();
        assert_eq!(choice.entrant, original);
        assert_eq!(choice.candidates, [island_ref, shock_ref, surveil_ref]);
        assert!(choice.optional);
    }
    let invalid = copy(game.object_ref(invalid));
    reject(&mut game, invalid);

    let mut declined = game.clone();
    act(&mut declined, Action::ChooseEntryCopy { object: None });
    assert_eq!(declined.object_view(land).card, definition());
    assert!(declined.has_subtype(land, "Cave"));
    assert!(!declined.object_view(land).tapped);
    act(
        &mut declined,
        Action::ActivateMana {
            card: land,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: COLORLESS,
        },
    );

    let mut life_choice = game.clone();
    act(&mut life_choice, copy(shock_ref));
    assert!(matches!(life_choice.state.pending, Pending::EntryLife { player: p } if p == player));
    act(&mut life_choice, Action::ChooseEntryLife { pay: true });
    let view = life_choice.object_view(land);
    assert_eq!(view.name, "Watery Grave");
    assert!(view.tapped);
    assert_eq!(life_choice.state.players[player.index()].life, 18);
    for subtype in ["Island", "Swamp", "Cave"] {
        assert!(view.has_subtype(subtype));
    }

    let mut triggered = game.clone();
    act(&mut triggered, copy(surveil_ref));
    assert!(triggered.object_view(land).tapped);
    assert!(triggered.has_subtype(land, "Desert"));
    assert!(triggered.has_subtype(land, "Cave"));
    act(&mut triggered, Action::Pass);
    act(&mut triggered, Action::Pass);
    let choice = triggered.observe(player).library_choice.unwrap();
    assert!(choice.surveil);
    assert!(triggered.observe(opponent).library_choice.is_none());
    choose(&mut triggered, |a| {
        matches!(
            a,
            Action::ArrangeLibrary {
                destination: LibraryDestination::Graveyard,
                ..
            }
        )
    });
    settle(&mut triggered);
    assert_eq!(triggered.object_ref(surveil_land), surveil_ref);
    assert_eq!(triggered.object_view(surveil_land).zone, Zone::Graveyard);

    let saved = game.snapshot();
    let action = copy(island_ref);
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, action.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
    let view = game.object_view(land);
    assert_eq!(view.name, "Island");
    assert_eq!(view.owner, player);
    assert_eq!(view.controller, player);
    assert!(view.card.is_basic_land());
    assert!(view.tapped);
    assert!(view.has_subtype("Island"));
    assert!(view.has_subtype("Cave"));
    assert_eq!(game.object_ref(island), island_ref);

    let recipe = game.copiable_recipe(view.reference);
    game.apply_temporary_copy(game.object_ref(follower), recipe, CopyDeadline::EndOfTurn);
    assert!(game.has_subtype(follower, "Cave"));
    assert!(!game.object_view(follower).tapped);
    act(&mut game, Action::TapMana { card: follower });
    let lots = game.observe(player).mana_lots;
    assert_eq!(lots[0].amounts, BLUE);
    assert!(lots[0].cave_source);
    game.move_object(land, Zone::Graveyard);
    assert_eq!(game.object_view(land).card, definition());
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
}

#[test]
fn simultaneous_graveyard_entries_and_self_copy_match_projected_entry() {
    let _rules = fixture_rules::activate(&[(definition(), &COPIER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Promising Vein", Zone::Graveyard);
    let donor = put(&mut game, player.other(), "Forest", Zone::Graveyard);
    let moves = vec![
        game.entry_move(land, Zone::Battlefield, false),
        game.entry_move(donor, Zone::Battlefield, false),
    ];
    let goal = FundingGoal {
        actor: player,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    };
    let world = FundingState::from_game(&game, &goal);
    let mut preview = None;
    world.visit_entry_moves(moves.clone(), true, None, &mut |next| {
        if next.object(land).card == Card::Forest {
            assert_eq!(next.object(donor).zone, Zone::Battlefield);
            preview =
                Some(characteristic_query::CharacteristicQuery::in_world(next).object_view(land));
            return true;
        }
        false
    });

    game.stage_entries(moves, None);
    assert!(game.advance_entries());
    let mut same_definition = game.clone();
    let self_ref = same_definition.object_ref(land);
    act(&mut same_definition, copy(self_ref));
    assert_eq!(same_definition.state.pending, Pending::Priority);
    assert_eq!(same_definition.object_view(land).card, definition());
    assert!(same_definition.object_view(land).tapped);

    let donor_ref = game.object_ref(donor);
    act(&mut game, copy(donor_ref));
    let actual = game.object_view(land);
    let expected = preview.expect("copied land is a legal projected entry");
    assert_eq!(actual.name, expected.name);
    assert_eq!(actual.kinds, expected.kinds);
    assert_eq!(actual.tapped, expected.tapped);
    assert_eq!(actual.granted_subtypes, expected.granted_subtypes);
    assert!(actual.tapped);
    assert!(actual.has_subtype("Cave"));
    assert!(actual.has_subtype("Forest"));
    assert_eq!(game.object_view(donor).zone, Zone::Battlefield);
}
