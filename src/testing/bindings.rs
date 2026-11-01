//! Literal Oracle contracts for repeated, uncoupled spell configurations.
//! This module is nested in cards so it can reject every unaccounted wrapper.
use super::*;
use serde::Deserialize;

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SpellTarget {
    None,
    Any,
    Creature,
    Spell,
    CreatureSpell,
    NoncreatureSpell,
    Player,
    Opponent,
}
impl SpellTarget {
    fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Any => "any",
            Self::Creature => "creature",
            Self::Spell => "spell",
            Self::CreatureSpell => "creature_spell",
            Self::NoncreatureSpell => "noncreature_spell",
            Self::Player => "player",
            Self::Opponent => "opponent",
        }
    }
    fn plan(self) -> TargetPlan {
        let rule = match self {
            Self::None => return TargetPlan::NONE,
            Self::Any => TargetRule::Any,
            Self::Creature => TargetRule::Creature,
            Self::Spell => TargetRule::Spell,
            Self::CreatureSpell => TargetRule::SpellKinds {
                kinds: 1,
                excluded: 0,
            },
            Self::NoncreatureSpell => TargetRule::SpellKinds {
                kinds: 0,
                excluded: 1,
            },
            Self::Player => TargetRule::Player {
                opponent_only: false,
            },
            Self::Opponent => TargetRule::Player {
                opponent_only: true,
            },
        };
        TargetPlan::Single(TargetSpec {
            rule,
            min: 1,
            max: 1,
        })
    }
}

