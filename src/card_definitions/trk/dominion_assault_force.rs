// Pinned Oracle and complete XMage class: modal entry choice.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::VIGILANCE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Exile target card from a graveyard.",
                targets: one(TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 0,
                    relation: -1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                }),
                program: &[Instruction::Exile],
            },
            SpellMode {
                text: "Surveil 1.",
                targets: TargetPlan::NONE,
                program: &[Instruction::Surveil(1)],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
