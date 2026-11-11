// Draw only when the optional discard actually occurs.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::Always,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::DiscardUpToOneDraw],
    }],
    activated: &[
        ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Choice {
                mask: 4 | 8,
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
                colored: [0, 0, 1, 1, 0, 0],
                generic: 1,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AnimateSourceColored(
                AnimationSpec {
                    artifact: false,
                    creature_types: AnimationCreatureTypes::Additional(&["Insect"]),
                    base: Some((2, 3)),
                    keywords: Keywords::MENACE,
                    duration: AnimationDuration::EndOfTurn,
                },
                4 | 8,
            )],
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
