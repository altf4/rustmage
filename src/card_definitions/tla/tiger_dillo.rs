// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    triggers: &[],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords(0),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CombatRestriction(
            0,
            CombatRestriction {
                subject: Subject::Source,
                attack: true,
                block: true,
                unless: Some(StepGuard::Threshold {
                    value: StepValue::GreatestOtherCreaturePower,
                    minimum: Some(4),
                    maximum: None,
                }),
            },
        )],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CombatRestriction(
                0,
                CombatRestriction {
                    subject: Subject::Source,
                    attack: true,
                    block: true,
                    unless: Some(StepGuard::Threshold {
                        value: StepValue::GreatestOtherCreaturePower,
                        minimum: Some(4),
                        maximum: None,
                    }),
                },
            )],
        }],
        combat_restriction: Some(CombatRestriction {
            subject: Subject::Source,
            attack: true,
            block: true,
            unless: Some(StepGuard::Threshold {
                value: StepValue::GreatestOtherCreaturePower,
                minimum: Some(4),
                maximum: None,
            }),
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
