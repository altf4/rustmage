pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::DEATHTOUCH,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::DEATHTOUCH,
        )],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: one(TargetRule::DamagedPlayerNonlandPermanentCard),
        program: &[Instruction::CastTargetedGraveyardPermanentPaid {
            spend_any_mana: true,
        }],
    }],
    ..CardRules::EMPTY
};
