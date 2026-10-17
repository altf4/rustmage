// Direct typed behavior from complete Oracle and pinned XMage sources.
const OWN: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(OWN),
        &[Instruction::GrantKeywords {
            subject: Subject::Targets,
            keywords: Keywords::INDESTRUCTIBLE,
        }],
    ),
    extras: &CardExtras {
        kicker_cost: Some(ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 2,
        }),
        kicked_targets: Some(TargetPlan::Single(TargetSpec {
            rule: OWN,
            min: 0,
            max: usize::MAX,
        })),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
