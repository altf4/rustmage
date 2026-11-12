//! Immediate mana production followed by an owned source counter placement.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const BLACK: Mana = [0, 0, 1, 0, 0, 0];
const GREEN: Mana = [0, 0, 0, 0, 1, 0];
const COLORLESS: Mana = [0, 0, 0, 0, 0, 1];
const PLAYER: Player = Player::P0;

const ADD_NEST: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[
        Instruction::AddMana(ManaProduction::any_color(1)),
        Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::Nest,
            amount: 1,
        },
    ],
    sorcery: false,
    once_per_turn: false,
};

const MANA: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ADD_NEST],
    ..CardRules::EMPTY
};
const GUARDED: CardRules = CardRules {
    activated: &[
        ADD_NEST,
        ActivatedAbility {
            guard: ActivationGuard::SourceCounterExactly {
                kind: CounterKind::Nest,
                amount: 2,
            },
            cost: ActivationCost::ZERO,
            program: &[Instruction::AddMana(ManaProduction::Fixed(COLORLESS))],
            once_per_turn: true,
            ..ADD_NEST
        },
    ],
    ..MANA
};

const EXTRA_NEST: ReplacementSpec = ReplacementSpec::Counters {
    filter: ReplacementFilter::Permanent,
    kind: Some(CounterKind::Nest),
    factor: 1,
    add: 1,
    effects_only: true,
};
const EXTRA_NEST_PROVIDER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, EXTRA_NEST)],
    }],
    extras: &CardExtras {
        replacement_effects: &[EXTRA_NEST],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const PROHIBIT_CREATURE_COUNTERS: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CounterProhibition(1)],
    }],
    ..CardRules::EMPTY
};
const MAKE_TOKENS: CardRules = CardRules {
    activated: &[
        ADD_NEST,
        ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::CreateTokensScaled {
                token: Card::DrakeToken,
                amount: AmountValue::SourceTotalCounters,
                tapped: false,
            }],
            sorcery: true,
            ..ADD_NEST
        },
    ],
    ..MANA
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn mana(card: CardId, ability: u16, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability,
        counter_payment: Counters::default(),
        payment: [0; 6],
        production,
    }
}

fn replacement(game: &Game, provider: CardId) -> Action {
    Action::ChooseReplacement {
        source: game.object_ref(provider),
        ability: 0,
    }
}

fn apply_raw_and_indexed(game: &mut Game, action: Action) -> Game {
    let snapshot = game.snapshot();
    let mut indexed = game.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let actor = indexed.actor().unwrap();
    act(game, action);
    indexed
        .apply_index(actor, indexed.revision(), index)
        .unwrap();
    assert_eq!(indexed.snapshot(), game.snapshot());
    indexed.restore(&snapshot).unwrap();
    indexed
        .apply_index(actor, indexed.revision(), index)
        .unwrap();
    assert_eq!(indexed.snapshot(), game.snapshot());
    indexed
}

#[test]
fn projected_funding_can_use_a_producer_enabled_by_the_replaced_mana_counter() {
    let _rules = fixture_rules::activate(&[
        (named("Gingerbrute"), &GUARDED),
        (named("Hill Giant"), &PROHIBIT_CREATURE_COUNTERS),
    ]);
    let mut game = scenario(PLAYER);
    game.state.players[PLAYER.index()].mana = [0; 6];
    let source = put(&mut game, PLAYER, "Gingerbrute", Zone::Battlefield);
    let spell = put(&mut game, PLAYER, "Grizzly Bears", Zone::Hand);
    let announce = Action::AnnounceCast { card: spell };
    reject(&mut game, announce.clone());

    put(&mut game, PLAYER, "Doubling Season", Zone::Battlefield);
    let mut prohibited = game.clone();
    put(
        &mut prohibited,
        PLAYER.other(),
        "Hill Giant",
        Zone::Battlefield,
    );
    reject(&mut prohibited, announce.clone());

    act(&mut game, announce);
    let mut indexed = apply_raw_and_indexed(&mut game, mana(source, 0, GREEN));
    for branch in [&mut game, &mut indexed] {
        assert!(matches!(branch.state.pending, Pending::PayCosts { .. }));
        assert_eq!(branch.counter_count(source, CounterKind::Nest), 2);
        assert!(branch.object_view(source).tapped);
        assert!(branch.observe(PLAYER).stack.is_empty());
        act(branch, mana(source, 1, COLORLESS));
        pay(branch);
        settle(branch);
        assert_eq!(branch.object_view(spell).zone, Zone::Battlefield);
    }
    assert_eq!(game.snapshot(), indexed.snapshot());
}

