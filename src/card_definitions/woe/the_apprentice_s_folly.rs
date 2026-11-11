// Direct typed behavior from the pinned Oracle and XMage sources.
const ORIGINAL: TargetPlan = one(TargetRule::Filtered {
    base: &TargetRule::Filtered {
        base: &TargetRule::creature_you_control(),
        predicate: &TargetPredicate::Nontoken,
    },
    predicate: &TargetPredicate::NoOtherControlledSameName { tokens_only: true },
});
const REFLECTION: &[Instruction] = &[Instruction::CreateReflectionCopy(Subject::Targets)];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: ORIGINAL,
            program: REFLECTION,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: ORIGINAL,
            program: REFLECTION,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: &[Instruction::SacrificeControlledSubtype("Reflection")],
        },
    ],
    ..CardRules::EMPTY
};
