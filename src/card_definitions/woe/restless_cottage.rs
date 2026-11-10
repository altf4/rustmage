// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::Always,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 0,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: 1,
        }),
        program: &[
            Instruction::CreateTokens {
                token: Card::from_catalogue_id(5246),
                amount: 1,
                tapped: false,
            },
            Instruction::Exile,
        ],
    }],
    activated: &[
        ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Choice {
                mask: 20,
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
                colored: [0, 0, 1, 0, 1, 0],
                generic: 2,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AnimateSourceColored(
                AnimationSpec {
                    artifact: false,
                    creature_types: AnimationCreatureTypes::Named(&["Horror"]),
                    base: Some((4, 4)),
                    keywords: Keywords::NONE,
                    duration: AnimationDuration::EndOfTurn,
                },
                20,
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
