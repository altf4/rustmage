// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
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
                filter: CastFilter::Creature,
                ordinal: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddCastEntryCounters {
            kind: CounterKind::PlusOne,
            amount: AmountValue::ColorsSpent,
        }],
    }],
    extras: &CardExtras {
        entry_counter_rules: &[EntryCounterRule {
            guard: StepGuard::Always,
            amount: EntryCounterAmount::ColorsSpent,
        }],
        cost_alternatives: &[
            ManaCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 2,
            },
            ManaCost {
                colored: [0, 0, 0, 0, 2, 0],
                generic: 0,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
