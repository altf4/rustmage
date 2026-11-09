// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING,
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledLandColorlessTapManaBonus],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 2,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::Blight,
                amount: 1,
            },
            Instruction::BlankTargetLandWhileBlighted,
        ],
    }],
    ..CardRules::EMPTY
};
