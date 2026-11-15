// Oracle and official Star Trek release notes; no pinned XMage class exists.
const HAND_CARD: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: false,
    tokens: Some(false),
    artifact_enchantment_or_token: false,
    subtype: None,
};
const DRAW_AND_LINK: &[Instruction] = &[
    Instruction::Draw(PlayerSet::Controller, 2),
    Instruction::ChooseObjects {
        kind: EffectChoiceKind::ExileFromHandLinked,
        players: PlayerSet::Controller,
        filter: HAND_CARD,
        amount: 1,
        optional_skip: None,
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: DRAW_AND_LINK,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: DRAW_AND_LINK,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: DRAW_AND_LINK,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 4 },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::IfLinkedExileSharesCardType { skip: 1 },
                Instruction::GainLife(PlayerSet::Controller, 10),
            ],
        },
    ],
    ..CardRules::EMPTY
};
