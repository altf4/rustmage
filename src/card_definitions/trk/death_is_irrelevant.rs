// Pinned Oracle and official Star Trek rules evidence; XMage has no set-table class.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledPermanentAllKindsKeywords {
            kinds: 1 | 16,
            keywords: Keywords::INDESTRUCTIBLE,
        }],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 1,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[Instruction::Assimilate(Subject::Targets)],
    }],
    ..CardRules::EMPTY
};
