// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::UnlessBasic,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&TriggerCondition::SelfAttacks),
        targets: TargetPlan::NONE,
        program: &[Instruction::AddCombatRed(4)],
    }],
    activated: &[
        ActivatedAbility {
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            once_per_turn: false,
            sorcery: false,
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 1, 0, 0,
            ]))],
        },
        ActivatedAbility {
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            once_per_turn: false,
            sorcery: false,
            cost: ActivationCost {
                colored: [0, 0, 0, 1, 0, 0],
                generic: 1,
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
            program: &[Instruction::GrantTargetTriggerUntilEndOfTurn(0)],
        },
    ],
    ..CardRules::EMPTY
};
