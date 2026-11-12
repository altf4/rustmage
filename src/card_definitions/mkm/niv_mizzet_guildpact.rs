const HEXPROOF: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::QualifiedHexproof(
        SourceCharacteristicFilter::Multicolored,
    )],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::FLYING,
    static_groups: HEXPROOF,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Any),
            TargetSpec::one(TargetRule::Player {
                opponent_only: false,
            }),
        ]),
        program: &[
            Instruction::RememberCount(AmountValue::Stat(StatValue::ControlledTwoColorPairs)),
            Instruction::DamageScaled(AmountValue::MovedCount),
            Instruction::SelectGroup(1),
            Instruction::DrawScaled(PlayerSet::Targeted, AmountValue::MovedCount),
            Instruction::GainLifeScaled(PlayerSet::Controller, AmountValue::MovedCount),
        ],
    }],
    extras: &CardExtras {
        static_ability_groups: HEXPROOF,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
