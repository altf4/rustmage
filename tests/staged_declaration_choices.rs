//! Immediate declaration/material choices never prove an entire payment plan.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const TOP_PERMISSION: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CastFromLibraryTop(
            CastFilter::ManaValueMinimum {
                filter: &CastFilter::Any,
                minimum: 4,
            },
        )],
    }],
    ..CardRules::EMPTY
};
const CONSPIRE: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::NoncreatureConspire],
    }],
    ..CardRules::EMPTY
};

fn assert_raw_index_x_boundary(mut game: Game, maximum: u32, generic: u32) {
    let player = game.actor().unwrap();
    assert_eq!(
        game.x_choice().unwrap().intervals.last().unwrap().maximum,
        maximum
    );
    if let Some(value) = maximum.checked_add(1) {
        // Rejected representation values leave both rules and external revision
        // intact, including the owned active attempt checkpoint.
        reject(&mut game, Action::ChooseX { value });
    }
    let action = Action::ChooseX { value: maximum };
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, action);
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.revision(), indexed.revision());
    assert_eq!(
        game.state
            .announcing
            .as_ref()
            .unwrap()
            .total_cost
            .unwrap()
            .generic,
        generic
    );
}

#[test]
fn repeated_x_spell_stops_at_representation_capacity_without_payment_search() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        game.state.players[player.index()].mana = [0; 6];
        let card = put(&mut game, player, "Another Round", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card });
        reject(&mut game, Action::ChooseX { value: u32::MAX });
        let maximum = (u32::MAX - 2) / 2;
        assert_raw_index_x_boundary(game, maximum, 2 + 2 * maximum);
    }
}

const LATE_VALUE_REDUCTION: SpellReduction = SpellReduction {
    source_only: true,
    filter: CastFilter::ManaValueMinimum {
        filter: &CastFilter::Any,
        minimum: u32::MAX as u64 + 20,
    },
    amount: SpellReductionAmount::Fixed(100),
    turn: CostTurn::Any,
    guard: StepGuard::Always,
};
const DISCOUNTED_XX_SPELL: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[]),
    extras: &rustmage_core::cards::authored::CardExtras {
        x_symbols: 2,
        cost_reductions: &[LATE_VALUE_REDUCTION],
        ..rustmage_core::cards::authored::CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn x_capacity_keeps_a_later_mana_value_discount_interval() {
    let card = Card::from_name("Another Round").unwrap();
    let _rules = fixture_rules::activate(&[(card, &DISCOUNTED_XX_SPELL)]);
    let mut game = scenario(Player::P0);
    game.state.players[0].mana = [0; 6];
    let card = put(&mut game, Player::P0, "Another Round", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card });
    let minimum = (u64::from(u32::MAX) + 20 - 3).div_ceil(2) as u32;
    let maximum = ((u64::from(u32::MAX) + 100 - 2) / 2) as u32;
    assert_eq!(
        game.x_choice().unwrap().intervals,
        [
            XInterval {
                minimum: 0,
                maximum: (u32::MAX - 2) / 2
            },
            XInterval { minimum, maximum },
        ]
    );
    reject(&mut game, Action::ChooseX { value: minimum - 1 });
    assert_raw_index_x_boundary(game, maximum, (2 + 2 * u64::from(maximum) - 100) as u32);
}

const XX_ABILITY: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 3,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ActivationX,
            Instruction::ActivationX,
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const DISCOUNTED_XX_ABILITY: CardRules = CardRules {
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ControlledActivationReduction {
            kinds: 1,
            amount: ActivationReductionAmount::Fixed(6),
            leave_one_mana: true,
        }],
    }],
    ..XX_ABILITY
};

