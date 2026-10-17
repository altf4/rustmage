// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
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
            event: &TriggerCondition::SelfEnters,
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 0,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            predicate: &TargetPredicate::Permanent,
        })),
        program: &[Instruction::ReturnToHand],
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
    ..CardRules::EMPTY
};
