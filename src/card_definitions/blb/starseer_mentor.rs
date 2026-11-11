const KEYWORDS: Keywords = Keywords::FLYING.union(Keywords::VIGILANCE);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 5),
    keywords: KEYWORDS,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(KEYWORDS)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            event: &TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            guard: StepGuard::GainedOrLostLife,
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        program: &[Instruction::OpponentSacrificeDiscardOrLoseLife {
            repetitions: AmountValue::Fixed(1),
            life: 3,
        }],
    }],
    ..CardRules::EMPTY
};
