// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Union(&[
            TargetRule::Creature,
            TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 64,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
        ])),
        &[
            Instruction::Destroy,
            Instruction::MayTargetController { skip: 1 },
            Instruction::SearchTargetController(SearchSpec {
                filter: SearchFilter::BasicLand,
                maximum: 1,
                destination: SearchDestination::Battlefield,
                tapped: true,
                reveal: false,
            }),
        ],
    ),
    ..CardRules::EMPTY
};
