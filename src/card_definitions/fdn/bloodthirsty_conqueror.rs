// Directly authored from the paired complete Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::FLYING.union(Keywords::DEATHTOUCH),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::LifeLost {
            relation: CastRelation::Opponent,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLifeScaled(
            PlayerSet::Controller,
            AmountValue::LifeEventAmount,
        )],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::FLYING.union(Keywords::DEATHTOUCH),
        )],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::FLYING.union(Keywords::DEATHTOUCH),
            )],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
