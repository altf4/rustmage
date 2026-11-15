// Complete Oracle behavior, checked against pinned XMage.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5246),
            amount: 1,
            tapped: false,
        }],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Filtered {
            base: &TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            predicate: &TargetPredicate::Token,
        }),
        program: &[Instruction::CreateTappedCopyToken {
            subject: Subject::Targets,
            amount: 1,
        }],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        sorcery: false,
        once_per_turn: false,
        cycling: false,
        exhaust: false,
    }],
    ..CardRules::EMPTY
};
