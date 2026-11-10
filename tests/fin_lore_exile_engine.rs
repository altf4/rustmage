//! Exact lore placement receipts and normal play of a linked exile card.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SAGA_NAME: &str = "Summon: Anima";
const SAGA: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: &[Instruction::ExileTopPlayOnSourceLoreTurns],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: &[],
        },
    ],
    ..CardRules::EMPTY
};
const CREATURE: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Creature));
const LORE: CardRules = CardRules {
    effect: Effect::Spell(
        CREATURE,
        &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::Lore,
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};
const PLUS_ONE: CardRules = CardRules {
    effect: Effect::Spell(
        CREATURE,
        &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};

fn setup(player: Player, top: &str) -> (Game, CardId, CardId) {
    let mut game = scenario(player);
    let exiled = put(&mut game, player, top, Zone::Library);
    let saga = put(&mut game, player, SAGA_NAME, Zone::Hand);
    cast(&mut game, saga, None);
    settle(&mut game);
    assert_eq!(game.counter_count(saga, CounterKind::Lore), 1);
    assert_eq!(game.object_view(exiled).zone, Zone::Exile);
    (game, saga, exiled)
}

fn start_next_turn(game: &mut Game) {
    let previous = game.state.active;
    game.state.step = Step::End;
    game.give_priority();
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.state.active, previous.other());
    assert_eq!(game.state.step, Step::Upkeep);
}

fn cast_for(game: &mut Game, player: Player, name: &str, target: CardId) -> CardId {
    if game.actor() != Some(player) {
        act(game, Action::Pass);
    }
    assert_eq!(game.actor(), Some(player));
    let spell = put(game, player, name, Zone::Hand);
    cast(game, spell, Some(Target::Card(target)));
    settle(game);
    spell
}

#[test]
fn entry_lore_grants_normal_land_play_and_survives_lore_removal() {
    let _rules = fixture_rules::activate(&[(Card::from_name(SAGA_NAME).unwrap(), &SAGA)]);
    let player = Player::P0;
    let (mut parent, saga, exiled) = setup(player, "Forest");
    let hand_land = put(&mut parent, player, "Forest", Zone::Hand);
    let source = parent.object_ref(saga);
    let object = parent.object_ref(exiled);
    for viewer in [player, player.other()] {
        let observation = parent.observe(viewer);
        assert!(
            observation
                .counter_placement_history
                .iter()
                .any(|row| { row.object == source && row.placed_by == player && row.lore })
        );
        assert!(observation.exile_permissions.iter().any(|permission| {
            permission.object == object
                && permission.source == Some(source)
                && permission.player == player
                && permission.expiry == ExilePlayExpiry::SourceLoreTurns
        }));
    }
    parent.pay_counters(saga, &Counters::of(CounterKind::Lore, 1));
    assert_eq!(parent.counter_count(saga, CounterKind::Lore), 0);
    let land = Action::PlayLand { card: exiled };
    let saved = parent.snapshot();
    let revision = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|action| *action == land)
        .unwrap();

    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    act(&mut raw, land.clone());
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.object_view(exiled).zone, Zone::Battlefield);
    assert_ne!(raw.object_ref(exiled), object);
    assert!(raw.observe(player).exile_permissions.is_empty());
    reject(&mut raw, Action::PlayLand { card: hand_land });

    raw.restore(&saved).unwrap();
    raw.state.step = Step::BeginCombat;
    reject(&mut raw, land.clone());
    raw.restore(&saved).unwrap();
    act(&mut raw, Action::PlayLand { card: hand_land });
    reject(&mut raw, land);
    assert_eq!(raw.object_view(exiled).zone, Zone::Exile);
    assert_eq!(indexed.object_view(exiled).zone, Zone::Battlefield);
}