#[test]
fn activation_x_capacity_applies_current_reductions_before_narrowing() {
    for (rules, reduction) in [(&XX_ABILITY, 0u64), (&DISCOUNTED_XX_ABILITY, 6)] {
        let _rules = fixture_rules::activate(&[(Card::HillGiant, rules)]);
        let mut game = scenario(Player::P0);
        game.state.players[0].mana = [0; 6];
        let card = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
        act(
            &mut game,
            Action::AnnounceActivate {
                card,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(&mut game, Action::ChooseX { value: u32::MAX });
        let maximum = ((u64::from(u32::MAX) + reduction - 3) / 2) as u32;
        assert_raw_index_x_boundary(
            game,
            maximum,
            (3 + 2 * u64::from(maximum) - reduction) as u32,
        );
    }
}

fn clause(game: &mut Game, kind: CastCostClause) {
    let index = game
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| !row.paid && row.clause == kind)
        .unwrap();
    act(
        game,
        Action::ChooseCastCost {
            clause: index as u16,
        },
    );
}

#[test]
fn craft_admits_an_unfunded_attempt_and_keeps_incomplete_materials_provisional() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        game.state.players[player.index()].mana = [0; 6];
        let jewel = put(&mut game, player, "The Enigma Jewel", Zone::Battlefield);
        let material = put(&mut game, player, "Braided Net", Zone::Graveyard);
        let object = game.object_ref(material);
        let action = Action::AnnounceActivate {
            card: jewel,
            ability: 1,
            counter_payment: Counters::default(),
        };
        assert!(game.legal_actions().contains(&action));
        act(&mut game, action);
        act(&mut game, Action::BeginOrderedCosts);
        clause(&mut game, CastCostClause::Additional);
        assert!(game.current_material_cost_impossible());
        let before = game.snapshot();
        let revision = game.revision();
        let selected = Action::ChooseAdditionalCost { object };
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == selected)
            .unwrap();
        let mut raw = game.clone();
        act(&mut raw, selected);
        game.apply_index(player, revision, index).unwrap();
        assert_eq!(game.snapshot(), raw.snapshot());
        assert_eq!(game.object_ref(material), object);
        assert_eq!(game.object_view(material).zone, Zone::Graveyard);
        assert!(!game.legal_actions().contains(&Action::FinishCraftMaterials));
        game.restore(&before).unwrap();
        assert!(matches!(
            game.apply_index(player, revision, index),
            Err(Error::StaleRevision)
        ));
    }
}

#[test]
fn library_x_permissions_constrain_declarations_without_affordability() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &TOP_PERMISSION)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        game.state.players[player.index()].mana = [0; 6];
        put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let card = put(&mut game, player, "Exsanguinate", Zone::Library);
        let action = Action::AnnounceCast { card };
        assert!(game.legal_actions().contains(&action));
        act(&mut game, action);
        let domain = game.x_choice().unwrap();
        assert_eq!(
            domain.intervals,
            [XInterval {
                minimum: 2,
                maximum: u32::MAX
            }]
        );
        assert!(!domain.contains(1));
        assert!(domain.contains(1_000_000));
        reject(&mut game, Action::ChooseX { value: 1 });
        act(&mut game, Action::ChooseX { value: 1_000_000 });
        assert!(game.state.stack.is_empty());
        assert_eq!(
            game.state.announcing.as_ref().unwrap().item.x,
            Some(1_000_000)
        );
        assert!(game.observe(player.other()).x_choice.is_none());
    }
}

