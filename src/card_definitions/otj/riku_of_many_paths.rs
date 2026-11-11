// Complete typed behavior from the pinned Oracle and XMage implementation.
const EXILE: Instruction = Instruction::ExileTopPlay {
    amount: 1,
    next_turn: true,
};
const COUNTER: Instruction = Instruction::PutCounters {
    subject: Subject::Source,
    kind: CounterKind::PlusOne,
    amount: 1,
};
const TRAMPLE: Instruction = Instruction::GrantKeywords {
    subject: Subject::Source,
    keywords: Keywords::TRAMPLE,
};
const BIRD: Instruction = Instruction::CreateTokens {
    token: Card(5616),
    amount: 1,
    tapped: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ModalSpellCastByYou,
        targets: TargetPlan::NONE,
        program: &[Instruction::ModalUpToX {
            modes: &[
                SpellMode {
                    text: "Choose no modes",
                    targets: TargetPlan::NONE,
                    program: &[],
                },
                SpellMode {
                    text: "Exile and play the top card",
                    targets: TargetPlan::NONE,
                    program: &[EXILE],
                },
                SpellMode {
                    text: "Counter and trample",
                    targets: TargetPlan::NONE,
                    program: &[COUNTER, TRAMPLE],
                },
                SpellMode {
                    text: "Exile and play the top card, and counter and trample",
                    targets: TargetPlan::NONE,
                    program: &[EXILE, COUNTER, TRAMPLE],
                },
                SpellMode {
                    text: "Create a Bird",
                    targets: TargetPlan::NONE,
                    program: &[BIRD],
                },
                SpellMode {
                    text: "Exile and play the top card, and create a Bird",
                    targets: TargetPlan::NONE,
                    program: &[EXILE, BIRD],
                },
                SpellMode {
                    text: "Counter and trample, and create a Bird",
                    targets: TargetPlan::NONE,
                    program: &[COUNTER, TRAMPLE, BIRD],
                },
                SpellMode {
                    text: "Exile and play the top card, counter and trample, and create a Bird",
                    targets: TargetPlan::NONE,
                    program: &[EXILE, COUNTER, TRAMPLE, BIRD],
                },
            ],
            counts: &[0, 1, 1, 2, 1, 2, 2, 3],
        }],
    }],
    ..CardRules::EMPTY
};
