// Whole-card typed Oracle behavior; complete XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::REACH.union(Keywords::TRAMPLE),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::REACH.union(Keywords::TRAMPLE),
        )],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ManaSpent {
            minimum: 0,
            increment: false,
            event: &TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::InstantSorcery,
                ordinal: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GrowCreaturesScaled {
            filter: CreatureFilter {
                relation: 0,
                other: false,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            },
            excluding_subtype: None,
            amount: AmountValue::ColorsSpent,
            power: 1,
            toughness: 0,
        }],
    }],
    extras: &CardExtras {
        entry_counter_rules: &[EntryCounterRule {
            guard: StepGuard::Always,
            amount: EntryCounterAmount::ColorsSpent,
        }],
        cost_alternatives: &[
            ManaCost {
                colored: [0, 0, 0, 1, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 0, 0, 2, 0, 0],
                generic: 2,
            },
            ManaCost {
                colored: [0, 0, 0, 3, 0, 0],
                generic: 0,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