#[test]
fn only_this_players_positive_lore_on_this_exact_saga_unlocks_paid_casting() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name(SAGA_NAME).unwrap(), &SAGA),
        (Card::from_name("Opt").unwrap(), &LORE),
        (Card::from_name("Giant Growth").unwrap(), &PLUS_ONE),
    ]);
    let player = Player::P1;
    let opponent = player.other();
    let (mut game, saga, exiled) = setup(player, "Shock");
    let exact = game.object_ref(saga);
    let other = put(&mut game, player, SAGA_NAME, Zone::Battlefield);
    game.set_counters(other, Counters::of(CounterKind::Lore, 1));
    start_next_turn(&mut game);
    game.state.step = Step::Main1;
    for seat in [player, opponent] {
        game.state.players[seat.index()].mana = [8; 6];
    }
    assert!(!game.has_printed_cost_exile_permission(player, exiled));

    cast_for(&mut game, opponent, "Opt", saga);
    assert_eq!(game.counter_count(saga, CounterKind::Lore), 2);
    let history = game.observe(player).counter_placement_history;
    assert!(
        history
            .iter()
            .any(|row| row.object == exact && row.placed_by == opponent && row.lore)
    );
    assert!(!game.has_printed_cost_exile_permission(opponent, exiled));
    assert!(!game.has_printed_cost_exile_permission(player, exiled));

    cast_for(&mut game, player, "Opt", other);
    assert_eq!(game.counter_count(other, CounterKind::Lore), 2);
    assert!(!game.has_printed_cost_exile_permission(player, exiled));

    let aura = cast_for(&mut game, opponent, "Blossombind", saga);
    cast_for(&mut game, player, "Opt", saga);
    assert_eq!(game.counter_count(saga, CounterKind::Lore), 2);
    assert!(!game.has_printed_cost_exile_permission(player, exiled));
    assert!(
        !game
            .observe(player)
            .counter_placement_history
            .iter()
            .any(|row| { row.object == exact && row.placed_by == player && row.lore })
    );

    game.pay_counters(saga, &Counters::of(CounterKind::Lore, 1));
    assert!(!game.has_printed_cost_exile_permission(player, exiled));
    game.move_object(aura, Zone::Graveyard);
    cast_for(&mut game, player, "Giant Growth", saga);
    assert_eq!(game.counter_count(saga, CounterKind::PlusOne), 1);
    assert!(!game.has_printed_cost_exile_permission(player, exiled));

    cast_for(&mut game, player, "Opt", saga);
    assert_eq!(game.counter_count(saga, CounterKind::Lore), 2);
    assert!(game.has_printed_cost_exile_permission(player, exiled));
    assert_eq!(game.state.active, opponent);
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(player));
    game.state.players[player.index()].mana[3] = 0;
    let announce = Action::AnnounceExile { card: exiled };
    reject(&mut game, announce.clone());
    game.state.players[player.index()].mana[3] = 1;
    act(&mut game, announce);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana[3], 0);
    assert_eq!(game.state.players[opponent.index()].life, 18);
    assert_eq!(game.object_view(exiled).zone, Zone::Graveyard);
}

#[test]
fn blinking_the_saga_keeps_the_qualifying_turn_but_cannot_reopen_its_old_link() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name(SAGA_NAME).unwrap(), &SAGA),
        (Card::from_name("Opt").unwrap(), &LORE),
    ]);
    let player = Player::P0;
    let (mut game, saga, old_card) = setup(player, "Shock");
    let old_source = game.object_ref(saga);
    let new_card = put(&mut game, player, "Giant Growth", Zone::Library);
    game.move_object(saga, Zone::Exile);
    game.move_object(saga, Zone::Battlefield);
    let new_source = game.object_ref(saga);
    assert_ne!(new_source, old_source);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.object_view(new_card).zone, Zone::Exile);
    assert!(game.has_printed_cost_exile_permission(player, old_card));
    assert!(game.has_printed_cost_exile_permission(player, new_card));

    start_next_turn(&mut game);
    assert!(!game.has_printed_cost_exile_permission(player, old_card));
    assert!(!game.has_printed_cost_exile_permission(player, new_card));
    game.state.players[player.index()].mana = [8; 6];
    cast_for(&mut game, player, "Opt", saga);
    assert_eq!(game.counter_count(saga, CounterKind::Lore), 2);
    assert!(game.has_printed_cost_exile_permission(player, new_card));
    assert!(!game.has_printed_cost_exile_permission(player, old_card));
    let permissions = game.observe(player).exile_permissions;
    assert_eq!(permissions.len(), 1);
    assert_eq!(permissions[0].source, Some(new_source));
    assert_eq!(permissions[0].object, game.object_ref(new_card));
    assert_eq!(game.object_view(old_card).zone, Zone::Exile);

    start_next_turn(&mut game);
    assert!(!game.has_printed_cost_exile_permission(player, new_card));
    for _ in 0..4 {
        act(&mut game, Action::Pass);
    }
    assert_eq!(game.state.step, Step::Main1);
    settle(&mut game);
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);
    assert!(game.has_printed_cost_exile_permission(player, new_card));
    assert!(!game.has_printed_cost_exile_permission(player, old_card));
}
