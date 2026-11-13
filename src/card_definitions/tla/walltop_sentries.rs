// Direct typed behavior from the paired Oracle and pinned XMage sources.
const ABILITIES: Keywords = Keywords::REACH.union(Keywords::DEATHTOUCH);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: ABILITIES,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(ABILITIES)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            event: &TriggerCondition::SelfDies,
            guard: StepGuard::Expression {
                value: StatValue::Count(CountFilter {
                    zone: CountZone::Graveyard,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Lesson"),
                    basic: false,
                    other: false,
                    cards_only: true,
                }),
                minimum: Some(1),
                maximum: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
    }],
    ..CardRules::EMPTY
};
