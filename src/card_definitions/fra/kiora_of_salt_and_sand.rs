// Directly authored from the full pinned Oracle and XMage sources; gated pending integration.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::LoyaltyAbilityActivatedThisTurn,
            event: &TriggerCondition::AttackDeclared {
                minimum: 1,
                power_min: None,
                guard: StepGuard::Always,
            },
        },
        targets: one(TargetRule::Filtered {
            base: &TargetRule::Creature,
            predicate: &TargetPredicate::Attacking,
        }),
        program: &[
            Instruction::Untap,
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::CANT_BE_BLOCKED,
            },
        ],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledPlaneswalkerActivation(0)],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: false,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: Some(-8),
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5455),
            amount: 1,
            tapped: false,
        }],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
