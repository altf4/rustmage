//! Mandatory, resolution and nonmana-X blight use real announcements and payment.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;
const FILTER: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 1,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const BODY: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::BlightCost(1),
                filter: FILTER,
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 7)],
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const PAID: CardRules = CardRules {
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            ..BODY.activated[0].cost
        },
        ..BODY.activated[0]
    }],
    ..BODY
};
const X: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::DamagePlayersAndTheirCreatures {
            players: PlayerSet::Opponent,
            amount: AmountValue::ChosenX,
        }],
    ),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::BlightXCost,
            filter: FILTER,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const OPTIONAL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::BlightCost(2),
                players: PlayerSet::Controller,
                filter: FILTER,
                amount: 1,
                optional_skip: Some(1),
            },
            Instruction::GainLife(PlayerSet::Controller, 7),
        ],
    ),
    ..CardRules::EMPTY
};
const FORCED: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::Blight(2),
                players: PlayerSet::Opponent,
                filter: FILTER,
                amount: 1,
                optional_skip: None,
            },
            Instruction::GainLife(PlayerSet::Controller, 7),
        ],
    ),
    ..CardRules::EMPTY
};
const BARRIER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CounterProhibition(1)],
    }],
    ..CardRules::EMPTY
};
fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn ready(g: &mut Game, condition: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if condition(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::ChooseReplacement { .. }
            )
        });
    }
    panic!("bounded blight scenario");
}
fn drain(g: &mut Game) {
    ready(g, |g| {
        g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending == Pending::Priority
    });
}
fn finish_costs(g: &mut Game, witness: CardId) {
    for _ in 0..24 {
        if g.state.announcing.is_none() {
            return;
        }
        match g.state.pending {
            Pending::PayCosts { .. } => pay(g),
            Pending::CastCosts { .. } => choose(g, |a| matches!(a, Action::ChooseCastCost { .. })),
            Pending::AdditionalCost { .. } => {
                let object = g.object_ref(witness);
                act(g, Action::ChooseAdditionalCost { object });
            }
            Pending::Replacement { .. } => {
                choose(g, |a| matches!(a, Action::ChooseReplacement { .. }))
            }
            _ => panic!("unexpected blight payment {:?}", g.state.pending),
        }
    }
    panic!("bounded blight costs");
}
#[test]
fn blight_activation_is_mandatory_nontarget_and_not_an_effect() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &BODY)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        act(&mut g, activate(source));
        finish_costs(&mut g, source);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 1);
        assert!(g.object_view(source).tapped);
        assert_eq!(g.state.players[p.index()].life, 20);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].life, 27);
    }
}
#[test]
fn blight_source_can_fund_mana_then_die_with_the_ability_on_stack() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &PAID)]);
    let mut g = scenario(Player::P0);
    g.state.players[0].mana = [0; 6];
    let source = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let elf = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    act(&mut g, activate(source));
    if matches!(g.state.pending, Pending::AdditionalCost { .. }) {
        let object = g.object_ref(elf);
        act(&mut g, Action::ChooseAdditionalCost { object });
    }
    choose(
        &mut g,
        |a| matches!(a, Action::TapMana { card, .. } if *card == elf),
    );
    finish_costs(&mut g, elf);
    assert_eq!(g.object_view(elf).zone, Zone::Graveyard);
    assert_eq!(g.state.stack.len(), 1);
    drain(&mut g);
    assert_eq!(g.state.players[0].life, 27);
}
#[test]
fn blight_resolution_decline_skips_bonus_and_effect_counters_are_doubled() {
    for (rules, forced) in [(&OPTIONAL, false), (&FORCED, true)] {
        let _scope = fixture_rules::activate(&[(card("Opt"), rules), (card("Bear Cub"), &BODY)]);
        for paid in [false, true] {
            let mut g = scenario(Player::P0);
            let owner = if forced { Player::P1 } else { Player::P0 };
            let recipient = put(&mut g, owner, "Bear Cub", Zone::Battlefield);
            g.state.objects[recipient as usize].growth.toughness = 5;
            put(&mut g, owner, "Doubling Season", Zone::Battlefield);
            let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
            cast(&mut g, spell, None);
            ready(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectChoice { .. })
            });
            let object = g.object_ref(recipient);
            if forced || paid {
                act(&mut g, Action::Blight { object });
            } else {
                act(&mut g, Action::DeclineEffectPayment);
            }
            drain(&mut g);
            assert_eq!(
                g.counter_count(recipient, CounterKind::MinusOne),
                if forced {
                    4
                } else if paid {
                    2
                } else {
                    0
                }
            );
            assert_eq!(
                g.state.players[0].life,
                if forced || paid { 27 } else { 20 }
            );
        }
    }
}
#[test]
fn blight_forced_effect_can_choose_prohibited_creature_and_empty_board_finishes() {
    let _scope = fixture_rules::activate(&[
        (card("Opt"), &FORCED),
        (card("Bear Cub"), &BODY),
        (card("Short Bow"), &BARRIER),
    ]);
    for creature in [false, true] {
        let mut g = scenario(Player::P0);
        put(&mut g, Player::P1, "Short Bow", Zone::Battlefield);
        let recipient = creature.then(|| put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield));
        let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
        cast(&mut g, spell, None);
        if let Some(recipient) = recipient {
            ready(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectChoice { .. })
            });
            let object = g.object_ref(recipient);
            act(&mut g, Action::Blight { object });
            assert_eq!(g.counter_count(recipient, CounterKind::MinusOne), 0);
        }
        drain(&mut g);
        assert_eq!(g.state.players[0].life, 27);
    }
}
#[test]
fn blight_positive_cost_is_unavailable_under_prohibition_but_zero_x_is_payable() {
    let _scope = fixture_rules::activate(&[
        (card("Opt"), &X),
        (card("Bear Cub"), &BODY),
        (card("Short Bow"), &BARRIER),
    ]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
    let source = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    reject(&mut g, activate(source));
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: spell });
    assert_eq!(
        g.x_choice().unwrap().intervals,
        vec![XInterval {
            minimum: 0,
            maximum: 0
        }]
    );
    reject(&mut g, Action::ChooseX { value: 1 });
    act(&mut g, Action::ChooseX { value: 0 });
    finish_costs(&mut g, source);
    drain(&mut g);
    assert_eq!(g.counter_count(source, CounterKind::MinusOne), 0);
}
#[test]
fn blight_x_bound_uses_greatest_toughness_and_may_pay_with_a_smaller_creature() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &X), (card("Bear Cub"), &BODY)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let large = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let small = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let foe = put(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let spell = put(&mut g, p, "Opt", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        assert_eq!(
            g.x_choice().unwrap().intervals,
            vec![XInterval {
                minimum: 0,
                maximum: 4
            }]
        );
        reject(&mut g, Action::ChooseX { value: 5 });
        act(&mut g, Action::ChooseX { value: 4 });
        finish_costs(&mut g, small);
        assert_eq!(g.object_view(small).zone, Zone::Graveyard);
        assert_eq!(g.state.stack[0].x, Some(4));
        assert_eq!(g.state.stack[0].mana_value(), 1);
        drain(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        assert_eq!(g.object_view(foe).damage, 4);
        assert_eq!(g.object_view(large).damage, 0);
    }
}
#[test]
fn blight_x_full_u32_is_an_owned_scalar_and_not_a_mana_component() {
    const LIFE_X: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::GainLifeScaled(
                PlayerSet::Controller,
                AmountValue::ChosenX,
            )],
        ),
        ..X
    };
    let _scope = fixture_rules::activate(&[(card("Opt"), &LIFE_X), (card("Bear Cub"), &BODY)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.state.objects[source as usize].growth.toughness = i128::from(u32::MAX);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: spell });
    assert_eq!(g.x_choice().unwrap().intervals[0].maximum, u32::MAX);
    act(&mut g, Action::ChooseX { value: u32::MAX });
    finish_costs(&mut g, source);
    assert_eq!(
        g.counter_count(source, CounterKind::MinusOne),
        u64::from(u32::MAX)
    );
    assert_eq!(g.state.stack[0].mana_value(), 1);
    drain(&mut g);
    assert_eq!(g.state.players[0].life, 20 + i128::from(u32::MAX));
}

