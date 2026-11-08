// Protection is a removable layer-six grant lasting until the original controller's next turn.
const COLOR_PROTECTION: Keywords = Keywords::PROTECTION_WHITE
    .union(Keywords::PROTECTION_BLUE)
    .union(Keywords::PROTECTION_BLACK)
    .union(Keywords::PROTECTION_RED)
    .union(Keywords::PROTECTION_GREEN);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::CANT_BE_BLOCKED,
    deck_limit: None,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfTransforms,
            targets: one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[Instruction::GrantKeywordsUntilNextTurn {
                subject: Subject::Targets,
                keywords: COLOR_PROTECTION,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::Main1,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayMana {
                    cost: ManaCost {
                        colored: [0, 1, 0, 0, 0, 0],
                        generic: 0,
                    },
                    skip: 1,
                },
                Instruction::TransformSource,
            ],
        },
    ],
    ..CardRules::EMPTY
};
