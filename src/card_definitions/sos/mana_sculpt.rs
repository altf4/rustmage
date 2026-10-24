// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Spell),
        &[Instruction::CounterAndDelayManaIfSubtype {
            subtype: "Wizard",
            ability: 0,
        }],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::AddColorlessScaled(AmountValue::ManaSpent)],
    }],
    ..CardRules::EMPTY
};
