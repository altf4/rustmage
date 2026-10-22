// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::ChooseCreatureType, Instruction::ReturnChosenCreatureTypeFromGraveyard]),
    extras: &CardExtras { has_convoke: true, ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
