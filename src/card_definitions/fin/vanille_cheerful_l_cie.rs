// Direct typed behavior from the paired Oracle and pinned XMage sources.
const PAIR: MeldPair = MeldPair {
    source: Card(4846),
    partner: Card(1443),
    combined: Card(3344),
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(2)),
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::ReturnGraveyardToHand {
                        kinds: 1 | 2 | 16 | 32 | 64 | 256,
                    },
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
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::Main1,
                relation: CastRelation::You,
                guard: StepGuard::OwnSourceAndNamedCreature(PAIR.partner),
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayMana {
                    cost: ManaCost {
                        colored: [0, 0, 1, 0, 1, 0],
                        generic: 3,
                    },
                    skip: 1,
                },
                Instruction::Meld(PAIR),
            ],
        },
    ],
    ..CardRules::EMPTY
};
