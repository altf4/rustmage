// Direct Oracle behavior; the granted permission follows the exact graveyard card.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::HASTE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 12,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[Instruction::GrantGraveyardCast(
            GraveyardCastKind::Flashback,
        )],
    }],
    ..CardRules::EMPTY
};
