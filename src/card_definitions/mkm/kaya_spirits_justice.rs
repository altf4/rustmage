// Partial typed preparation; gated until the exile-batch trigger and Spirit token work.
const fn loyalty(n: i32) -> ActivationCost {
    ActivationCost {
        colored: [0; 6],
        generic: 0,
        alternatives: &[],
        additional: None,
        mana_component: false,
        tap: false,
        sacrifice: false,
        exile_self: false,
        discard: false,
        life: 0,
        remove: None,
        loyalty: Some(n),
    }
}
const OPPONENT_CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: 1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Unsupported,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreaturesOrGraveyardCardsExiled,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
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
        })),
        program: &[Instruction::CopyChosenExiledTriggerCreature],
    }],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(2),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Surveil(2),
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::ExileCardFromAnyGraveyard,
                    players: PlayerSet::Controller,
                    filter: SacrificeFilter {
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 0,
                        nonland: false,
                        other: false,
                        tokens: None,
                        artifact_enchantment_or_token: false,
                        subtype: None,
                    },
                    amount: 1,
                    optional_skip: None,
                },
            ],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(1),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5477),
                amount: 1,
                tapped: false,
            }],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(-2),
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::creature_you_control()),
                TargetSpec {
                    rule: OPPONENT_CREATURE,
                    min: 0,
                    max: 1,
                },
            ]),
            program: &[Instruction::Exile],
            sorcery: true,
            once_per_turn: false,
        },
    ],
    extras: &CardExtras {
        starting_loyalty: 3,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
