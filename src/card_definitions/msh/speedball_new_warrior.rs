// Direct typed behavior from the full frozen Oracle and pinned Java.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::Any,
            filter: CastFilter::Targeted {
                filter: &CastFilter::Any,
                target: CastTarget::Source,
            },
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Grow {
                subject: Subject::Source,
                power: 2,
                toughness: 2,
            },
            Instruction::ChangeCastSpellTargets,
        ],
    }],
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 2,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
