pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Outlaw,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyCapturedSpell {
            count: SpellCopyCount::One,
            retarget: false,
        }],
    }],
    ..CardRules::EMPTY
};
