// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Expression {
                value: StatValue::PermanentsSacrificedThisTurn,
                minimum: Some(1),
                maximum: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateCopyTokenWithExceptions {
            subject: Subject::Source,
            amount: 1,
            exceptions: CopyExceptions::NONE,
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::Crew { minimum: 1 }],
        sorcery: false,
        once_per_turn: false,
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Type,
        fragments: &[
            StaticAbilityFragment::SourceArtifactCreatureWhileControlNamed {
                name: "Phoenix Fleet Airship",
                minimum: 8,
            },
        ],
    }],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 4,
            toughness: 4,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
