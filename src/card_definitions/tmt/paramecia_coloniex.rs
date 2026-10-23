// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::Mill(
                PlayerSet::Controller,
                MillAmount::Fixed(3),
            )],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayExileDepartedSource { skip: 1 },
                Instruction::ExileDepartedSourceReflexive { ability: 2 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::TopLibrary],
        },
    ],
    ..CardRules::EMPTY
};
