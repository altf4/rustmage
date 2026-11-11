// Add Dinosaur, artifact and creature while retaining the target's existing types.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1 | 16,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::AnimateTargets(AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Additional(&["Dinosaur"]),
            base: Some((4, 3)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        })],
    ),
    ..CardRules::EMPTY
};
