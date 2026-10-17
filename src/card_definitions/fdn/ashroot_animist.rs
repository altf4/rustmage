// Direct typed behavior from the complete paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::TRAMPLE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 0,
            other: true,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::TRAMPLE,
            },
            Instruction::GrowScaled {
                subject: Subject::Targets,
                amount: AmountValue::SourcePower,
                power: 1,
                toughness: 1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
