// Complete direct typed behavior from full pinned Oracle and XMage sources.

const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::FLYING.union(Keywords::TRAMPLE),
        )],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::StartYourEngines],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::FLYING.union(Keywords::TRAMPLE),
    static_groups: GROUPS,
    activated: &[],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::DrawScaled(
                PlayerSet::Controller,
                AmountValue::Stat(StatValue::ControllerSpeed),
            ),
            Instruction::LoseLifeScaled(
                PlayerSet::Controller,
                AmountValue::Stat(StatValue::ControllerSpeed),
            ),
        ],
    }],

    extras: &CardExtras {
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
