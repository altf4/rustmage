// Authored from pinned Oracle and full XMage card and color-count helper; awaiting tests.
const OTHER_ARTIFACTS: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 16,
    exclude: 0,
    subtype: None,
    basic: false,
    other: true,
    cards_only: false,
};
const INTRINSIC: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::IntrinsicKeywords(
        Keywords::VIGILANCE.union(Keywords::TRAMPLE),
    )],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::DrawScaled(
            PlayerSet::Controller,
            AmountValue::Stat(StatValue::Colors(OTHER_ARTIFACTS)),
        )],
    }],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::VIGILANCE.union(Keywords::TRAMPLE),
    static_groups: INTRINSIC,
    extras: &CardExtras {
        static_ability_groups: INTRINSIC,
        cost_alternatives: &[
            ManaCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 8,
            },
            ManaCost {
                colored: [0, 0, 0, 1, 0, 0],
                generic: 8,
            },
            ManaCost {
                colored: [0, 0, 0, 1, 1, 0],
                generic: 6,
            },
            ManaCost {
                colored: [0, 0, 1, 0, 0, 0],
                generic: 8,
            },
            ManaCost {
                colored: [0, 0, 1, 0, 1, 0],
                generic: 6,
            },
            ManaCost {
                colored: [0, 0, 1, 1, 0, 0],
                generic: 6,
            },
            ManaCost {
                colored: [0, 0, 1, 1, 1, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 1, 0, 0, 0, 0],
                generic: 8,
            },
            ManaCost {
                colored: [0, 1, 0, 0, 1, 0],
                generic: 6,
            },
            ManaCost {
                colored: [0, 1, 0, 1, 0, 0],
                generic: 6,
            },
            ManaCost {
                colored: [0, 1, 0, 1, 1, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 1, 1, 0, 0, 0],
                generic: 6,
            },
            ManaCost {
                colored: [0, 1, 1, 0, 1, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 1, 1, 1, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 1, 1, 1, 1, 0],
                generic: 2,
            },
            ManaCost {
                colored: [1, 0, 0, 0, 0, 0],
                generic: 8,
            },
            ManaCost {
                colored: [1, 0, 0, 0, 1, 0],
                generic: 6,
            },
            ManaCost {
                colored: [1, 0, 0, 1, 0, 0],
                generic: 6,
            },
            ManaCost {
                colored: [1, 0, 0, 1, 1, 0],
                generic: 4,
            },
            ManaCost {
                colored: [1, 0, 1, 0, 0, 0],
                generic: 6,
            },
            ManaCost {
                colored: [1, 0, 1, 0, 1, 0],
                generic: 4,
            },
            ManaCost {
                colored: [1, 0, 1, 1, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [1, 0, 1, 1, 1, 0],
                generic: 2,
            },
            ManaCost {
                colored: [1, 1, 0, 0, 0, 0],
                generic: 6,
            },
            ManaCost {
                colored: [1, 1, 0, 0, 1, 0],
                generic: 4,
            },
            ManaCost {
                colored: [1, 1, 0, 1, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [1, 1, 0, 1, 1, 0],
                generic: 2,
            },
            ManaCost {
                colored: [1, 1, 1, 0, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [1, 1, 1, 0, 1, 0],
                generic: 2,
            },
            ManaCost {
                colored: [1, 1, 1, 1, 0, 0],
                generic: 2,
            },
            ManaCost {
                colored: [1, 1, 1, 1, 1, 0],
                generic: 0,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
