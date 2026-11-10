// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::SelectLibraryTo(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 5,
                    count: LibrarySelectionCount::Fixed(1),
                    filter: LibrarySelectionFilter::Kinds(1),
                    optional: true,
                    reveal: false,
                    remainder: LibraryRemainder::BottomRandom,
                },
                look: LibraryLook::Fixed(5),
                destination: LibraryPickDestination::ExileCastThisTurn,
                public: false,
                maximum_mana_value: None,
            }),
            Instruction::ScheduleStepThisTurn {
                step: TriggerStep::BeginCombat,
                relation: CastRelation::Any,
                ability: 0,
            },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::creature_you_control()),
            TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: 1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: 1,
            },
        ]),
        program: &[Instruction::GroupsPowerDamage {
            sources: &[0],
            recipient: 1,
        }],
    }],
    ..CardRules::EMPTY
};
