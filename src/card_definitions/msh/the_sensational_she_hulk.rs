// Complete nonphysical back face; damage amounts and controller are frozen at trigger time.
pub(in crate::cards) const RULES: CardRules = CardRules {
    deck_limit: None,
    effect: Effect::Creature(6, 6),
    keywords: Keywords::REACH.union(Keywords::TRAMPLE),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::StaticCastRestriction(0, 1)],
    }],
    extras: &CardExtras {
        static_cast_restriction: 1,
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDamaged {
            relation: CastRelation::You,
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        program: &[
            Instruction::MayOncePerTurn { skip: 2 },
            Instruction::RecordOncePerTurn,
            Instruction::DamageScaled(AmountValue::DamageReceivedAmount),
        ],
    }],
    ..CardRules::EMPTY
};
