// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::FIRST_STRIKE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::FIRST_STRIKE,
        )],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::PutFilteredCounters {
            filter: CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 16,
                exclude: 0,
                subtype: Some("Equipment"),
                basic: false,
                other: false,
                cards_only: false,
            },
            kind: CounterKind::Hone,
            amount: 1,
        }],
    }],
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 1,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
