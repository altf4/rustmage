pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::DamagePlayers(PlayerSet::Opponent, 4)],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                guard: StepGuard::Expression {
                    value: StatValue::CONTROLLER_GRAVEYARD_CARD_TYPES,
                    minimum: Some(4),
                    maximum: None,
                },
                event: &TriggerCondition::PlayerDamaged {
                    subject: DamageSubject::ControlledNoncombatSources,
                    combat_only: false,
                    opponent_only: true,
                    one_or_more: false,
                },
            },
            targets: one(TargetRule::DamagedPlayerPermanent { kinds: 1 }),
            program: &[Instruction::DamageScaled(AmountValue::DamageEventAmount)],
        },
    ],
    ..CardRules::EMPTY
};
