// Full pinned Oracle and XMage; both ordinary combat and Adventure are complete.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::CANT_BE_BLOCKED,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::CANT_BE_BLOCKED,
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
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::Discard(PlayerSet::Controller, 1),
        ],
    }],
    extras: &CardExtras {
        adventure_identity: Some("Bilbo, Luckwearer // Burglar's Plot"),
        adventure_spell: Some(Card(5419)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
