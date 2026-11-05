// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Any,
            ordinal: Some(2),
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::CopyCapturedSpell {
                count: SpellCopyCount::One,
                retarget: false,
            },
            Instruction::SuspendCapturedSpell { time: 4 },
        ],
    }],
    ..CardRules::EMPTY
};
