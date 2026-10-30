// Direct Oracle and pinned XMage behavior; gated pending rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::REACH,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::REACH)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::LandEntersYou,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::DamagePlayers(PlayerSet::Opponent, 1),
            Instruction::IfEventObjectSubtype {
                subtype: "Mountain",
                skip: 1,
            },
            Instruction::AddMana(ManaProduction::Fixed([0, 0, 0, 1, 0, 0])),
        ],
    }],
    ..CardRules::EMPTY
};
