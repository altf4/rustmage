// Direct typed behavior from the pinned Oracle and XMage sources.
const CONTROLLED: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 0,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Put five charge counters on target Spacecraft or Planet you control.",
            targets: one(TargetRule::Union(&[
                TargetRule::Filtered {
                    base: &CONTROLLED,
                    predicate: &TargetPredicate::Subtype("Spacecraft"),
                },
                TargetRule::Filtered {
                    base: &CONTROLLED,
                    predicate: &TargetPredicate::Subtype("Planet"),
                },
            ])),
            program: &[Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::Charge,
                amount: 5,
            }],
        },
        SpellMode {
            text: "Destroy target artifact.",
            targets: one(object(16)),
            program: &[Instruction::Destroy],
        },
    ]),
    ..CardRules::EMPTY
};
