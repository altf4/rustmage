// Direct typed behavior from complete Oracle, XMage and successful-resolution rules.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::CastFromHand(&TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::InstantSorcery,
                ordinal: None,
            }),
            targets: TargetPlan::NONE,
            program: &[Instruction::MarkCastDreamExile],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::CastOwnedDreamCard],
        },
    ],
    ..CardRules::EMPTY
};
