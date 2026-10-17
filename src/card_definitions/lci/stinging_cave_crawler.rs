// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::Expression {
                value: StatValue::Count(CountFilter {
                    zone: CountZone::Graveyard,
                    relation: 0,
                    kind: 371,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: true,
                }),
                minimum: Some(4),
                maximum: None,
            },
            event: &TriggerCondition::SelfAttacks,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::LoseLife(PlayerSet::Controller, 1),
        ],
    }],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords(128),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(128))],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(128))],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
