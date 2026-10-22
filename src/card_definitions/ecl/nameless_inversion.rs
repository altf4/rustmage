// Direct typed behavior from the pinned Oracle and complete XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(one(TargetRule::Creature), &[
        Instruction::Grow { subject: Subject::Targets, power: 3, toughness: -3 },
        Instruction::SetCreatureTypesUntilEndTurn { subject: Subject::Targets, types: &[] },
    ]),
    keywords: Keywords::CHANGELING,
    ..CardRules::EMPTY
};