#[derive(Clone, Copy, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum SpellFamily {
    Draw { amount: u16 },
    GainLife { amount: u16 },
    Damage { amount: u16, target: SpellTarget },
    DestroyCreature,
    ReturnCreature,
    GrowCreature { power: i32, toughness: i32 },
    Counter { target: SpellTarget },
    DrawLose { cards: u16, life: u16 },
    DrawDiscard { cards: u16, discard: u16 },
}
impl SpellFamily {
    pub(crate) fn signature(self) -> String {
        let detail = match self {
            Self::Draw { amount } => format!("draw/controller/{amount}"),
            Self::GainLife { amount } => format!("gain_life/controller/{amount}"),
            Self::Damage { amount, target } => format!("damage/{}/{amount}", target.label()),
            Self::DestroyCreature => "destroy/creature".into(),
            Self::ReturnCreature => "return_hand/creature".into(),
            Self::GrowCreature { power, toughness } => {
                format!("grow/creature/{power}/{toughness}/end_turn")
            }
            Self::Counter { target } => format!("counter/{}", target.label()),
            Self::DrawLose { cards, life } => format!("draw_lose/controller/{cards}/{life}"),
            Self::DrawDiscard { cards, discard } => {
                format!("draw_discard/controller/{cards}/{discard}")
            }
        };
        format!("plain_normal_spell_v1/{detail}")
    }
    pub(crate) fn target(self) -> SpellTarget {
        match self {
            Self::Damage { target, .. } | Self::Counter { target } => target,
            Self::DestroyCreature | Self::ReturnCreature | Self::GrowCreature { .. } => {
                SpellTarget::Creature
            }
            _ => SpellTarget::None,
        }
    }
    fn matches_program(self, program: &[Instruction]) -> bool {
        match self {
            Self::Draw { amount } => program == [Instruction::Draw(PlayerSet::Controller, amount)],
            Self::GainLife { amount } => {
                program == [Instruction::GainLife(PlayerSet::Controller, amount)]
            }
            Self::Damage { amount, .. } => program == [Instruction::Damage(i32::from(amount))],
            Self::DestroyCreature => program == [Instruction::Destroy],
            Self::ReturnCreature => program == [Instruction::ReturnToHand],
            Self::GrowCreature { power, toughness } => {
                program
                    == [Instruction::Grow {
                        subject: Subject::Targets,
                        power,
                        toughness,
                    }]
            }
            Self::Counter { .. } => program == [Instruction::Counter],
            Self::DrawLose { cards, life } => {
                program
                    == [
                        Instruction::Draw(PlayerSet::Controller, cards),
                        Instruction::LoseLife(PlayerSet::Controller, life),
                    ]
            }
            Self::DrawDiscard { cards, discard } => {
                program
                    == [
                        Instruction::Draw(PlayerSet::Controller, cards),
                        Instruction::Discard(PlayerSet::Controller, discard),
                    ]
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Cost {
    pub printed: String,
    pub colored: Mana,
    pub generic: u32,
}
impl Cost {
    pub(crate) fn pool(&self) -> Mana {
        let mut pool = self.colored;
        pool[5] = pool[5]
            .checked_add(self.generic)
            .expect("bounded printed cost");
        pool
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Member {
    pub card: String,
    pub catalogue_id: u16,
    pub oracle_id: Option<String>,
    pub type_line: String,
    pub cost: Cost,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Family {
    pub id: String,
    pub effect: SpellFamily,
    pub representative: String,
    pub members: Vec<Member>,
}
pub(crate) fn families() -> Vec<Family> {
    crate::engine::testing::fixture_json("oracle_families.json")
}

fn bind(member: &Member, effect: SpellFamily) {
    let card = Card::from_name(&member.card).expect("known Oracle identity");
    assert_eq!(card.index(), usize::from(member.catalogue_id));
    assert!(card.is_supported());
    assert_eq!(card.info().oracle_id, member.oracle_id.as_deref());
    assert_eq!(card.info().type_line, member.type_line);
    assert_eq!(card.info().mana_cost, member.cost.printed);
    assert!(!card.info().legendary);
    assert_eq!(card.cost(), (member.cost.colored, member.cost.generic));
    let definition = &generated::DEFINITIONS[card.index()];
    let expected_kind = match member.type_line.as_str() {
        "Sorcery" => 4,
        "Instant" => 8,
        other => panic!("not a plain spell: {other}"),
    };
    assert_eq!(definition.kind, expected_kind, "Oracle spell timing kind");
    assert!(!definition.basic && definition.mana_color.is_none());
    let rules = authored::rules(card).unwrap();
    // Exhaustive destructuring makes new fields require an eligibility audit.
    assert!(
        matches!(
            rules,
            authored::CardRules {
                deck_limit: Some(4),
                extras: _,
                effect: Effect::Spell(_, _),
                triggers: [],
                activated: [],
                keywords: Keywords::NONE,
                enters: InitialCounters {
                    plus_one: 0,
                    minus_one: 0,
                    stun: 0,
                    loyalty: 0,
                    lore: 0
                },
                entry: EntryTapped::Never,
                mode_costs: None,
                aura: None,
                static_groups: [],
            }
        ),
        "{} is not a plain spell; use a direct outcome case",
        member.card
    );
    assert!(
        plain_extras(rules.extras, Keywords::NONE, None),
        "{} has an unaccounted wrapper; use a direct outcome case",
        member.card
    );
    assert_eq!(card.effect(), rules.effect);
    assert!(card.triggers().is_empty() && card.activations().is_empty());
    assert!(card.replacement_effects().is_empty() && card.cost_reductions().is_empty());
    let (targets, program) = card
        .spell_program(None)
        .expect("engine-consumed spell program");
    assert_eq!(
        targets,
        effect.target().plan(),
        "{} target contract",
        member.card
    );
    assert!(
        effect.matches_program(program),
        "{} complete program contract",
        member.card
    );
}

#[test]
fn oracle_families_bind_the_engine_programs() {
    let groups = families();
    assert!(!groups.is_empty());
    let mut members = std::collections::BTreeSet::new();
    for family in groups {
        assert_eq!(family.id, family.effect.signature());
        assert!(
            family
                .members
                .iter()
                .any(|m| m.card == family.representative)
        );
        for member in family.members {
            println!("binding {}: {}", family.id, member.card);
            assert!(
                members.insert(member.catalogue_id),
                "a plain spell needs one exact family"
            );
            bind(&member, family.effect);
            crate::engine::testing::covered(&format!(
                "card/{}/binding/{}",
                member.catalogue_id, family.id
            ));
        }
    }
}

pub(super) fn plain_extras(
    extras: &authored::CardExtras,
    keywords: Keywords,
    legacy_mana: Option<u16>,
) -> bool {
    if extras.legacy_subtype_mana_activation != legacy_mana {
        return false;
    }
    if !keyword_groups(extras.static_ability_groups, keywords) {
        return false;
    }
    matches!(
        extras,
        authored::CardExtras {
            transforming_double_face: false,
            targets_up_to_x: false,
            life_x_cost: false,
            spell_cannot_be_copied: false,
            cast_graveyard_minimum: 0,
            cast_after_first_three_turns: false,
            prepared_spell: None,
            enters_prepared: false,
            prepared_identity: None,
            borrows_exiled_activations: false,
            self_granted_activation_templates: [],
            named_activation_lock: false,
            zone_replacement: None,
            opening_hand_battlefield: false,
            power_up_abilities: [],
            power_up_additional_uses: 0,
            power_up_other_creature_reduction: 0,
            power_up_mana_cost_alternatives: [],
            activation_target_reductions: [],
            activation_target_counter_discounts: [],
            activation_counter_discounts: [],
            activation_subtype_discounts: [],
            activation_put_counters: [],
            fixed_entry_counters: [],
            battle_defense: 0,
            as_turned_face_up_counters: [],
            entry_choice: None,
            chosen_entry_counters: None,
            static_modifiers: [],
            equipment: None,
            aura: None,
            static_ability_groups: _,
            live_static_effects: [],
            tiered_costs: None,
            base_stat_setter: None,
            combat_restriction: None,
            role: None,
            plot_cost: None,
            adventure_identity: None,
            adventure_spell: None,
            adventure_parent: None,
            omen_spell: None,
            omen_parent: None,
            warp_cost: None,
            disguise_cost: None,
            sneak_cost: None,
            harmonize_cost: None,
            mayhem_cost: None,
            vehicle_stats: None,
            replacement_effects: [],
            attached_subtype: None,
            attached_turn_keywords: Keywords::NONE,
            dynamic_stats: None,
            scaling_modifiers: [],
            entry_trigger_suppression: 0,
            controlled_entry_untapped: 0,
            opponent_entry_tapped: 0,
            keyword_grants: [],
            additional_cost: None,
            graveyard_counter_cost: None,
            collect_evidence: None,
            bargain_reduction: None,
            x_symbols: 0,
            x_entry_counters: false,
            x_complement_entry_counters: None,
            teamwork: None,
            blight: None,
            kicker_cost: None,
            gift: None,
            kicked_entry_counters: 0,
            counter_keyword_grants: [],
            offspring_cost: None,
            offspring_copy: None,
            flashback_cost: None,
            flashback_cost_alternatives: [],
            flashback_additional: None,
            target_cost_reduction: None,
            cast_timing_grants: [],
            cast_timing_requires_opponent_spell: false,
            self_cast_timing: None,
            cast_timing_guard: StepGuard::Always,
            cost_reductions: [],
            cost_alternatives: [],
            cost_alternative_guard: None,
            mana_alternative_replaces_additional: false,
            additional_mana_alternative: None,
            legacy_subtype_mana_activation: _,
            kicked_targets: None,
            cast_entry_counters: [],
            hand_cast_entry_counters: [],
            entry_counter_rules: [],
            starting_loyalty: 0,
            exhaust_mask: 0,
            has_convoke: false,
            has_improvise: false,
            static_cast_restriction: 0,
            uncounterable: false,
            aura_artifact_or_non_aura_enchantment: false,
            enchanted_modifiers: [],
            flash_surcharge: None,
        }
    )
}

// Multiple legacy projections may carry the same intrinsic keyword groups.
pub(super) fn keyword_groups(groups: &[StaticAbilityGroup], keywords: Keywords) -> bool {
    groups.iter().all(|group| group.first_layer == AbilityLayer::Ability && group.fragments.iter().all(|fragment| matches!(fragment, StaticAbilityFragment::IntrinsicKeywords(flags) if keywords.contains(*flags))))
}
