// Direct typed behavior from the complete pinned Oracle and XMage sources.
const LAND: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 2,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
}));
const SWEEP: Instruction = Instruction::DamageBattlefield {
    filters: &[BattlefieldFilter {
        kinds: 1,
        relation: -1,
        nonland: false,
        tapped: None,
        without: Keywords::NONE,
        subtype: None,
        max_mana: u32::MAX,
    }],
    players: None,
    amount: 3,
};
const SEARCH: Instruction = Instruction::SearchTargetController(SearchSpec {
    filter: SearchFilter::BasicLand,
    maximum: 1,
    destination: SearchDestination::Battlefield,
    tapped: true,
    reveal: false,
});
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Deal 3 damage to each creature",
            targets: TargetPlan::NONE,
            program: &[SWEEP],
        },
        SpellMode {
            text: "Destroy target land; its controller may search for a basic land",
            targets: LAND,
            program: &[
                Instruction::Destroy,
                Instruction::MayTargetController { skip: 1 },
                SEARCH,
            ],
        },
        SpellMode {
            text: "Choose both",
            targets: LAND,
            program: &[
                SWEEP,
                Instruction::Destroy,
                Instruction::MayTargetController { skip: 1 },
                SEARCH,
            ],
        },
    ]),
    ..CardRules::EMPTY
};
