const TOUGHNESS_DISCOUNT: SpellReduction = SpellReduction {
    source_only: true,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::TotalControlledCreatureToughness,
    turn: CostTurn::Any,
    guard: StepGuard::Always,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 15),
    keywords: Keywords::DEFENDER,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, TOUGHNESS_DISCOUNT)],
    }],
    extras: &CardExtras {
        cost_reductions: &[TOUGHNESS_DISCOUNT],
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 2, 0, 0, 0, 0],
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        program: &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 1,
                toughness: 0,
            },
            Instruction::GrantTargetTriggerUntilEndOfTurn(0),
            Instruction::TargetsAttackWithoutDefenderThisTurn,
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::DrawScaled(
            PlayerSet::Controller,
            AmountValue::SourceToughness,
        )],
    }],
    ..CardRules::EMPTY
};
