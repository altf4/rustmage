// Direct typed rules from pinned Oracle and XMage; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::FLYING,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
    }],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                event: &TriggerCondition::AtStep {
                    step: TriggerStep::End,
                    relation: CastRelation::Any,
                    guard: StepGuard::Always,
                },
                guard: StepGuard::Expression {
                    value: StatValue::CardsDrawnThisTurn,
                    minimum: Some(3),
                    maximum: None,
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5457),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::PlayerDamaged {
                subject: DamageSubject::Source,
                combat_only: true,
                opponent_only: false,
                one_or_more: false,
            }),
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 2)],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0, 2, 0, 0, 0, 0],
            generic: 3,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantSourceTriggerUntilEndOfTurn(1)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
