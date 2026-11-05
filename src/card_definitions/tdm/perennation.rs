// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 0,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            predicate: &TargetPredicate::Permanent,
        })),
        &[Instruction::ReturnToBattlefieldWithCounters(&[
            (CounterKind::Hexproof, 1),
            (CounterKind::Indestructible, 1),
        ])],
    ),

    ..CardRules::EMPTY
};
