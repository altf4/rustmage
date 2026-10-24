// Complete nonphysical back face, with ordinary damage prevention and draw observers.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::FLASH.union(Keywords::DOUBLE_STRIKE),
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::Replacement(
            0,
            ReplacementSpec::PreventAllDamageToSelf,
        )],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    extras: &CardExtras {
        replacement_effects: &[ReplacementSpec::PreventAllDamageToSelf],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
