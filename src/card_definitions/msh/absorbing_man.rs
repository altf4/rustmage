// Complete frozen Oracle and direct pinned Java; owned in-place copies preserve physical identity.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Main1,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Union(&[
                TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 18,
                    relation: -1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                TargetRule::Filtered {
                    base: &TargetRule::Object {
                        zone: TargetZone::Battlefield,
                        kinds: 32,
                        relation: -1,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    predicate: &TargetPredicate::WithoutSubtype("Aura"),
                },
            ]),
            min: 0,
            max: 1,
        }),
        program: &[Instruction::BecomeCopy(InPlaceCopySpec {
            recipient_group: None,
            blueprint_group: 0,
            overrides: crate::engine::CopyOverrides {
                name: Some("Absorbing Man"),
                legendary: Some(true),
                kinds: None,
                types: Some(crate::engine::CopyTypes {
                    replace: false,
                    human_villain: true,
                    mercenary: false,
                    nightmare: false,
                    golem: false,
                    fractal: false,
                }),
                keywords: Keywords::VIGILANCE,
            },
            add_kinds: 1,
            base: Some((4, 4)),
            next_turn: true,
        })],
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
    keywords: Keywords::VIGILANCE,
    ..CardRules::EMPTY
};
