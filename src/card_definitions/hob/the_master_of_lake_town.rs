// Directly authored from the complete Oracle body and paired pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::DEATHTOUCH,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::LifeLost {
                relation: CastRelation::Any,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::MillLifeEvent],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: &[Instruction::DrawForGraveyardsAtLeast(7)],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::DEATHTOUCH,
        )],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::DEATHTOUCH,
            )],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
