// Direct typed behavior; full Oracle and pinned XMage dependencies reviewed.
const LOOT: &[Instruction] = &[
    Instruction::ChooseObjects {
        kind: EffectChoiceKind::Discard,
        players: PlayerSet::Controller,
        filter: SacrificeFilter {
            keywords: Keywords::NONE,
            source_only: false,
            kinds: 0,
            nonland: false,
            other: false,
            tokens: None,
            artifact_enchantment_or_token: false,
            subtype: None,
        },
        amount: 1,
        optional_skip: Some(1),
    },
    Instruction::Draw(PlayerSet::Controller, 1),
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: LOOT,
        },
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::ManaValueMinimum {
                    filter: &CastFilter::Any,
                    minimum: 4,
                },
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: LOOT,
        },
    ],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::NONE,
    extras: &CardExtras {
        cost_alternatives: &[
            ManaCost {
                colored: [0, 1, 0, 1, 0, 0],
                generic: 1,
            },
            ManaCost {
                colored: [0, 2, 0, 0, 0, 0],
                generic: 1,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
