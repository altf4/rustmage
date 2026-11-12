// Direct typed behavior from the pinned Oracle and XMage sources.
const CONTROLLED_CREATURES: &[BattlefieldFilter] = &[BattlefieldFilter {
    kinds: 1,
    relation: 0,
    nonland: false,
    tapped: None,
    without: Keywords::NONE,
    subtype: None,
    max_mana: u32::MAX,
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 10),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfTurnedFaceUp,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GrantPermanentGroupKeywords {
                filters: CONTROLLED_CREATURES,
                keywords: Keywords::HEXPROOF,
            },
            Instruction::UntapPermanentGroup(CONTROLLED_CREATURES),
        ],
    }],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec::fixed(ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 2,
        })),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
