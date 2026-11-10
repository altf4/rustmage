// Direct typed behavior from the pinned Oracle and XMage sources.
const DRAW: &[Instruction] = &[Instruction::Draw(PlayerSet::Controller, 1)];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: DRAW,
        },
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::ManaValueMinimum {
                    filter: &CastFilter::Any,
                    minimum: 5,
                },
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: DRAW,
        },
    ],
    ..CardRules::EMPTY
};
