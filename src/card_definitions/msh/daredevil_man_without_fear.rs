// Complete typed Oracle behavior; pinned direct Java and helper audit in provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::HASTE.union(Keywords::VIGILANCE),
    static_groups: &[StaticAbilityGroup {
        line: 0, first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::LookAtLibraryTop],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttackDeclared {
            minimum: 1, power_min: None, guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::May { skip: 1 }, Instruction::ExileTopPlayBoostSourceIfSubtype {
            subtype: "Hero", power: 2, toughness: 1,
        }],
    }],
    ..CardRules::EMPTY
};
