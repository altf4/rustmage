// Direct typed behavior from complete pinned Oracle and XMage sources.
const CREATURES: BattlefieldFilter = BattlefieldFilter {
    kinds: 1,
    relation: -1,
    nonland: false,
    tapped: None,
    without: Keywords::NONE,
    subtype: None,
    max_mana: u32::MAX,
};
const GROUP: &[BattlefieldFilter] = &[
    CREATURES,
    BattlefieldFilter {
        kinds: 0,
        subtype: Some("Vehicle"),
        ..CREATURES
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::RemovePermanentGroupKeywords {
                filters: GROUP,
                keywords: Keywords::INDESTRUCTIBLE,
            },
            Instruction::MoveBattlefield {
                filters: GROUP,
                exile: false,
            },
        ],
    ),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: true,
        origin: ActivationOrigin::Hand,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            discard: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