#[test]
fn conspire_selects_one_object_before_an_atomic_pair_payment() {
    let grant = Card::from_name("Short Bow").unwrap();
    let _rules = fixture_rules::activate(&[(grant, &CONSPIRE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Short Bow", Zone::Battlefield);
    let creatures: Vec<_> = (0..12)
        .map(|_| put(&mut game, player, "Fog Bank", Zone::Battlefield))
        .collect();
    let card = put(&mut game, player, "Divination", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card });
    act(&mut game, Action::ChooseConspire { pay: true });
    act(&mut game, Action::BeginOrderedCosts);
    clause(&mut game, CastCostClause::Conspire(0));
    let actions = game.legal_actions();
    assert_eq!(
        actions
            .iter()
            .filter(|action| matches!(action, Action::ChooseConspireObject { .. }))
            .count(),
        12
    );
    assert!(
        !actions
            .iter()
            .any(|action| matches!(action, Action::PayConspire { .. }))
    );
    let first = game.object_ref(creatures[0]);
    let second = game.object_ref(creatures[1]);
    act(&mut game, Action::ChooseConspireObject { object: first });
    assert!(creatures.iter().all(|&card| !game.object_view(card).tapped));
    reject(
        &mut game,
        Action::PayConspire {
            objects: [first, first],
        },
    );
    assert_eq!(
        game.legal_actions()
            .iter()
            .filter(|action| matches!(action, Action::PayConspire { .. }))
            .count(),
        11
    );
    act(
        &mut game,
        Action::PayConspire {
            objects: [first, second],
        },
    );
    assert!(game.object_view(first.card).tapped && game.object_view(second.card).tapped);
    assert!(
        creatures[2..]
            .iter()
            .all(|&card| !game.object_view(card).tapped)
    );
}

const ANY_CARD: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const LIFE_SPELL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Draw(PlayerSet::Controller, 1)],
    ),
    extras: &rustmage_core::cards::authored::CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::PayLife(2),
            filter: ANY_CARD,
        }),
        ..rustmage_core::cards::authored::CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const DISCARD_OR_LIFE: CardRules = CardRules {
    extras: &rustmage_core::cards::authored::CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::DiscardOrLife(2),
            filter: ANY_CARD,
        }),
        ..rustmage_core::cards::authored::CardExtras::EMPTY
    },
    ..LIFE_SPELL
};
const LIFE_ABILITY: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            life: 2,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

#[test]
fn fixed_life_costs_use_current_life_and_reject_changed_concrete_payments() {
    let spell_definition = Card::from_name("Divination").unwrap();
    let ability_definition = Card::from_name("Short Bow").unwrap();
    let _rules = fixture_rules::activate(&[
        (spell_definition, &LIFE_SPELL),
        (ability_definition, &LIFE_ABILITY),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Divination", Zone::Hand);
        let source = put(&mut game, player, "Short Bow", Zone::Battlefield);
        game.state.players[player.index()].life = 1;
        let announce_spell = Action::AnnounceCast { card: spell };
        let announce_ability = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        assert!(!game.legal_actions().contains(&announce_spell));
        assert!(!game.legal_actions().contains(&announce_ability));
        reject(&mut game, announce_spell.clone());
        reject(&mut game, announce_ability.clone());

        game.state.players[player.index()].life = 2;
        assert!(game.legal_actions().contains(&announce_ability));
        let concrete = Action::Activate {
            card: source,
            ability: 0,
            target: None,
            payment: [0; 6],
            counter_payment: Counters::default(),
        };
        assert!(game.legal_actions().contains(&concrete));
        game.state.players[player.index()].life = 1;
        reject(&mut game, concrete);

        game.state.players[player.index()].life = 2;
        act(&mut game, announce_spell);
        act(&mut game, Action::BeginOrderedCosts);
        clause(&mut game, CastCostClause::Additional);
        assert!(game.legal_actions().contains(&Action::ChooseAdditionalLife));
        game.state.players[player.index()].life = 1;
        reject(&mut game, Action::ChooseAdditionalLife);
        assert!(game.current_material_cost_impossible());
    }
}

#[test]
fn a_discard_or_life_cost_keeps_the_discard_route_at_one_life() {
    let spell_definition = Card::from_name("Divination").unwrap();
    let _rules = fixture_rules::activate(&[(spell_definition, &DISCARD_OR_LIFE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.players[player.index()].life = 1;
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    let discard = put(&mut game, player, "Forest", Zone::Hand);
    let object = game.object_ref(discard);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::BeginOrderedCosts);
    clause(&mut game, CastCostClause::Additional);
    assert!(!game.legal_actions().contains(&Action::ChooseAdditionalLife));
    assert!(
        game.legal_actions()
            .contains(&Action::ChooseAdditionalCost { object })
    );
    assert!(!game.current_material_cost_impossible());
    act(&mut game, Action::ChooseAdditionalCost { object });
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 1);
}
