//! Generic boundaries exposed by the full ECL candidate scenarios.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

macro_rules! form_aura {
    ($form:expr) => {
        CardRules {
            effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Creature))),
            aura: Some(AuraSpec {
                kinds: 1,
                relation: -1,
                bonus: EquipmentSpec {
                    stats: Stats {
                        power: 0,
                        toughness: 0,
                    },
                    keywords: Keywords::NONE,
                },
            }),
            static_groups: &[StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Type,
                fragments: &[StaticAbilityFragment::AttachedCharacteristics(
                    &AttachedCharacteristics {
                        name: None,
                        form: $form,
                        creature_types: &["Noggle"],
                        colors: &[],
                        base: Some((1, 1)),
                    },
                )],
            }],
            ..CardRules::EMPTY
        }
    };
}
const PRESERVE: CardRules = form_aura!(PermanentForm::Preserve);
const COLORLESS: CardRules = form_aura!(PermanentForm::PreserveWithColors);
const LIBRARY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Union(&[
            TargetRule::Spell,
            TargetRule::Creature,
        ]))),
        &[Instruction::OwnerLibrary],
    ),
    ..CardRules::EMPTY
};
const LINKED: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfLeaves,
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnLinkedExileToHand],
    }],
    ..CardRules::EMPTY
};
const BARRIER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CounterProhibition(1)],
    }],
    ..CardRules::EMPTY
};
macro_rules! counter_replacement {
    ($factor:expr, $add:expr) => {
        CardRules {
            effect: Effect::Permanent,
            static_groups: &[StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Rule,
                fragments: &[StaticAbilityFragment::Replacement(
                    0,
                    ReplacementSpec::Counters {
                        filter: ReplacementFilter::Permanent,
                        kind: None,
                        factor: $factor,
                        add: $add,
                        effects_only: false,
                    },
                )],
            }],
            extras: &CardExtras {
                replacement_effects: &[ReplacementSpec::Counters {
                    filter: ReplacementFilter::Permanent,
                    kind: None,
                    factor: $factor,
                    add: $add,
                    effects_only: false,
                }],
                ..CardExtras::EMPTY
            },
            ..CardRules::EMPTY
        }
    };
}
const DOUBLE: CardRules = counter_replacement!(2, 0);
const ADD: CardRules = counter_replacement!(1, 1);
fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn owned(g: &mut Game, action: Action) {
    let actor = g.actor().unwrap();
    let before = g.snapshot();
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let mut child = g.clone();
    child.apply_index(actor, child.revision(), index).unwrap();
    g.apply(actor, action.clone()).unwrap();
    assert_eq!(g.snapshot(), child.snapshot());
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(viewer)).unwrap(),
            serde_json::to_value(child.observe(viewer)).unwrap()
        );
    }
    let after = g.snapshot();
    g.restore(&before).unwrap();
    act(g, action);
    assert_eq!(g.snapshot(), after);
}

#[test]
fn colorless_and_unchanged_colors_remain_distinct_while_preserving_card_types() {
    let _scope = fixture_rules::activate(&[
        (card("Pacifism"), &PRESERVE),
        (card("Witness Protection"), &COLORLESS),
    ]);
    for p in [Player::P0, Player::P1] {
        for (name, colorless) in [("Pacifism", false), ("Witness Protection", true)] {
            let mut g = scenario(p);
            let creature = put(&mut g, p.other(), "Juggernaut", Zone::Battlefield);
            g.set_color_effects(&[creature], 1, AnimationDuration::Permanent);
            let aura = put(&mut g, p, name, Zone::Hand);
            cast(&mut g, aura, Some(Target::Card(creature)));
            settle(&mut g);
            assert_eq!(g.current_kinds(creature) & (1 | 16), 1 | 16);
            assert!(g.has_subtype(creature, "Noggle"));
            assert_eq!(stats(&g, creature), (1, 1));
            assert_eq!(
                g.object_view(creature).colors,
                if colorless { 0 } else { 1 }
            );
            g.set_color_effects(&[creature], 2, AnimationDuration::Permanent);
            assert_eq!(g.object_view(creature).colors, 2, "later color change wins");
            g.move_object(aura, Zone::Graveyard);
            assert_eq!(g.object_view(creature).colors, 2);
            assert_eq!(g.current_kinds(creature) & (1 | 16), 1 | 16);
        }
    }
}

