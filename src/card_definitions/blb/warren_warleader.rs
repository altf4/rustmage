const ATTACK_MODES: &[SpellMode] = &[
    SpellMode {
        text: "Create a tapped and attacking Rabbit",
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokensAttacking {
            token: Card(5285),
            amount: 1,
        }],
    },
    SpellMode {
        text: "Attacking creatures you control get +1/+1 until end of turn",
        targets: TargetPlan::NONE,
        program: &[Instruction::GrowCreatures {
            filter: CreatureFilter {
                relation: 0,
                other: false,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: true,
                tokens: None,
                artifact: false,
            },
            power: 1,
            toughness: 1,
        }],
    },
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfOffspring,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateCopyTokenWithExceptions {
                subject: Subject::Source,
                amount: 1,
                exceptions: CopyExceptions {
                    base: Some((1, 1)),
                    ..CopyExceptions::NONE
                },
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::AttackDeclared {
                minimum: 1,
                power_min: None,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Modal(ATTACK_MODES)],
        },
    ],
    extras: &CardExtras {
        offspring_cost: Some(ManaCost {
            colored: [0; 6],
            generic: 2,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
