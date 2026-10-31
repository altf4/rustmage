// Complete Oracle and pinned XMage implementation.
const fn loyalty(amount: i32) -> ActivationCost {
    ActivationCost {
        loyalty: Some(amount),
        ..ActivationCost::ZERO
    }
}
const fn ability(
    amount: i32,
    targets: TargetPlan,
    program: &'static [Instruction],
) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: loyalty(amount),
        targets,
        program,
        sorcery: true,
        once_per_turn: false,
    }
}
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::AttachedCreaturePreventsSourceAttack],
    }],
    extras: &CardExtras {
        starting_loyalty: 4,
        equipment: Some(EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        }),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::YourTurn,
            event: &TriggerCondition::AttachedDealtCombatDamage,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCountersScaled {
            subject: Subject::Source,
            kind: CounterKind::Loyalty,
            amount: AmountValue::CreatureCombatDamageAmount,
        }],
    }],
    activated: &[
        ability(
            1,
            TargetPlan::Single(TargetSpec {
                rule: TargetRule::creature_you_control(),
                min: 0,
                max: 1,
            }),
            &[
                Instruction::AttachSource,
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
            ],
        ),
        ability(
            -5,
            TargetPlan::NONE,
            &[Instruction::Draw(PlayerSet::Controller, 2)],
        ),
        ability(
            -10,
            TargetPlan::NONE,
            &[Instruction::AddMana(ManaProduction::any_color(10))],
        ),
    ],
    ..CardRules::EMPTY
};