#[test]
fn copy_exception_changeling_needs_a_creature_or_kindred_card_type() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        let object = g.object_ref(land);
        let mut recipe = CopyRecipe::new(Card::Forest);
        recipe.overrides = Some(CopyOverrides {
            keywords: Keywords::CHANGELING,
            ..CopyOverrides::NONE
        });
        g.apply_temporary_copy(object, recipe, CopyDeadline::WhileSource { source: object });
        assert!(g.keywords(land).contains(Keywords::CHANGELING));
        assert!(g.has_subtype(land, "Forest"));
        assert!(!g.has_subtype(land, "Elf"));
        g.change_object_form(
            object,
            AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Unchanged,
                base: Some((2, 2)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            },
            characteristic_layers::AnimationTypeMode::AddCreature,
        );
        assert!(g.has_subtype(land, "Elf"));
        g.expire_animations();
        assert!(!g.has_subtype(land, "Elf"));
        assert!(g.has_subtype(land, "Forest"));
    }
}

#[test]
fn owner_library_moves_uncounterable_spells_and_copies_with_exact_owned_choices() {
    let _scope = fixture_rules::activate(&[(Card::Shock, &LIBRARY)]);
    for p in [Player::P0, Player::P1] {
        for copy in [false, true] {
            for destination in [LibraryDestination::Top, LibraryDestination::Bottom] {
                let mut g = scenario(p);
                let original = put(&mut g, p, "Sphinx of the Final Word", Zone::Hand);
                cast(&mut g, original, None);
                assert!(!g.counter_stack_item(g.state.stack.last().unwrap().id, false));
                let (target, bound) = if copy {
                    let template = g.state.stack.last().unwrap().clone();
                    let packet = g.allocate_spell_copy(&template, p);
                    let id = packet.source.unwrap().id;
                    let bound = Target::Stack(packet.id);
                    g.state.stack.push(packet);
                    (id, bound)
                } else {
                    (original, Target::Spell(original))
                };
                let library = g.state.players[p.index()].library.clone();
                let prior_top = g.object_ref(g.top_library_card(p).unwrap());
                g.remember_private_library_top(p, p, prior_top);
                act(&mut g, Action::Pass);
                let bounce = put(&mut g, p.other(), "Shock", Zone::Hand);
                cast(&mut g, bounce, Some(bound));
                act(&mut g, Action::Pass);
                act(&mut g, Action::Pass);
                assert!(matches!(g.state.pending, Pending::TargetLibrary { .. }));
                assert_eq!(g.actor(), Some(p));
                assert!(g.observe(p.other()).target_library.is_none());
                let object = g.object_ref(target);
                reject(
                    &mut g,
                    Action::ChooseLibraryDestination {
                        object,
                        destination: LibraryDestination::SecondFromTop,
                    },
                );
                owned(
                    &mut g,
                    Action::ChooseLibraryDestination {
                        object,
                        destination,
                    },
                );
                settle(&mut g);
                if copy || destination == LibraryDestination::Bottom {
                    assert_eq!(
                        g.observe(p).known_library_top[p.index()]
                            .as_ref()
                            .map(|top| top.object),
                        Some(prior_top)
                    );
                    assert!(g.observe(p.other()).known_library_top[p.index()].is_none());
                }
                if copy {
                    assert_eq!(g.object_view(target).zone, Zone::Outside);
                    assert_eq!(g.state.players[p.index()].library, library);
                    assert_eq!(g.object_view(original).zone, Zone::Battlefield);
                } else {
                    assert_eq!(g.object_view(original).zone, Zone::Library);
                    let cards = &g.state.players[p.index()].library;
                    assert_eq!(
                        if destination == LibraryDestination::Top {
                            cards.last()
                        } else {
                            cards.first()
                        },
                        Some(&original)
                    );
                }
            }
        }
    }
}