#[test]
fn blight_additional_payment_is_owned_and_transactional_after_parent_disposal() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &X), (card("Bear Cub"), &BODY)]);
    let mut parent = scenario(Player::P0);
    let source = put(&mut parent, Player::P0, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut parent, Player::P0, "Opt", Zone::Hand);
    act(&mut parent, Action::AnnounceCast { card: spell });
    act(&mut parent, Action::ChooseX { value: 2 });
    for _ in 0..8 {
        if matches!(parent.state.pending, Pending::AdditionalCost { .. }) {
            break;
        }
        choose(&mut parent, |a| {
            matches!(a, Action::PayCosts { .. } | Action::ChooseCastCost { .. })
        });
    }
    assert!(matches!(
        parent.state.pending,
        Pending::AdditionalCost { .. }
    ));
    let action = Action::ChooseAdditionalCost {
        object: parent.object_ref(source),
    };
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| a == &action)
        .unwrap();
    let revision = parent.revision();
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut restored = parent.clone();
    drop(parent);
    assert_eq!(
        raw.apply(Player::P1, action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(raw.snapshot(), saved);
    raw.apply(Player::P0, action.clone()).unwrap();
    indexed.apply_index(Player::P0, revision, index).unwrap();
    finish_costs(&mut raw, source);
    finish_costs(&mut indexed, source);
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.counter_count(source, CounterKind::MinusOne), 2);
    assert_eq!(raw.state.players[1].life, 18);
    assert_eq!(restored.counter_count(source, CounterKind::MinusOne), 0);
    restored.restore(&saved).unwrap();
    act(&mut restored, action);
    finish_costs(&mut restored, source);
    drain(&mut restored);
    assert_eq!(raw.snapshot(), restored.snapshot());
}
