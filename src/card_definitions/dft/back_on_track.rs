// Complete Oracle/XMage: targeted own-graveyard union and an independent Pilot creation.
const OWN_GRAVE: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 0,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Union(&[
            TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            TargetRule::Filtered {
                base: &OWN_GRAVE,
                predicate: &TargetPredicate::Subtype("Vehicle"),
            },
        ]))),
        &[
            Instruction::ReturnToBattlefield,
            Instruction::CreateTokens {
                token: Card(5422),
                amount: 1,
                tapped: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};
