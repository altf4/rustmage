//! Paid exercises of replacement-created token groups and their exact source departure.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const TREASURE: Card = Card(5302);
const SOURCE_NAME: &str = "Mysterio's Phantasm";
const TOKEN_DEPARTURE: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokensWithSourceDeparture {
                token: TREASURE,
                amount: AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Villain"),
                    basic: false,
                    other: false,
                    cards_only: true,
                })),
                ability: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::DelayedTargetEvent,
            targets: TargetPlan::NONE,
            program: &[Instruction::ExileDelayedCreatedTokens],
        },
    ],
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn enter_source(game: &mut Game, player: Player) -> ObjectRef {
    let source = put(game, player, SOURCE_NAME, Zone::Hand);
    cast(game, source, None);
    pass_pair(game);
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(game.object_view(source).zone, Zone::Battlefield);
    game.object_ref(source)
}

fn treasures(game: &Game) -> Vec<ObjectRef> {
    game.observe(Player::P0)
        .objects
        .into_iter()
        .filter(|object| object.zone == Zone::Battlefield && object.card == TREASURE)
        .map(|object| object.reference)
        .collect()
}

fn source_watch(game: &Game, source: ObjectRef, player: Player) -> Vec<ObjectRef> {
    let rows = game.observe(player).delayed_target_events.unwrap();
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.subject, source);
    assert_eq!(row.event, DelayedTargetEvent::LeavesBattlefieldOnce);
    assert_eq!(row.expires_turn, None);
    assert_eq!(row.packet.controller, player);
    assert!(row.packet.targets.is_empty());
    row.packet.reflexive_action_objects().collect()
}

fn advance_main(game: &mut Game, player: Player) {
    for _ in 0..72 {
        if game.state.step == Step::Main1 && game.state.active == player {
            assert_eq!(game.actor(), Some(player));
            game.state.players[player.index()].mana = [8; 6];
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    panic!("next main phase did not arrive");
}

#[test]
fn exact_created_batch_is_exiled_once_after_control_ability_and_turn_changes() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name(SOURCE_NAME).unwrap(), &TOKEN_DEPARTURE)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    put(
        &mut game,
        player,
        "Inner Demons Gangsters",
        Zone::Battlefield,
    );
    put(
        &mut game,
        opponent,
        "Inner Demons Gangsters",
        Zone::Battlefield,
    );
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    let source = enter_source(&mut game, player);
    settle(&mut game);
    let linked = source_watch(&game, source, player);
    assert_eq!(linked.len(), 4);
    assert_eq!(linked, treasures(&game));

    let destroyed = linked[0];
    let disenchant = put(&mut game, player, "Disenchant", Zone::Hand);
    cast(&mut game, disenchant, Some(Target::Card(destroyed.card)));
    settle(&mut game);
    assert_eq!(game.object_view(destroyed.card).zone, Zone::Outside);
    advance_main(&mut game, opponent);
    assert_eq!(source_watch(&game, source, player), linked);
    let blank = put(&mut game, opponent, "Witness Protection", Zone::Hand);
    cast(&mut game, blank, Some(Target::Card(source.card)));
    settle(&mut game);
    assert_eq!(game.object_ref(source.card), source);
    assert!(!game.printed_abilities_available(source.card));
    let steal = put(&mut game, opponent, "Involuntary Employment", Zone::Hand);
    cast(&mut game, steal, Some(Target::Card(source.card)));
    settle(&mut game);
    assert_eq!(game.object_view(source.card).controller, opponent);
    let unrelated = game.object_view(destroyed.card);
    assert_eq!(unrelated.zone, Zone::Battlefield);
    assert_eq!(unrelated.card, TREASURE);
    assert_eq!(unrelated.controller, opponent);
    assert_ne!(unrelated.reference, destroyed);

    let unsummon = put(&mut game, opponent, "Unsummon", Zone::Hand);
    cast(&mut game, unsummon, Some(Target::Card(source.card)));
    act(&mut game, Action::Pass);
    let saved = game.snapshot();
    let revision = game.revision();
    let pass_index = game
        .legal_actions()
        .iter()
        .position(|action| *action == Action::Pass)
        .unwrap();
    assert_eq!(game.apply(opponent, Action::Pass), Err(Error::WrongActor));
    assert_eq!(game.snapshot(), saved);
    let mut indexed = game.clone();
    indexed.apply_index(player, revision, pass_index).unwrap();
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, pass_index),
        Err(Error::StaleRevision)
    );
    indexed
        .apply_index(player, indexed.revision(), pass_index)
        .unwrap();
    act(&mut game, Action::Pass);
    assert_eq!(indexed.snapshot(), game.snapshot());
    assert_eq!(game.state.stack.len(), 1);
    let exile = &game.state.stack[0];
    assert_eq!(exile.controller, player);
    assert_eq!(exile.source.unwrap().object, source);
    assert_eq!(exile.reflexive_action_objects().collect::<Vec<_>>(), linked);
    assert!(game.observe(player).delayed_target_events.is_none());
    assert_eq!(game.object_view(source.card).owner, player);
    assert_eq!(game.object_view(source.card).zone, Zone::Hand);
    settle(&mut game);
    assert_eq!(treasures(&game), [unrelated.reference]);
}

#[test]
fn blinking_before_creation_leaves_the_old_batch_unlinked() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name(SOURCE_NAME).unwrap(), &TOKEN_DEPARTURE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(
        &mut game,
        player,
        "Inner Demons Gangsters",
        Zone::Battlefield,
    );
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    let old_source = enter_source(&mut game, player);
    let blink = put(&mut game, player, "Personify", Zone::Hand);
    cast(&mut game, blink, Some(Target::Card(old_source.card)));
    settle(&mut game);
    let new_source = game.object_ref(old_source.card);
    assert_ne!(old_source, new_source);
    let linked = source_watch(&game, new_source, player);
    assert_eq!(linked.len(), 4);
    let all = treasures(&game);
    assert_eq!(all.len(), 8);
    let unlinked: Vec<_> = all
        .into_iter()
        .filter(|token| !linked.contains(token))
        .collect();

    let unsummon = put(&mut game, player, "Unsummon", Zone::Hand);
    cast(&mut game, unsummon, Some(Target::Card(new_source.card)));
    pass_pair(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert_eq!(treasures(&game), unlinked);
    assert!(game.observe(player).delayed_target_events.is_none());
}
