// Direct typed behavior from the full pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::CantDrawCards,
            StaticAbilityFragment::CantGainLife,
        ],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Draw,
            relation: CastRelation::Any,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::LoseLife(PlayerSet::Active, 3),
            Instruction::SearchFor {
                player: PlayerSet::Active,
                required: true,
                spec: SearchSpec {
                    filter: SearchFilter::Any,
                    maximum: 1,
                    destination: SearchDestination::Hand,
                    tapped: false,
                    reveal: false,
                },
            },
        ],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[
                StaticAbilityFragment::CantDrawCards,
                StaticAbilityFragment::CantGainLife,
            ],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
