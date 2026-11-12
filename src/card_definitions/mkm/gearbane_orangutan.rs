pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::REACH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Destroy up to one target artifact.",
                targets: TargetPlan::Single(TargetSpec {
                    rule: TargetRule::Object {
                        zone: TargetZone::Battlefield,
                        kinds: 16,
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
                program: &[Instruction::Destroy],
            },
            SpellMode {
                text: "Sacrifice an artifact to put two +1/+1 counters on this creature.",
                targets: TargetPlan::NONE,
                program: &[
                    Instruction::ChooseObjects {
                        kind: EffectChoiceKind::SacrificeAsCost,
                        players: PlayerSet::Controller,
                        filter: SacrificeFilter {
                            keywords: Keywords::NONE,
                            source_only: false,
                            kinds: 16,
                            nonland: false,
                            other: false,
                            tokens: None,
                            artifact_enchantment_or_token: false,
                            subtype: None,
                        },
                        amount: 1,
                        optional_skip: None,
                    },
                    Instruction::IfAmountAtLeast {
                        value: AmountValue::MovedCount,
                        minimum: 1,
                        skip: 1,
                    },
                    Instruction::PutCounters {
                        subject: Subject::Source,
                        kind: CounterKind::PlusOne,
                        amount: 2,
                    },
                ],
            },
        ])],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::REACH)],
    }],
    ..CardRules::EMPTY
};
