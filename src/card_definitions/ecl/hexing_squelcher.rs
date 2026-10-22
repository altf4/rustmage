// Direct typed behavior from complete pinned Oracle and XMage sources.
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ControlledSpellsUncounterable(
            u16::MAX,
        )],
    },
    StaticAbilityGroup {
        line: 3,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledCreatureTrigger {
            ability: 1,
            other: true,
            nontoken: false,
        }],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterRelatedUnlessPayLife(2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::Ward),
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterRelatedUnlessPayLife(2)],
        },
    ],
    static_groups: GROUPS,
    extras: &CardExtras {
        uncounterable: true,
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
