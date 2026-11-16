// Oracle and official Star Trek release notes; no pinned XMage class exists.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::SpellKindsControlled {
            kinds: 0,
            excluded: 1,
            relation: 1,
        }),
        &[
            Instruction::MayTargetSpellControllerExile,
            Instruction::CastFilteredHandFree(CastFilter::Noncreature),
        ],
    ),
    ..CardRules::EMPTY
};
