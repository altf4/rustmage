// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Filtered {
            base: &TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 0,
                relation: -1,
                other: false,
                nonland: true,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            predicate: &TargetPredicate::EnteredGraveyardThisTurn,
        }),
        &[Instruction::ExileTargetCopyCastFree],
    ),
    ..CardRules::EMPTY
};
