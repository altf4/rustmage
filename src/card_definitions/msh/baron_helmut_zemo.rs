// Direct typed behavior from the full frozen Oracle and pinned Java.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CastFromHand(&TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Colors {
                filter: &CastFilter::Any,
                mask: 4,
            },
            ordinal: None,
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::Connive {
            subject: Subject::Source,
            amount: 1,
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::SourceAttackedThisTurn,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 0,
            alternatives: &[],
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::ExileGraveyardSymbols {
                    color: ChoiceColor::Black,
                    minimum: 15,
                },
                filter: SacrificeFilter {
                    artifact_enchantment_or_token: false,
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    subtype: None,
                },
            }),
            mana_component: false,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyPaidExiledCardsFree { maximum_casts: 3 }],
        sorcery: false,
        once_per_turn: true,
    }],
    ..CardRules::EMPTY
};
