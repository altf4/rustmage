// Pinned Oracle and XMage behavior; gated pending integration and review.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras { starting_loyalty: 5, ..CardExtras::EMPTY },
    triggers: &[
        TriggeredAbility { condition: TriggerCondition::FilteredEntersYou(EntryPredicate::CreaturePowerAtLeast(4)),
            targets: TargetPlan::NONE, program: &[Instruction::Draw(PlayerSet::Controller, 1)] },
        TriggeredAbility { condition: TriggerCondition::Granted(&TriggerCondition::Reflexive),
            targets: TargetPlan::NONE, program: &[Instruction::GrowDeclaredAttackers {
                power: 2, toughness: 2, keywords: Keywords::TRAMPLE,
            }] },
    ],
    activated: &[
        ActivatedAbility { exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
            cycling: false, once_per_turn: false, sorcery: true,
            cost: ActivationCost { colored: [0; 6], generic: 0, alternatives: &[], additional: None,
                mana_component: false, tap: false, sacrifice: false, exile_self: false,
                discard: false, life: 0, remove: None, loyalty: Some(2) },
            targets: TargetPlan::Single(TargetSpec { rule: TargetRule::Object {
                zone: TargetZone::Battlefield, kinds: 2, relation: -1, other: false,
                nonland: false, tapped: false, min_mana: 0, max_mana: u32::MAX,
            }, min: 0, max: 2 }), program: &[Instruction::Untap],
        },
        ActivatedAbility { exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
            cycling: false, once_per_turn: false, sorcery: true,
            cost: ActivationCost { colored: [0; 6], generic: 0, alternatives: &[], additional: None,
                mana_component: false, tap: false, sacrifice: false, exile_self: false,
                discard: false, life: 0, remove: None, loyalty: Some(-3) },
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens { token: Card(5239), amount: 1, tapped: false }],
        },
        ActivatedAbility { exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
            cycling: false, once_per_turn: false, sorcery: true,
            cost: ActivationCost { colored: [0; 6], generic: 0, alternatives: &[], additional: None,
                mana_component: false, tap: false, sacrifice: false, exile_self: false,
                discard: false, life: 0, remove: None, loyalty: Some(-4) },
            targets: TargetPlan::NONE, program: &[Instruction::ScheduleOpponentAttackUntilNextTurn { ability: 1 }],
        },
    ],
    ..CardRules::EMPTY
};
