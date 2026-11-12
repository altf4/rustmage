//! Signed, effective powers constrain a private selection from a public reveal.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const RIP: Card = Card::GrizzlyBears;
const RIP_RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SURVIVAL,
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 0,
                count: LibrarySelectionCount::DistinctPower,
                filter: LibrarySelectionFilter::AnyOf(&[
                    LibrarySelectionFilter::Kinds(1),
                    LibrarySelectionFilter::Subtype("Vehicle"),
                ]),
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Amount(AmountValue::SourcePower),
            destination: LibraryPickDestination::Hand,
            public: true,
            maximum_mana_value: None,
        })],
    }],
    ..CardRules::EMPTY
};

const NEGATIVE: CardRules = CardRules {
    effect: Effect::Creature(-2, 3),
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn trigger_survival(game: &mut Game, source: CardId) {
    game.begin_step(Step::Main1);
    game.state.objects[source as usize].tapped = true;
    game.begin_step(Step::Main2);
    game.prepare_triggers();
    assert_eq!(game.state.stack.len(), 1);
}

fn select(card: CardId) -> Action {
    Action::SelectLibrary { card }
}

#[test]
fn creature_and_vehicle_powers_share_one_signed_selection() {
    let negative = Card::from_name("Hill Giant").unwrap();
    let _rules = fixture_rules::activate(&[(RIP, &RIP_RULES), (negative, &NEGATIVE)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = spawn(&mut game, player, RIP, Zone::Battlefield);
    for _ in 0..3 {
        put(&mut game, player, "Forest", Zone::Battlefield);
    }
    let dormant = put(&mut game, player, "Rover Blades", Zone::Battlefield);
    let hidden = put(&mut game, player, "Opt", Zone::Library);
    let instant = put(&mut game, player, "Shock", Zone::Library);
    let zero = put(&mut game, player, "Aegis Turtle", Zone::Library);
    let dynamic = put(&mut game, player, "Lumbering Worldwagon", Zone::Library);
    let vehicle = put(&mut game, player, "Rover Blades", Zone::Library);
    let two = put(&mut game, player, "Bear Cub", Zone::Library);
    let equal_negative = spawn(&mut game, player, negative, Zone::Library);
    let below_zero = spawn(&mut game, player, negative, Zone::Library);
    let cohort = [
        below_zero,
        equal_negative,
        two,
        vehicle,
        dynamic,
        zero,
        instant,
    ];

    trigger_survival(&mut game, source);
    let boost = put(&mut game, player, "Giant Growth", Zone::Hand);
    game.state.players[player.index()].mana = [8; 6];
    cast(&mut game, boost, Some(Target::Card(source)));
    pass_pair(&mut game);
    assert_eq!(stats(&game, source).0, 7);
    game.move_object(source, Zone::Graveyard);
    pass_pair(&mut game);

    let revealed = game.observe(opponent).library_selection.unwrap();
    assert_eq!(
        revealed.remaining.iter().map(|r| r.id).collect::<Vec<_>>(),
        cohort
    );
    assert!(revealed.candidates.is_empty());
    assert!(revealed.selected.is_empty());
    assert_eq!(stats(&game, dormant), (0, 0));
    assert_eq!(stats(&game, vehicle), (2, 2));
    assert_eq!(stats(&game, dynamic), (3, 4));
    assert_eq!(stats(&game, below_zero), (-2, 3));

    let view = game.observe(player).library_selection.unwrap();
    assert_eq!(view.maximum, 4);
    assert_eq!(view.candidates.len(), 6);
    reject(&mut game, select(hidden));
    reject(&mut game, select(instant));

    let saved = game.snapshot();
    let mut indexed = game.clone();
    let pick = select(below_zero);
    let index = indexed
        .legal_actions()
        .iter()
        .position(|action| *action == pick)
        .unwrap();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, pick);
    assert_eq!(game.snapshot(), indexed.snapshot());

    let after_private_pick = game.observe(opponent).library_selection.unwrap();
    assert_eq!(
        after_private_pick
            .remaining
            .iter()
            .map(|r| r.id)
            .collect::<Vec<_>>(),
        cohort
    );
    assert!(after_private_pick.candidates.is_empty());
    assert!(after_private_pick.selected.is_empty());
    reject(&mut game, select(equal_negative));
    act(&mut game, select(vehicle));
    reject(&mut game, select(two));
    act(&mut game, select(dynamic));
    act(&mut game, select(zero));
    act(&mut game, Action::FinishLibrarySelection);

    let public = game.observe(opponent);
    for kept in [below_zero, vehicle, dynamic, zero] {
        assert_eq!(game.object_view(kept).zone, Zone::Hand);
        assert!(public.known_hand.contains(&game.object_ref(kept)));
    }
    let library = &game.state.players[player.index()].library;
    assert_eq!(library.last(), Some(&hidden));
    for bottom in [equal_negative, two, instant] {
        assert!(library[..3].contains(&bottom));
    }

    indexed.restore(&saved).unwrap();
    act(&mut indexed, Action::FinishLibrarySelection);
    assert_eq!(indexed.hand_count(player), 0);
    let declined_library = &indexed.state.players[player.index()].library;
    assert_eq!(declined_library.last(), Some(&hidden));
    assert!(cohort.iter().all(|id| declined_library[..7].contains(id)));
}

#[test]
fn zero_source_power_reveals_nothing() {
    let _rules = fixture_rules::activate(&[(RIP, &RIP_RULES)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, RIP, Zone::Battlefield);
    let top = put(&mut game, player, "Rover Blades", Zone::Library);
    game.state.objects[source as usize].growth.power = -4;

    trigger_survival(&mut game, source);
    settle(&mut game);
    assert_eq!(
        game.state.players[player.index()].library.last(),
        Some(&top)
    );
    assert!(game.observe(player).library_selection.is_none());
    assert!(game.observe(player.other()).library_selection.is_none());
    assert_eq!(game.hand_count(player), 0);
}
