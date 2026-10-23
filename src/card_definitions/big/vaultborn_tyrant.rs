const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::TRAMPLE)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::TRAMPLE,
    static_groups: GROUPS,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::GainLife(PlayerSet::Controller, 3),
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::OtherFilteredEntersYou(
                EntryPredicate::CreaturePowerAtLeast(4),
            ),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::GainLife(PlayerSet::Controller, 3),
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfNontokenDies,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateCopyTokenWithExceptions {
                subject: Subject::Source,
                amount: 1,
                exceptions: CopyExceptions::ARTIFACT,
            }],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
