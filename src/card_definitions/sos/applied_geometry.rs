// Copy exceptions are copiable; the six counters are a subsequent replaced event.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            predicate: &TargetPredicate::WithoutSubtype("Aura"),
        })),
        &[Instruction::CreateCounteredCopyToken {
            subject: Subject::Targets,
            exceptions: CopyExceptions::FRACTAL,
            kind: CounterKind::PlusOne,
            amount: 6,
        }],
    ),
    ..CardRules::EMPTY
};
