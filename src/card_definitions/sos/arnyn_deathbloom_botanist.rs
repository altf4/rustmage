// Whole-card typed Oracle behavior; complete XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::DEATHTOUCH,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::DEATHTOUCH,
        )],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDiesWithMaximumStat {
            relation: CastRelation::You,
            maximum: 1,
        },
        targets: one(TargetRule::Player {
            opponent_only: true,
        }),
        program: &[
            Instruction::LoseLife(PlayerSet::Targeted, 2),
            Instruction::GainLife(PlayerSet::Controller, 2),
        ],
    }],
    ..CardRules::EMPTY
};
