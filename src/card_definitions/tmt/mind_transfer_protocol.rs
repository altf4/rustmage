// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 17,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 1,
            max: 1,
        }),
        &[
            Instruction::AnimateTargets(AnimationSpec {
                artifact: true,
                creature_types: AnimationCreatureTypes::Unchanged,
                base: Some((4, 5)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            }),
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};
