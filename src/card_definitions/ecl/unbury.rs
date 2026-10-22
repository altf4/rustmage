// Direct typed behavior from the pinned Oracle and complete XMage sources.
const GRAVE: TargetRule = TargetRule::Object { zone: TargetZone::Graveyard, kinds: 1, relation: 0, other: false, nonland: false, tapped: false, min_mana: 0, max_mana: u32::MAX };
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode { text: "Return target creature card from your graveyard to your hand.", targets: one(GRAVE), program: &[Instruction::ReturnToHand] },
        SpellMode { text: "Return two target creature cards that share a creature type from your graveyard to your hand.", targets: TargetPlan::Single(TargetSpec { rule: TargetRule::Grouped { base: &GRAVE, constraint: TargetGroupConstraint::SharedCreatureType }, min: 2, max: 2 }), program: &[Instruction::ReturnToHand] },
    ]),
    ..CardRules::EMPTY
};
