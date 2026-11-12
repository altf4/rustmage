// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 2),
    keywords: Keywords::FLYING.union(Keywords::LIFELINK),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterUnlessPay {
                cost: generic(2),
                subject: CounterSubject::Related,
                exile: false,
                unpaid_skip: 0,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfTurnedFaceUp,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Union(&[
                    TargetRule::Object {
                        zone: TargetZone::Battlefield,
                        kinds: 1,
                        relation: -1,
                        other: true,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    TargetRule::Object {
                        zone: TargetZone::Graveyard,
                        kinds: 1,
                        relation: -1,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                ]),
                min: 0,
                max: usize::MAX,
            }),
            program: &[
                Instruction::UpToTargets(AmountValue::ChosenX),
                Instruction::ExileTargetsLinkedToSource,
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfLeaves,
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnLinkedExileToHand],
        },
    ],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec {
            mana: ManaCost {
                colored: [1, 0, 0, 0, 0, 0],
                generic: 3,
            },
            alternatives: &[],
            modifier: DisguiseCostModifier::X(std::num::NonZeroU8::MIN),
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
