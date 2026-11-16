// Oracle and official Star Trek release notes; no pinned XMage class exists.
const OWN_NON_DOCTOR_PERMANENT_CARD: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Filtered {
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
        predicate: &TargetPredicate::Permanent,
    },
    predicate: &TargetPredicate::WithoutSubtype("Doctor"),
};
const OTHER_CREATURE: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Creature,
    predicate: &TargetPredicate::OtherThanSource,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::FLASH,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::Modal(&[
                SpellMode {
                    text: "Return target non-Doctor permanent card from your graveyard to your hand.",
                    targets: one(OWN_NON_DOCTOR_PERMANENT_CARD),
                    program: &[Instruction::ReturnToHand],
                },
                SpellMode {
                    text: "Choose up to one other target creature. When it dies this turn, return that card under its owner's control with a +1/+1 counter at the beginning of the next end step.",
                    targets: TargetPlan::Single(TargetSpec {
                        rule: OTHER_CREATURE,
                        min: 0,
                        max: 1,
                    }),
                    program: &[Instruction::DelayTargetEvent {
                        event: DelayedTargetEvent::DiesOnce,
                        ability: 1,
                    }],
                },
            ])],
        },
        TriggeredAbility {
            condition: TriggerCondition::DelayedTargetEvent,
            targets: TargetPlan::NONE,
            program: &[Instruction::ScheduleDelayedDeathReturnWithCounter(
                CounterKind::PlusOne,
            )],
        },
    ],
    ..CardRules::EMPTY
};
