// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::Always,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::Scry(1)],
    }],
    activated: &[
        ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Choice {
                mask: 10,
                amount: 1,
            })],
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            exhaust: false,
            cycling: false,
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            cost: ActivationCost {
                colored: [0, 1, 0, 1, 0, 0],
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AnimateSourceColoredWithAbility {
                spec: &AnimationSpec {
                    artifact: false,
                    creature_types: AnimationCreatureTypes::Named(&["Elemental"]),
                    base: Some((2, 1)),
                    keywords: Keywords::NONE,
                    duration: AnimationDuration::EndOfTurn,
                },
                colors: 10,
                kind: SelfGrantedAbilityKind::OwnTurnFirstStrike,
            }],
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            exhaust: false,
            cycling: false,
            sorcery: false,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
