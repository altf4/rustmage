// Directly authored from complete pinned Oracle and XMage executable behavior.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Creature)),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords(0),
        },
    }),
    extras: &CardExtras {
        aura: Some(AuraSpec {
            kinds: 1,
            relation: -1,
            bonus: EquipmentSpec {
                stats: Stats {
                    power: 0,
                    toughness: 0,
                },
                keywords: Keywords(0),
            },
        }),
        ..CardExtras::EMPTY
    },
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::EquippedAttacks,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Attached,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
                Instruction::SacrificeSourceIfAttachedCounters { minimum: 3 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Sacrificed {
                filter: SacrificeFilter {
                    keywords: Keywords(0),
                    source_only: true,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                own_turn: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Search(SearchSpec {
                filter: SearchFilter::BasicLand,
                maximum: 2,
                destination: SearchDestination::Battlefield,
                tapped: true,
                reveal: false,
            })],
        },
    ],
    ..CardRules::EMPTY
};
