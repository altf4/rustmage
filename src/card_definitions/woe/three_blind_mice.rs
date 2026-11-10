// Direct typed behavior from the pinned Oracle and XMage sources.
const MOUSE: Card = Card(5568);
const OWN_TOKEN: TargetPlan = one(TargetRule::Filtered {
    base: &TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 0,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    },
    predicate: &TargetPredicate::Token,
});
const COPY: &[Instruction] = &[Instruction::CreateCopyToken {
    subject: Subject::Targets,
    amount: 1,
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: MOUSE,
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: OWN_TOKEN,
            program: COPY,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: OWN_TOKEN,
            program: COPY,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 4 },
            targets: TargetPlan::NONE,
            program: &[Instruction::ModifyPlayerCreatures {
                players: PlayerSet::Controller,
                power: 1,
                toughness: 1,
                keywords: Keywords::VIGILANCE,
            }],
        },
    ],
    ..CardRules::EMPTY
};
