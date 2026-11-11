pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::FLYING.union(Keywords::LIFELINK),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayFromOutsideOwnHand(&TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Any,
            ordinal: None,
        }),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::CastHandPermanentFreeUpTo(AmountValue::CastSpellManaValue),
            Instruction::IfOfferedCastCommitted { skip: 1 },
            Instruction::PutHandLand { tapped: false },
        ],
    }],
    ..CardRules::EMPTY
};
