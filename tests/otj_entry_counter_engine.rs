//! Guarded entry counters and simultaneous doubling of one counter kind.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const CONTROLLED_LANDS: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 2,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
};
const GUARDED_ENTRY: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    extras: &CardExtras {
        entry_counter_rules: &[EntryCounterRule {
            guard: StepGuard::Expression {
                value: StatValue::UntappedCount(CONTROLLED_LANDS),
                minimum: Some(5),
                maximum: None,
            },
            amount: EntryCounterAmount::FixedCounters(&[
                (CounterKind::PlusOne, 2),
                (CounterKind::Lifelink, 1),
            ]),
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn lands(game: &mut Game, player: Player, untapped: usize) {
    for index in 0..5 {
        let id = put(game, player, "Forest", Zone::Battlefield);
        game.state.objects[id as usize].tapped = index >= untapped;
    }
}

fn assert_entry(game: &Game, card: CardId, qualifies: bool) {
    assert_eq!(
        game.counter_count(card, CounterKind::PlusOne),
        if qualifies { 2 } else { 0 }
    );
    assert_eq!(
        game.counter_count(card, CounterKind::Lifelink),
        u64::from(qualifies)
    );
    assert_eq!(game.keywords(card).contains(Keywords::LIFELINK), qualifies);
}

#[test]
fn untapped_lands_enable_both_counters_on_the_actual_spell_entry() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &GUARDED_ENTRY)]);
    for (untapped, qualifies) in [(4, false), (5, true)] {
        let player = Player::P0;
        let mut game = scenario(player);
        lands(&mut game, player, untapped);
        lands(&mut game, player.other(), 5);
        put(&mut game, player, "Savannah Lions", Zone::Battlefield);
        let entrant = put(&mut game, player, "Grizzly Bears", Zone::Hand);

        cast(&mut game, entrant, None);
        settle(&mut game);
        assert_eq!(game.object_view(entrant).zone, Zone::Battlefield);
        assert_entry(&game, entrant, qualifies);
    }
}

#[test]
fn projected_and_actual_returns_use_the_incoming_controller_not_owner() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &GUARDED_ENTRY)]);
    let player = Player::P1;
    for (untapped, owner_untapped, qualifies) in [(4, 5, false), (5, 4, true)] {
        let mut game = scenario(player);
        lands(&mut game, player, untapped);
        lands(&mut game, player.other(), owner_untapped);
        let entrant = put(&mut game, player.other(), "Grizzly Bears", Zone::Graveyard);
        let mut movement = game.entry_move(entrant, Zone::Battlefield, false);
        movement.controller = player;
        let goal = FundingGoal {
            actor: player,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let projected = FundingState::from_game(&game, &goal);
        assert!(
            projected.visit_entry_moves(vec![movement.clone()], true, None, &mut |entered| {
                assert_eq!(
                    entered.counter_count(entrant, CounterKind::PlusOne),
                    if qualifies { 2 } else { 0 }
                );
                assert_eq!(
                    entered.counter_count(entrant, CounterKind::Lifelink),
                    u64::from(qualifies)
                );
                true
            })
        );

        game.stage_entries(vec![movement], None);
        assert_eq!(game.object_view(entrant).controller, player);
        assert_eq!(game.object_view(entrant).owner, player.other());
        assert_entry(&game, entrant, qualifies);
    }
}

const DOUBLE_ONE_KIND: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 2, 0],
            generic: 3,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ChangeControlledPermanentGroupCounters {
            filters: &[ControlledPermanentFilter {
                all_kinds: 1,
                ..ControlledPermanentFilter::ANY
            }],
            effect: PermanentGroupCounters::DoubleKind(CounterKind::PlusOne),
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const OBSERVE_BATCH: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CounterBatchPlacedByYou {
            kind: CounterKind::PlusOne,
            filter: CountFilter {
                kind: 1,
                ..CONTROLLED_LANDS
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn double_one_kind_uses_one_placement_batch_and_preserves_other_counters() {
    let observer = Card::from_name("The Wind Crystal").unwrap();
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &DOUBLE_ONE_KIND),
        (observer, &OBSERVE_BATCH),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let other = put(&mut game, player, "Savannah Lions", Zone::Battlefield);
    let zero = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let noncreature = spawn(&mut game, player, observer, Zone::Battlefield);
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    for id in [source, other, enemy, noncreature] {
        game.add_counter(id, CounterKind::PlusOne, 2);
        game.add_counter(id, CounterKind::Shield, 1);
    }
    let before = game.state.players[player.index()].life;

    act(
        &mut game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut game);
    settle(&mut game);

    assert_eq!(game.state.players[player.index()].life, before + 1);
    for id in [source, other] {
        assert_eq!(game.counter_count(id, CounterKind::PlusOne), 6);
        assert_eq!(game.counter_count(id, CounterKind::Shield), 1);
    }
    for id in [enemy, noncreature] {
        assert_eq!(game.counter_count(id, CounterKind::PlusOne), 2);
        assert_eq!(game.counter_count(id, CounterKind::Shield), 1);
    }
    assert_eq!(game.counter_count(zero, CounterKind::PlusOne), 0);
}
