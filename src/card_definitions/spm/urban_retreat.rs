// Direct typed rules from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::Always,
    activated: &[
        ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Choice {
                mask: 19,
                amount: 1,
            })],
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            sorcery: false,
            once_per_turn: false,
            cycling: false,
            exhaust: false,
        },
        ActivatedAbility {
            cost: ActivationCost {
                generic: 2,
                mana_component: true,
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::ReturnTappedCreature,
                    filter: SacrificeFilter {
                        kinds: 1,
                        keywords: Keywords::NONE,
                        source_only: false,
                        nonland: false,
                        other: false,
                        tokens: None,
                        artifact_enchantment_or_token: false,
                        subtype: None,
                    },
                }),
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutHandSourceOntoBattlefield],
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Hand,
            sorcery: true,
            once_per_turn: false,
            cycling: false,
            exhaust: false,
        },
    ],
    ..CardRules::EMPTY
};
