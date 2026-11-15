// Oracle and official Star Trek release notes; no pinned XMage class exists.
const MATERIAL: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 1 | 16,
    nonland: false,
    other: true,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MaySacrificeForManaValue {
                filter: MATERIAL,
                skip: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 2),
        ],
    }],
    extras: &CardExtras {
        self_exile_cast: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
