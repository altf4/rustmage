// Direct Oracle and pinned XMage rules; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Union(&[
        object(16),
        TargetRule::Filtered { base: &object(32), predicate: &TargetPredicate::WithoutSubtype("Aura") },
    ]))),
    aura: Some(AuraSpec {
        kinds: 16 | 32, relation: -1,
        bonus: EquipmentSpec { stats: Stats { power: 0, toughness: 0 }, keywords: Keywords::NONE },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 1, first_layer: AbilityLayer::Type,
        fragments: &[StaticAbilityFragment::AttachedCharacteristics(&AttachedCharacteristics {
            name: None, form: PermanentForm::AddCreature, creature_types: &["Construct"], colors: &[], base: Some((5, 5)),
        })],
    }],
    activated: &[ActivatedAbility {
        exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Graveyard,
        cycling: false, targets: TargetPlan::NONE, once_per_turn: false, sorcery: false,
        cost: ActivationCost { colored: [0, 0, 0, 0, 1, 0], generic: 4, alternatives: &[],
            additional: None, mana_component: true, tap: false, sacrifice: false, exile_self: false,
            discard: false, life: 0, remove: None, loyalty: None },
        program: &[Instruction::ReturnSourceToHand(TargetZone::Graveyard)],
    }],
    extras: &CardExtras { aura_artifact_or_non_aura_enchantment: true, ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
