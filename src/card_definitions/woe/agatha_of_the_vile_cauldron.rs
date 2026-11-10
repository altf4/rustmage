// Direct typed behavior from the pinned Oracle and XMage sources.
const OTHER_CREATURES: CreatureFilter = CreatureFilter {
    relation: 0,
    other: true,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ControlledActivationReduction {
            kinds: 1,
            amount: ActivationReductionAmount::ProviderPower,
            leave_one_mana: true,
        }],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            colored: [0, 0, 0, 1, 1, 0],
            generic: 4,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GrowCreatures {
                filter: OTHER_CREATURES,
                power: 1,
                toughness: 1,
            },
            Instruction::GrantCreatures {
                filter: OTHER_CREATURES,
                keywords: Keywords::TRAMPLE.union(Keywords::HASTE),
            },
        ],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
