// Direct typed Oracle behavior; official CR702.192 supplies the pinned XMage paradigm TODO.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ExileUntilManaValueCastFree(4),
            Instruction::Paradigm { ability: 0 },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::CastParadigmCopy],
    }],
    ..CardRules::EMPTY
};
