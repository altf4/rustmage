// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Conceal — Until end of turn, target creature you control becomes a Citizen with base power and toughness 1/1 and gains hexproof.",
            targets: one(TargetRule::creature_you_control()),
            program: &[Instruction::AnimateTargets(AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Named(&["Citizen"]),
                base: Some((1, 1)),
                keywords: Keywords::HEXPROOF,
                duration: AnimationDuration::EndOfTurn,
            })],
        },
        SpellMode {
            text: "Reveal — Until end of turn, target creature you control becomes a Hero with base power and toughness 3/4 and gains flying and vigilance.",
            targets: one(TargetRule::creature_you_control()),
            program: &[Instruction::AnimateTargets(AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Named(&["Hero"]),
                base: Some((3, 4)),
                keywords: Keywords::FLYING.union(Keywords::VIGILANCE),
                duration: AnimationDuration::EndOfTurn,
            })],
        },
    ]),
    ..CardRules::EMPTY
};
