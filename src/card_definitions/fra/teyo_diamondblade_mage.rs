// Directly authored from pinned Oracle and XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    keywords: Keywords::FLASH,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLASH)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 0,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::DEATHTOUCH,
            },
            Instruction::PutCountersOnTargetKinds {
                kinds: 1,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::PutCountersOnTargetKinds {
                kinds: 64,
                kind: CounterKind::Loyalty,
                amount: 1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