#[test]
fn linked_return_uses_the_departed_ability_acquisition_and_original_owner() {
    let _scope = fixture_rules::activate(&[(card("Swiftfoot Boots"), &LINKED)]);
    for p in [Player::P0, Player::P1] {
        for copied in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
            let old = put(&mut g, p.other(), "Grizzly Bears", Zone::Exile);
            let object = g.object_ref(source);
            let link = LinkedExileView {
                source: object,
                exiled: g.object_ref(old),
                ability_origin: None,
                exiled_turn: g.state.turn,
            };
            g.state
                .linked_choices
                .get_or_insert_default()
                .links
                .push(link);
            let fresh = if copied {
                g.apply_temporary_copy(
                    object,
                    CopyRecipe::new(card("Swiftfoot Boots")),
                    CopyDeadline::WhileSource { source: object },
                );
                let origin = g.copy_ability_origin(object);
                assert!(origin.is_some());
                let fresh = put(&mut g, p.other(), "Hill Giant", Zone::Exile);
                let link = LinkedExileView {
                    source: object,
                    exiled: g.object_ref(fresh),
                    ability_origin: origin,
                    exiled_turn: g.state.turn,
                };
                g.state
                    .linked_choices
                    .get_or_insert_default()
                    .links
                    .push(link);
                Some(fresh)
            } else {
                None
            };
            g.move_object(source, Zone::Graveyard);
            g.prepare_triggers();
            settle(&mut g);
            assert_eq!(
                g.object_view(old).zone,
                if copied { Zone::Exile } else { Zone::Hand }
            );
            if let Some(fresh) = fresh {
                assert_eq!(g.object_view(fresh).zone, Zone::Hand);
                assert_eq!(g.object_view(fresh).controller, p.other());
            }
        }
    }
}

#[test]
fn prohibited_counter_placement_skips_amount_replacements_but_wither_damage_is_dealt() {
    let _scope = fixture_rules::activate(&[
        (card("Ghost Vacuum"), &BARRIER),
        (card("Short Bow"), &DOUBLE),
        (card("Swiftfoot Boots"), &ADD),
    ]);
    for p in [Player::P0, Player::P1] {
        for wither in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let barrier = put(&mut g, p, "Ghost Vacuum", Zone::Battlefield);
            let double = put(&mut g, p, "Short Bow", Zone::Battlefield);
            put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
            if wither {
                let dealer = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
                g.grant_keyword_effects(
                    &[dealer],
                    g.object_ref(dealer),
                    Keywords::WITHER.union(Keywords::LIFELINK),
                );
                g.apply_damage_batch(&[DamageEvent {
                    source: g.damage_source(dealer),
                    target: Target::Card(target),
                    amount: 2,
                    combat: false,
                    preventable: true,
                }]);
            } else {
                g.effect_counter(
                    target,
                    CounterKind::PlusOne,
                    1,
                    replacements::ResumeMode::Pending,
                );
            }
            assert!(!g.advance_replacements(false));
            assert!(g.state.replacements.is_none());
            assert_eq!(g.counter_count(target, CounterKind::PlusOne), 0);
            assert_eq!(g.counter_count(target, CounterKind::MinusOne), 0);
            assert_eq!(g.object_view(target).damage, 0);
            assert_eq!(
                g.state.players[p.other().index()].life,
                if wither { 22 } else { 20 }
            );
            g.move_object(barrier, Zone::Graveyard);
            g.effect_counter(
                target,
                CounterKind::PlusOne,
                1,
                replacements::ResumeMode::Pending,
            );
            assert!(g.advance_replacements(false));
            let choice = g.observe(p).replacement_choice.unwrap();
            assert_eq!(choice.effects.len(), 2);
            assert_eq!(g.actor(), Some(p));
            let source = g.object_ref(double);
            owned(&mut g, Action::ChooseReplacement { source, ability: 0 });
            assert!(g.state.replacements.is_none());
            assert_eq!(g.counter_count(target, CounterKind::PlusOne), 3);
        }
    }
}
