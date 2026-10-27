// Direct typed behavior from complete pinned Oracle and XMage implementation.
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
            text: "Create a 1/1 colorless Pilot creature token with “This token saddles Mounts and crews Vehicles as though its power were 2 greater.”",
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5422),
                amount: 1,
                tapped: false,
            }],
        },
    ]),
    ..CardRules::EMPTY
};
