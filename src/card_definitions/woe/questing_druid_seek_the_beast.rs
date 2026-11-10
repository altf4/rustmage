// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Colors {
                filter: &CastFilter::Any,
                mask: 15,
            },
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    extras: &CardExtras {
        adventure_identity: Some("Questing Druid // Seek the Beast"),
        adventure_spell: Some(Card::from_catalogue_id(5560)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
