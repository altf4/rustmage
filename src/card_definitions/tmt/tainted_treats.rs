// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 17,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 1,
            max: 1,
        }),
        &[
            Instruction::IfTargetManaValueAtMost {
                maximum: 4,
                skip: 3,
            },
            Instruction::Destroy,
            Instruction::CreateTokens {
                token: Card(5246),
                amount: 1,
                tapped: false,
            },
            Instruction::Skip(1),
            Instruction::Destroy,
        ],
    ),
    ..CardRules::EMPTY
};
