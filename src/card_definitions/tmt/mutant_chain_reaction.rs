// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Union(&[
                TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 48,
                    relation: -1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                TargetRule::Filtered {
                    base: &TargetRule::Creature,
                    predicate: &TargetPredicate::Keyword(Keywords::FLYING),
                },
            ]),
            min: 0,
            max: 1,
        }),
        &[
            Instruction::Destroy,
            Instruction::CreateTokens {
                token: Card(5278),
                amount: 1,
                tapped: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};
