// Direct Oracle and pinned XMage behavior; gated pending integration validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::MENACE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::MENACE)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreatureAttacksPlayerAlone,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Discard(PlayerSet::Controller, 1),
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::PutCountersScaled {
                subject: Subject::Entering,
                kind: CounterKind::PlusOne,
                amount: AmountValue::CardsDiscardedThisTurn,
            },
        ],
    }],
    ..CardRules::EMPTY
};
