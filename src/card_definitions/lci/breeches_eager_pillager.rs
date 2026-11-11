const TREASURE: Card = Card::from_catalogue_id(5302);
const MODES: &[SpellMode] = &[
    SpellMode {
        text: "Create a Treasure token.",
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: TREASURE,
            amount: 1,
            tapped: false,
        }],
    },
    SpellMode {
        text: "Target creature can't block this turn.",
        targets: one(TargetRule::Creature),
        program: &[Instruction::GrantKeywords {
            subject: Subject::Targets,
            keywords: Keywords::CANT_BLOCK,
        }],
    },
    SpellMode {
        text: "Exile the top card of your library. You may play it this turn.",
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileTopPlay {
            amount: 1,
            next_turn: false,
        }],
    },
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::FIRST_STRIKE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledSubtypeAttacks {
            subtype: "Pirate",
            exclude: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ModalOncePerTurn(MODES)],
    }],
    ..CardRules::EMPTY
};
