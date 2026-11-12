// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ReturnToHandCounting,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    kinds: 1,
                    nonland: false,
                    other: true,
                    source_only: false,
                    tokens: None,
                    subtype: None,
                    keywords: Keywords::NONE,
                    artifact_enchantment_or_token: false,
                },
                amount: 1,
                optional_skip: None,
            },
            Instruction::IfAmountAtLeast {
                value: AmountValue::MovedCount,
                minimum: 1,
                skip: 2,
            },
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::Skip(2),
            Instruction::MayIfSourceOnBattlefield { skip: 1 },
            Instruction::ReturnSourceToHand(TargetZone::Battlefield),
        ],
    }],
    ..CardRules::EMPTY
};
