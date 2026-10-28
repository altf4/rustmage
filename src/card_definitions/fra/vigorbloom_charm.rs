// Direct Oracle behavior and pinned XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Target permanent you control gains hexproof and indestructible until end of turn.",
            targets: one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE),
            }],
        },
        SpellMode {
            text: "You draw a card and gain 3 life.",
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::GainLife(PlayerSet::Controller, 3),
            ],
        },
        SpellMode {
            text: "Put a +1/+1 counter on target creature you control. Then it fights target creature an opponent controls.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::creature_you_control()),
                TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: 1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                }),
            ]),
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
                Instruction::Fight {
                    source: 0,
                    recipient: 1,
                },
            ],
        },
    ]),
    ..CardRules::EMPTY
};
