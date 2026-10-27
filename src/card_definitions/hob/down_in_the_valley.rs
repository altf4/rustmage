// Direct typed behavior from the full pinned Oracle and XMage implementation.
const ELVES: &[BattlefieldFilter] = &[BattlefieldFilter {
    kinds: 0,
    relation: 0,
    nonland: false,
    tapped: None,
    without: Keywords::NONE,
    subtype: Some("Elf"),
    max_mana: u32::MAX,
}];
const ELF_BOOST: &[Instruction] = &[
    Instruction::GrowPermanentGroup {
        filters: ELVES,
        power: 1,
        toughness: 0,
    },
    Instruction::GrantPermanentGroupKeywords {
        filters: ELVES,
        keywords: Keywords::VIGILANCE,
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: &[Instruction::Search(SearchSpec {
                filter: SearchFilter::BasicLand,
                maximum: 1,
                destination: SearchDestination::Hand,
                tapped: false,
                reveal: true,
            })],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: &[Instruction::GrantSourcePermanentTrigger(4)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: ELF_BOOST,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 4 },
            targets: TargetPlan::NONE,
            program: ELF_BOOST,
        },
        TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::LandEntersYou),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5297),
                amount: 1,
                tapped: false,
            }],
        },
    ],
    ..CardRules::EMPTY
};
