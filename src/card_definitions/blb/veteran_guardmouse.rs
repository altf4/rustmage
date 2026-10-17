// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(&TriggerCondition::SelfTargeted {
            relation: CastRelation::You,
            spells_only: false,
        }),
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::Grow {
                subject: Subject::Source,
                power: 1,
                toughness: 0,
            },
            Instruction::GrantKeywords {
                subject: Subject::Source,
                keywords: Keywords(512),
            },
            Instruction::Scry(1),
        ],
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
    keywords: Keywords(0),
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 3,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
