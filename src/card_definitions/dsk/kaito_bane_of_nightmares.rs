// Pinned Oracle/XMage provenance is recorded in the paired JSON. Remains gated
// pending validation of ninjutsu and the conditional layered creature form.
const BASE: BaseStatSetter = BaseStatSetter {
    power: Some(3),
    toughness: Some(4),
    own_turn: true,
    condition: Some(StatValue::SourceCounters(CounterKind::Loyalty)),
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Type,
    fragments: &[
        StaticAbilityFragment::SourceCounterTurnCreature {
            kind: CounterKind::Loyalty,
            minimum: 1,
            subtypes: &["Ninja"],
            keywords: Keywords::HEXPROOF,
        },
        StaticAbilityFragment::BaseStatSetter(0, BASE),
    ],
}];
const fn loyalty(amount: i32) -> ActivationCost {
    ActivationCost {
        loyalty: Some(amount),
        ..ActivationCost::ZERO
    }
}
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GROUPS,
    extras: &CardExtras {
        starting_loyalty: 4,
        base_stat_setter: Some(BASE),
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(1),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateEmblem(Card(5476))],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Hand,
            cycling: false,
            cost: ActivationCost {
                colored: [0, 1, 1, 0, 0, 0],
                generic: 1,
                mana_component: true,
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::ReturnUnblockedAttacker,
                    filter: SacrificeFilter {
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 1,
                        nonland: false,
                        other: false,
                        tokens: None,
                        artifact_enchantment_or_token: false,
                        subtype: None,
                    },
                }),
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutHandSourceAttacking],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(0),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Surveil(2),
                Instruction::If {
                    guard: StepGuard::OpponentLostLife,
                    skip: 1,
                },
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(-2),
            targets: one(TargetRule::Creature),
            program: &[
                Instruction::Tap,
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::Stun,
                    amount: 2,
                },
            ],
            sorcery: true,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