#[test]
fn prohibited_placement_preserves_mana_and_later_sacrifice_counts_all_counter_kinds() {
    let _rules = fixture_rules::activate(&[
        (named("Gingerbrute"), &MAKE_TOKENS),
        (named("Hill Giant"), &PROHIBIT_CREATURE_COUNTERS),
    ]);
    let mut game = scenario(PLAYER);
    game.state.players[PLAYER.index()].mana = [0; 6];
    let source = put(&mut game, PLAYER, "Gingerbrute", Zone::Battlefield);
    let prohibition = put(&mut game, PLAYER.other(), "Hill Giant", Zone::Battlefield);
    act(&mut game, mana(source, 0, BLUE));
    assert_eq!(game.state.players[PLAYER.index()].mana, BLUE);
    assert_eq!(game.counter_count(source, CounterKind::Nest), 0);
    assert!(game.observe(PLAYER).stack.is_empty());

    game.move_object(prohibition, Zone::Graveyard);
    game.untap_objects(&[source], untap_rules::UntapCause::Effect);
    act(&mut game, mana(source, 0, GREEN));
    assert_eq!(game.counter_count(source, CounterKind::Nest), 1);
    game.untap_objects(&[source], untap_rules::UntapCause::Effect);
    game.add_counter(source, CounterKind::PlusOne, 2);
    game.add_counter(source, CounterKind::Charge, 1);
    let sacrifice = Action::Activate {
        card: source,
        ability: 1,
        target: None,
        payment: [0; 6],
        counter_payment: Counters::default(),
    };
    act(&mut game, sacrifice);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(
        game.observe(PLAYER)
            .objects
            .iter()
            .filter(|object| object.zone == Zone::Battlefield && object.card == Card::DrakeToken)
            .count(),
        4
    );
}

#[test]
fn failed_cast_rollback_retains_or_reverses_the_actual_replaced_counter_placement() {
    let _rules = fixture_rules::activate(&[
        (named("Gingerbrute"), &MANA),
        (named("Fountainport Bell"), &EXTRA_NEST_PROVIDER),
    ]);
    let mut game = scenario(PLAYER);
    let silver = put(
        &mut game,
        PLAYER,
        "Silverquill, the Disputant",
        Zone::Battlefield,
    );
    game.state.objects[silver as usize].growth.power = -1;
    let enemy = put(&mut game, PLAYER.other(), "Bear Cub", Zone::Battlefield);
    let aura = put(
        &mut game,
        PLAYER.other(),
        "Witness Protection",
        Zone::Battlefield,
    );
    game.state.objects[aura as usize].attached_to = Some(game.object_ref(enemy));
    let prison = put(&mut game, PLAYER, "Banishing Light", Zone::Hand);
    cast(&mut game, prison, None);
    for _ in 0..32 {
        if game.state.pending != Pending::Priority {
            break;
        }
        act(&mut game, Action::Pass);
    }
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(aura),
        },
    );
    for _ in 0..32 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.resolving.is_none()
        {
            break;
        }
        choose(&mut game, |action| {
            matches!(
                action,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishStackTargets
            )
        });
    }
    let source = put(&mut game, PLAYER, "Gingerbrute", Zone::Battlefield);
    let doubler = put(&mut game, PLAYER, "Doubling Season", Zone::Battlefield);
    let adder = put(&mut game, PLAYER, "Fountainport Bell", Zone::Battlefield);
    let spell = put(&mut game, PLAYER, "Candy Grapple", Zone::Hand);
    game.state.players[PLAYER.index()].mana = BLACK;
    act(&mut game, Action::AnnounceCast { card: spell });
    let sacrifice = Some(game.object_ref(prison));
    act(&mut game, Action::ChooseBargain { sacrifice });
    act(&mut game, Action::ChooseCasualty { pay: true });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    act(&mut game, mana(source, 0, BLUE));
    assert!(matches!(game.state.pending, Pending::Replacement { .. }));
    assert_eq!(game.counter_count(source, CounterKind::Nest), 0);
    assert!(game.observe(PLAYER).stack.is_empty());
    reject(&mut game, Action::Pass);
    let mut doubled_first = game.clone();
    let choose_double = Action::ChooseReplacement {
        source: doubled_first.object_ref(doubler),
        ability: 1,
    };
    act(&mut doubled_first, choose_double);
    assert_eq!(doubled_first.counter_count(source, CounterKind::Nest), 3);

    let choose_adder = replacement(&game, adder);
    apply_raw_and_indexed(&mut game, choose_adder);
    assert_eq!(game.counter_count(source, CounterKind::Nest), 4);
    pay(&mut game);
    let clause = game
        .observe(PLAYER)
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == CastCostClause::Additional)
        .unwrap() as u16;
    act(&mut game, Action::ChooseCastCost { clause });
    let object = game.object_ref(silver);
    act(&mut game, Action::ChooseAuraAttachment { object });
    assert!(matches!(game.state.pending, Pending::ManaRollback { .. }));
    assert!(game.observe(PLAYER.other()).mana_rollback.is_none());

    for keep in [false, true] {
        let mut branch = game.clone();
        let indexed = apply_raw_and_indexed(&mut branch, Action::ChooseManaRollback { keep });
        for result in [&branch, &indexed] {
            assert_eq!(result.object_view(spell).zone, Zone::Hand);
            assert_eq!(result.object_view(source).tapped, keep);
            assert_eq!(
                result.counter_count(source, CounterKind::Nest),
                if keep { 4 } else { 0 }
            );
            assert_eq!(
                result.state.players[PLAYER.index()].mana,
                [0, u32::from(keep), 1, 0, 0, 0]
            );
        }
    }
}
