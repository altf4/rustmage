// Directly authored from the complete pinned Oracle and XMage sources; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::FLASH,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLASH)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Target creature gets -X/-0 until end of turn, where X is the number of cards in your graveyard.",
                targets: one(TargetRule::Creature),
                program: &[Instruction::GrowScaled {
                    subject: Subject::Targets,
                    amount: AmountValue::Stat(StatValue::Count(CountFilter {
                        zone: CountZone::Graveyard,
                        relation: 0,
                        kind: 0,
                        exclude: 0,
                        subtype: None,
                        basic: false,
                        other: false,
                        cards_only: true,
                    })),
                    power: -1,
                    toughness: 0,
                }],
            },
            SpellMode {
                text: "Surveil 2.",
                targets: TargetPlan::NONE,
                program: &[Instruction::Surveil(2)],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
