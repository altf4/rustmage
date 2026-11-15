// Oracle and official TRK release notes: one permanent from this exact mill;
// the first-spell qualification is frozen when the spell is cast.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(4)),
            Instruction::SelectMilled(MilledSelectionSpec {
                filter: MilledFilter::Permanent,
                count: MilledCount::Range {
                    minimum: 0,
                    maximum: 1,
                },
                destination: MilledDestination::Hand,
            }),
            Instruction::IfFirstSpellCastThisGame { skip: 1 },
            Instruction::GainLife(PlayerSet::Controller, 2),
        ],
    ),
    ..CardRules::EMPTY
};
