// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Return target legendary card from your graveyard to your hand.",
                targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                    base: &TargetRule::Object {
                        zone: TargetZone::Graveyard,
                        kinds: 0,
                        relation: 0,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    predicate: &TargetPredicate::Legendary,
                })),
                program: &[Instruction::ReturnToHand],
            },
            SpellMode {
                text: "Put a +1/+1 counter on target creature. It gains vigilance and indestructible until end of turn.",
                targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
                program: &[
                    Instruction::PutCounters {
                        subject: Subject::Targets,
                        kind: CounterKind::PlusOne,
                        amount: 1,
                    },
                    Instruction::GrantKeywords {
                        subject: Subject::Targets,
                        keywords: Keywords::VIGILANCE.union(Keywords::INDESTRUCTIBLE),
                    },
                ],
            },
        ])],
    }],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::NONE,
    ..CardRules::EMPTY
};
