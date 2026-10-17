// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::CreateTokens {
            token: Card(5278),
            amount: 1,
            tapped: false,
        }],
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
    keywords: Keywords(0),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(
            0,
            ReplacementSpec::Counters {
                filter: ReplacementFilter::Creature,
                kind: Some(CounterKind::PlusOne),
                factor: 1,
                add: 1,
                effects_only: false,
            },
        )],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Replacement(
                0,
                ReplacementSpec::Counters {
                    filter: ReplacementFilter::Creature,
                    kind: Some(CounterKind::PlusOne),
                    factor: 1,
                    add: 1,
                    effects_only: false,
                },
            )],
        }],
        replacement_effects: &[ReplacementSpec::Counters {
            filter: ReplacementFilter::Creature,
            kind: Some(CounterKind::PlusOne),
            factor: 1,
            add: 1,
            effects_only: false,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
