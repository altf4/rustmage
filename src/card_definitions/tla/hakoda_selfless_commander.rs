// Direct typed behavior from the pinned Oracle and XMage sources.
const CREATURES: CreatureFilter = CreatureFilter {
    relation: 0,
    other: false,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 5),
    keywords: Keywords::VIGILANCE,
    static_groups: &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::LookAtLibraryTop],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CastFromLibraryTop(
                CastFilter::Subtype("Ally"),
            )],
        },
    ],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        sorcery: false,
        once_per_turn: false,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GrowCreatures {
                filter: CREATURES,
                power: 0,
                toughness: 5,
            },
            Instruction::GrantCreatures {
                filter: CREATURES,
                keywords: Keywords::INDESTRUCTIBLE,
            },
        ],
    }],
    ..CardRules::EMPTY
};
