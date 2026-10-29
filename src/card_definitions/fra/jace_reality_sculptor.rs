// Pinned Oracle and XMage behavior; gated pending validation and review.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras { starting_loyalty: 5, ..CardExtras::EMPTY },
    triggers: &[
        TriggeredAbility { condition: TriggerCondition::Granted(&TriggerCondition::Reflexive),
            targets: TargetPlan::NONE, program: &[Instruction::WeakenDeclaredAttacker(-5)] },
    ],
    activated: &[
        ActivatedAbility { exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
            cycling: false, once_per_turn: false, sorcery: true,
            cost: ActivationCost { colored: [0; 6], generic: 0, alternatives: &[], additional: None,
                mana_component: false, tap: false, sacrifice: false, exile_self: false,
                discard: false, life: 0, remove: None, loyalty: Some(1) },
            targets: TargetPlan::NONE, program: &[
                Instruction::EnsureJaceToken,
                Instruction::EmpowerJaceScaled(AmountValue::Stat(
                    StatValue::Count(CountFilter { zone: CountZone::Battlefield, relation: 0, kind: 0,
                        exclude: 0, subtype: Some("Island"), basic: false, other: false, cards_only: false })
                )),
            ],
        },
        ActivatedAbility { exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
            cycling: false, once_per_turn: false, sorcery: true,
            cost: ActivationCost { colored: [0; 6], generic: 0, alternatives: &[], additional: None,
                mana_component: false, tap: false, sacrifice: false, exile_self: false,
                discard: false, life: 0, remove: None, loyalty: Some(-3) },
            targets: TargetPlan::NONE, program: &[Instruction::ScheduleOpponentAttackUntilNextTurn { ability: 0 }],
        },
        ActivatedAbility { exhaust: false,
            guard: ActivationGuard::ControlledPlaneswalkerSubtypeLoyaltyAtLeast { subtype: "Jace", minimum: 25 },
            origin: ActivationOrigin::Battlefield, cycling: false, once_per_turn: false, sorcery: true,
            cost: ActivationCost { colored: [0; 6], generic: 0, alternatives: &[], additional: None,
                mana_component: false, tap: false, sacrifice: false, exile_self: false,
                discard: false, life: 0, remove: None, loyalty: Some(0) },
            targets: TargetPlan::NONE, program: &[Instruction::ExileOpponentLibrariesExceptBottom],
        },
    ],
    ..CardRules::EMPTY
};
