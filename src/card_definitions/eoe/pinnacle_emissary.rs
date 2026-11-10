const BLUE: ManaCost = ManaCost {
    colored: [0, 1, 0, 0, 0, 0],
    generic: 0,
};
const RED: ManaCost = ManaCost {
    colored: [0, 0, 0, 1, 0, 0],
    generic: 0,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    extras: &CardExtras {
        warp: Some(&WarpSpec {
            alternatives: &[BLUE, RED],
            ..WarpSpec::fixed(BLUE)
        }),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Artifact,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5537),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
