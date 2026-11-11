const MODES: &[SpellMode] = &[
    SpellMode {
        text: "You may put a Bird creature card from your hand or graveyard onto the battlefield with a finality counter.",
        targets: TargetPlan::NONE,
        program: &[Instruction::MayDeployFromHandOrGraveyard {
            filter: SearchFilter::Subtype("Bird"),
            counter: CounterKind::Finality,
        }],
    },
    SpellMode {
        text: "Put a +1/+1 counter on each Bird you control.",
        targets: TargetPlan::NONE,
        program: &[Instruction::PutFilteredCounters {
            filter: CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 0,
                exclude: 0,
                subtype: Some("Bird"),
                basic: false,
                other: false,
                cards_only: false,
            },
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    },
    SpellMode {
        text: "Draw a card.",
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    },
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
    keywords: Keywords::FLYING,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::ControlledSubtypes(&["Bird"]),
            combat_only: true,
            opponent_only: false,
            one_or_more: true,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(MODES)],
    }],
    ..CardRules::EMPTY
};
