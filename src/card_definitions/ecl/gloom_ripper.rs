// Direct typed behavior from the pinned Oracle and complete XMage sources.
const ELVES: AmountValue = AmountValue::Stat(StatValue::Sum(&[
    StatValue::Count(CountFilter {
        zone: CountZone::Battlefield,
        relation: 0,
        kind: 0,
        exclude: 0,
        subtype: Some("Elf"),
        basic: false,
        other: false,
        cards_only: false,
    }),
    StatValue::Count(CountFilter {
        zone: CountZone::Graveyard,
        relation: 0,
        kind: 0,
        exclude: 0,
        subtype: Some("Elf"),
        basic: false,
        other: false,
        cards_only: true,
    }),
]));
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::creature_you_control()),
            TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: 1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: 1,
            },
        ]),
        program: &[
            Instruction::SelectGroup(0),
            Instruction::GrowScaled {
                subject: Subject::Targets,
                amount: ELVES,
                power: 1,
                toughness: 0,
            },
            Instruction::SelectGroup(1),
            Instruction::GrowScaled {
                subject: Subject::Targets,
                amount: ELVES,
                power: 0,
                toughness: -1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
