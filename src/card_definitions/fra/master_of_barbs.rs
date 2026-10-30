// Directly authored from pinned Oracle and XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::MENACE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::MENACE)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::AnyNoncombatSources,
            combat_only: false,
            opponent_only: true,
            one_or_more: true,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GrowPlayerCreatures {
            players: PlayerSet::Controller,
            power: 1,
            toughness: 0,
        }],
    }],
    ..CardRules::EMPTY
};
