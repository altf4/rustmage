// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::CreateRole {
                token: Card(5248),
                subject: Subject::Targets,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::Draw,
                relation: CastRelation::You,
                guard: StepGuard::Threshold {
                    value: StepValue::EnchantedCreatures,
                    minimum: Some(1),
                    maximum: None,
                },
            },
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[
                Instruction::LoseLife(PlayerSet::Controller, 1),
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
        },
    ],
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
