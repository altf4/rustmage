//! Grouped reveal costs retain only their paid type/exile outcomes on the spell.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::funding::FundingState;
use rustmage_core::engine::*;
use std::collections::HashSet;
use support::*;
const CREATURES: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 1,
    nonland: false,
    other: true,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const SEARCH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SearchCreatureManaValue {
            maximum: AmountValue::ChosenX,
            destination: SearchDestination::BeheldCreatureType,
        }],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::OptionalBeholdType(2),
            filter: CREATURES,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const EXILE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfLeaves,
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnLinkedExileToHand],
    }],
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::BeholdAndExile,
            filter: SacrificeFilter {
                subtype: Some("Elf"),
                ..CREATURES
            },
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const FLASHBACK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Draw(PlayerSet::Controller, 2)],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 1,
        }),
        flashback_additional: Some(AdditionalCost {
            kind: EffectChoiceKind::BeholdMany(3),
            filter: CREATURES,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
fn finish_cast(g: &mut Game) {
    for _ in 0..32 {
        if g.state.pending == Pending::Priority {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::PayCosts { .. }
                    | Action::BeginOrderedCosts
                    | Action::ChooseCastCost { .. }
                    | Action::ChooseAdditionalCost { .. }
            )
        });
    }
    panic!("behold payment bound");
}
fn wait_search(g: &mut Game) {
    for _ in 0..20 {
        if matches!(g.state.pending, Pending::EffectSearch { .. }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("private search not reached");
}
#[test]
fn optional_group_reveals_only_on_payment_and_paid_type_survives_witness_departure() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &SEARCH)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let witness = put(&mut g, p, "Grizzly Bears", Zone::Hand);
        let permanent = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let found = put(&mut g, p, "Bear Cub", Zone::Library);
        let source = put(&mut g, p, "Divination", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: source });
        act(
            &mut g,
            Action::ChooseBeholdType {
                creature_type: Some(CreatureType::from_name("Bear").unwrap()),
            },
        );
        assert!(
            !g.observe(p.other())
                .known_hand
                .contains(&g.object_ref(witness))
        );
        act(&mut g, Action::ChooseX { value: 2 });
        finish_cast(&mut g);
        assert!(
            g.observe(p.other())
                .known_hand
                .contains(&g.object_ref(witness))
        );
        g.move_object(permanent, Zone::Graveyard);
        g.move_object(witness, Zone::Graveyard);
        wait_search(&mut g);
        assert!(g.observe(p.other()).search_choice.is_none());
        act(&mut g, Action::ChooseSearch { card: found });
        assert_eq!(g.state.objects[found as usize].zone, Zone::Library);
        act(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.state.objects[found as usize].zone, Zone::Battlefield);
    }
}
#[test]
fn optional_behold_decline_searches_to_hand_and_private_failure_is_allowed() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &SEARCH)]);
    for fail in [false, true] {
        let mut g = scenario(Player::P0);
        let found = put(&mut g, Player::P0, "Grizzly Bears", Zone::Library);
        let source = put(&mut g, Player::P0, "Divination", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: source });
        reject(
            &mut g,
            Action::ChooseBeholdType {
                creature_type: Some(CreatureType::from_name("Bear").unwrap()),
            },
        );
        act(
            &mut g,
            Action::ChooseBeholdType {
                creature_type: None,
            },
        );
        act(&mut g, Action::ChooseX { value: 2 });
        finish_cast(&mut g);
        wait_search(&mut g);
        if !fail {
            act(&mut g, Action::ChooseSearch { card: found });
        }
        act(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(
            g.state.objects[found as usize].zone,
            if fail { Zone::Library } else { Zone::Hand }
        );
    }
}
#[test]
fn behold_exile_links_the_entered_permanent_and_a_countered_spell_keeps_its_cost_exiled() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &EXILE)]);
    for counter in [false, true] {
        let mut g = scenario(Player::P1);
        let elf = put(&mut g, Player::P1, "Llanowar Elves", Zone::Hand);
        let source = put(&mut g, Player::P1, "Grizzly Bears", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: source });
        finish_cast(&mut g);
        assert_eq!(g.state.objects[elf as usize].zone, Zone::Exile);
        if counter {
            assert_eq!(g.state.stack.len(), 1, "{:#?}", g.state.stack);
            assert_eq!(g.state.stack[0].kind, StackKind::Spell);
            let negate = put(&mut g, Player::P0, "Cancel", Zone::Hand);
            act(&mut g, Action::Pass);
            cast(&mut g, negate, Some(Target::Spell(source)));
            settle(&mut g);
            assert_eq!(g.state.objects[elf as usize].zone, Zone::Exile);
        } else {
            settle(&mut g);
            assert_eq!(
                g.observe(Player::P1).linked_exile[0].source,
                g.object_ref(source)
            );
            g.move_object(source, Zone::Graveyard);
            g.prepare_triggers();
            settle(&mut g);
            assert_eq!(g.state.objects[elf as usize].zone, Zone::Hand);
        }
    }
}
#[test]
fn flashback_beholds_three_distinct_hand_or_battlefield_creatures_without_tapping_them() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &FLASHBACK)]);
    let mut g = scenario(Player::P0);
    let a = put(&mut g, Player::P0, "Llanowar Elves", Zone::Hand);
    let b = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let source = put(&mut g, Player::P0, "Divination", Zone::Graveyard);
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceFlashback { card: source })
    );
    let c = put(&mut g, Player::P0, "Fog Bank", Zone::Battlefield);
    act(&mut g, Action::AnnounceFlashback { card: source });
    finish_cast(&mut g);
    settle(&mut g);
    assert_eq!(g.state.objects[source as usize].zone, Zone::Exile);
    assert_eq!(g.state.objects[a as usize].zone, Zone::Hand);
    assert!(!g.state.objects[b as usize].tapped && !g.state.objects[c as usize].tapped);
    assert_eq!(g.hand_count(Player::P0), 3);
}

fn behold_funding_probe(
    g: &Game,
    p: Player,
    source: CardId,
    kind: Option<CreatureType>,
) -> (bool, bool, usize) {
    let origin = g.state.objects[source as usize].zone;
    let mut item = g.cast_preview(
        p,
        source,
        origin,
        g.object_ref(source),
        g.state.next_stack_id,
        [0; 6],
    );
    if origin == Zone::Graveyard {
        item.cast.as_mut().unwrap().method = CastMethod::Flashback;
    }
    let mut a = g.payment_announcement(item, Counters::default(), None, None);
    a.item.x = Some(0);
    let cast = a.item.cast.as_mut().unwrap();
    cast.behold_type_declared = true;
    cast.beheld_type = kind;
    let cost = a.cost_options().next();
    a.cost = cost;
    a.cast_costs = Some(Box::default());
    a.initialize_cast_costs(false);
    let goal = g.funding_goal(&a);
    let mut world = FundingState::from_game(g, &goal);
    world.set_purpose(g.purpose_for_item(&a.item));
    world.set_payment_context(&a);
    let deficit = world.additional_cost_deficit();
    let mut visited = HashSet::new();
    let payable = world.reachable(&goal, &mut visited);
    (deficit, payable, visited.len())
}

#[test]
fn fixed_behold_type_domain_rejects_absent_witnesses_before_dense_mana_search() {
    let _rules = fixture_rules::activate(&[(Card::GiantGrowth, &SEARCH)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let lands: Vec<_> = (0..14)
            .map(|_| put(&mut g, p, "Forest", Zone::Battlefield))
            .collect();
        let hand = put(&mut g, p, "Grizzly Bears", Zone::Hand);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        put(&mut g, p, "Llanowar Elves", Zone::Hand);
        put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        put(&mut g, p, "Llanowar Elves", Zone::Library);
        let source = put(&mut g, p, "Giant Growth", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        let bear = CreatureType::from_name("Bear").unwrap();
        for with_color_aura in [false, true] {
            if with_color_aura {
                let enchanted = put(&mut g, p.other(), "Forest", Zone::Battlefield);
                let aura = put(&mut g, p.other(), "Shimmerwilds Growth", Zone::Battlefield);
                g.state.objects[aura as usize].attached_to = Some(g.object_ref(enchanted));
                let aura = g.object_ref(aura);
                g.set_entry_value(aura, EntryChoiceValue::Color(ChoiceColor::Blue));
                assert!(
                    g.state
                        .roles
                        .as_ref()
                        .unwrap()
                        .characteristic_sources
                        .contains(&aura)
                );
                assert_eq!(g.object_view(enchanted).colors, 2);
            }
            for name in CREATURE_TYPES {
                let kind = CreatureType::from_name(name).unwrap();
                let (deficit, payable, visited) = behold_funding_probe(&g, p, source, Some(kind));
                if kind == bear {
                    assert!(!deficit && payable);
                } else {
                    assert_eq!(
                        (deficit, payable, visited),
                        (true, false, 0),
                        "{name}; color Aura {with_color_aura}"
                    );
                }
            }
        }
        // Keep the real cast focused on the witness and reveal rules after
        // the dense probes have exercised every type with all lands ready.
        for &land in &lands[1..] {
            g.state.objects[land as usize].tapped = true;
        }
        act(&mut g, Action::AnnounceCast { card: source });
        let chosen = Action::ChooseBeholdType {
            creature_type: Some(bear),
        };
        let actions = g.legal_actions();
        assert_eq!(
            actions
                .iter()
                .filter(|action| matches!(action, Action::ChooseBeholdType { .. }))
                .count(),
            2
        );
        assert!(actions.contains(&Action::ChooseBeholdType {
            creature_type: None,
        }));
        assert!(actions.contains(&chosen));
        assert!(
            !g.observe(p.other())
                .known_hand
                .contains(&g.object_ref(hand))
        );
        let saved = g.snapshot();
        let mut indexed = g.clone();
        let index = actions.iter().position(|a| *a == chosen).unwrap();
        indexed.apply_index(p, indexed.revision(), index).unwrap();
        act(&mut g, chosen.clone());
        assert_eq!(g.snapshot(), indexed.snapshot());
        act(&mut g, Action::ChooseX { value: 0 });
        act(&mut g, Action::TapMana { card: lands[0] });
        finish_cast(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Stack);
        assert!(
            g.observe(p.other())
                .known_hand
                .contains(&g.object_ref(hand))
        );
        let paid = g.snapshot();
        g.restore(&saved).unwrap();
        act(&mut g, chosen);
        act(&mut g, Action::ChooseX { value: 0 });
        act(&mut g, Action::TapMana { card: lands[0] });
        finish_cast(&mut g);
        assert_eq!(g.snapshot(), paid);
    }
}

#[test]
fn fixed_single_behold_and_exile_deficits_preserve_live_paid_witnesses() {
    const SINGLE: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::Draw(PlayerSet::Controller, 1)],
        ),
        extras: &CardExtras {
            additional_cost: Some(AdditionalCost {
                kind: EffectChoiceKind::Behold,
                filter: SacrificeFilter {
                    subtype: Some("Elf"),
                    ..CREATURES
                },
            }),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _rules =
        fixture_rules::activate(&[(Card::GrizzlyBears, &EXILE), (Card::Divination, &SINGLE)]);
    for p in [Player::P0, Player::P1] {
        for exile in [false, true] {
            let mut g = scenario(p);
            let lands: Vec<_> = (0..12)
                .map(|i| {
                    put(
                        &mut g,
                        p,
                        if i % 2 == 0 { "Forest" } else { "Island" },
                        Zone::Battlefield,
                    )
                })
                .collect();
            put(&mut g, p, "Bear Cub", Zone::Battlefield);
            put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
            let source = put(
                &mut g,
                p,
                if exile { "Grizzly Bears" } else { "Divination" },
                Zone::Hand,
            );
            g.state.players[p.index()].mana = [0; 6];
            assert_eq!(behold_funding_probe(&g, p, source, None), (true, false, 0));
            reject(&mut g, Action::AnnounceCast { card: source });
            let witness = put(&mut g, p, "Llanowar Elves", Zone::Hand);
            let (deficit, payable, _) = behold_funding_probe(&g, p, source, None);
            assert!(!deficit && payable);
            act(&mut g, Action::AnnounceCast { card: source });
            // Ordinary single behold may request its witness before the mana
            // window; the exile group may instead order its declared clauses.
            if matches!(g.state.pending, Pending::AdditionalCost { .. }) {
                let object = g.object_ref(witness);
                act(&mut g, Action::ChooseAdditionalCost { object });
            }
            for &land in &lands[..if exile { 2 } else { 3 }] {
                act(&mut g, Action::TapMana { card: land });
            }
            finish_cast(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Stack);
            assert_eq!(
                g.object_view(witness).zone,
                if exile { Zone::Exile } else { Zone::Hand }
            );
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn behold_bound_allows_mana_tapping_and_current_changeling_type_and_control() {
    const CHANGELING: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        keywords: Keywords::CHANGELING,
        ..CardRules::EMPTY
    };
    const ARTIFACT: CardRules = CardRules {
        effect: Effect::Permanent,
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::GiantGrowth, &SEARCH),
        (Card::GrizzlyBears, &CHANGELING),
        (Card::from_name("Short Bow").unwrap(), &ARTIFACT),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let first = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let second = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let source = put(&mut g, p, "Giant Growth", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        let elf = CreatureType::from_name("Elf").unwrap();
        act(&mut g, Action::AnnounceCast { card: source });
        act(
            &mut g,
            Action::ChooseBeholdType {
                creature_type: Some(elf),
            },
        );
        act(&mut g, Action::ChooseX { value: 1 });
        for card in [first, second] {
            act(&mut g, Action::TapMana { card });
        }
        finish_cast(&mut g);
        assert!(g.object_view(first).tapped && g.object_view(second).tapped);
        assert_eq!(g.object_view(source).zone, Zone::Stack);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);

        let mut g = scenario(p);
        let changed = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        put(&mut g, p, "Grizzly Bears", Zone::Hand);
        let source = put(&mut g, p, "Giant Growth", Zone::Hand);
        let dragon = CreatureType::from_name("Dragon").unwrap();
        let (deficit, payable, _) = behold_funding_probe(&g, p, source, Some(dragon));
        assert!(!deficit && payable);
        g.change_controller(g.object_ref(changed), p.other());
        assert!(!behold_funding_probe(&g, p, source, Some(dragon)).1);
        g.change_controller(g.object_ref(changed), p);
        assert!(behold_funding_probe(&g, p, source, Some(dragon)).1);

        let mut g = scenario(p);
        let changed = put(&mut g, p, "Short Bow", Zone::Battlefield);
        put(&mut g, p, "Llanowar Elves", Zone::Hand);
        let source = put(&mut g, p, "Giant Growth", Zone::Hand);
        assert!(behold_funding_probe(&g, p, source, Some(elf)).0);
        g.change_object_form(
            g.object_ref(changed),
            AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Named(&["Elf"]),
                base: Some((1, 1)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::Permanent,
            },
            characteristic_layers::AnimationTypeMode::AddCreature,
        );
        assert!(behold_funding_probe(&g, p, source, Some(elf)).1);
        act(&mut g, Action::AnnounceCast { card: source });
        act(
            &mut g,
            Action::ChooseBeholdType {
                creature_type: Some(elf),
            },
        );
        act(&mut g, Action::ChooseX { value: 0 });
        finish_cast(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Stack);
    }
}

#[test]
fn behold_witness_deficit_falls_back_for_a_mana_source_linked_return() {
    const MANA: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                tap: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 1, 0,
            ]))],
            sorcery: false,
            once_per_turn: false,
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::GiantGrowth, &SEARCH),
        (Card::from_name("Short Bow").unwrap(), &MANA),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let mana = put(&mut g, p, "Short Bow", Zone::Battlefield);
        let returning = put(&mut g, p, "Llanowar Elves", Zone::Exile);
        let original = g.object_ref(returning);
        let hand = put(&mut g, p, "Llanowar Elves", Zone::Hand);
        let source = put(&mut g, p, "Giant Growth", Zone::Hand);
        g.state.exile_links.push(ExileLink {
            ability_origin: None,
            source: g.object_ref(mana),
            exiled: original,
            return_zone: Zone::Battlefield,
        });
        g.state.players[p.index()].mana = [0; 6];
        let elf = CreatureType::from_name("Elf").unwrap();
        let (deficit, payable, _) = behold_funding_probe(&g, p, source, Some(elf));
        assert!(
            !deficit && payable,
            "the missing second Elf enters during mana payment"
        );
        act(&mut g, Action::AnnounceCast { card: source });
        act(
            &mut g,
            Action::ChooseBeholdType {
                creature_type: Some(elf),
            },
        );
        act(&mut g, Action::ChooseX { value: 0 });
        act(
            &mut g,
            Action::ActivateMana {
                card: mana,
                ability: 0,
                counter_payment: Counters::default(),
                payment: [0; 6],
                production: [0, 0, 0, 0, 1, 0],
            },
        );
        assert_eq!(g.object_view(mana).zone, Zone::Graveyard);
        assert_eq!(g.object_view(returning).zone, Zone::Battlefield);
        assert_ne!(g.object_ref(returning), original);
        finish_cast(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Stack);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(
            g.observe(p.other())
                .known_hand
                .contains(&g.object_ref(hand))
        );
    }
}

#[test]
fn damage_only_replacement_keeps_flashback_behold_deficits_bounded() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        for _ in 0..12 {
            put(&mut g, p, "Mountain", Zone::Battlefield);
        }
        let inferno = put(&mut g, p, "Collective Inferno", Zone::Battlefield);
        g.set_entry_value(
            g.object_ref(inferno),
            EntryChoiceValue::CreatureType(CreatureType::from_name("Elemental").unwrap()),
        );
        assert!(g.state.static_sources.contains(&inferno));
        assert!(
            g.state.objects[inferno as usize]
                .card
                .static_ability_groups()
                .iter()
                .flat_map(|group| group.fragments)
                .any(|fragment| matches!(
                    fragment,
                    StaticAbilityFragment::Replacement(
                        _,
                        ReplacementSpec::DoubleControlledChosenTypeDamage
                    )
                ))
        );
        let first = put(&mut g, p, "Flamebraider", Zone::Battlefield);
        let source = put(&mut g, p, "Kindle the Inner Flame", Zone::Graveyard);
        g.state.players[p.index()].mana = [0; 6];
        assert_eq!(behold_funding_probe(&g, p, source, None), (true, false, 0));
        reject(&mut g, Action::AnnounceFlashback { card: source });
        let second = put(&mut g, p, "Flamebraider", Zone::Battlefield);
        assert_eq!(behold_funding_probe(&g, p, source, None), (true, false, 0));
        reject(&mut g, Action::AnnounceFlashback { card: source });
        let third = put(&mut g, p, "Flamebraider", Zone::Hand);
        let (deficit, payable, _) = behold_funding_probe(&g, p, source, None);
        assert!(!deficit && payable);
        act(&mut g, Action::AnnounceFlashback { card: source });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(first),
            },
        );
        act(
            &mut g,
            Action::ActivateMana {
                card: first,
                ability: 0,
                counter_payment: Counters::default(),
                payment: [0; 6],
                production: [0, 0, 0, 2, 0, 0],
            },
        );
        act(&mut g, Action::BeginOrderedCosts);
        let clause = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses
            .iter()
            .position(|row| !row.paid && row.clause == CastCostClause::Additional)
            .unwrap() as u16;
        act(&mut g, Action::ChooseCastCost { clause });
        let first_ref = g.object_ref(first);
        act(&mut g, Action::ChooseAdditionalCost { object: first_ref });
        reject(&mut g, Action::ChooseAdditionalCost { object: first_ref });
        for witness in [second, third] {
            let object = g.object_ref(witness);
            act(&mut g, Action::ChooseAdditionalCost { object });
        }
        finish_cast(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Stack);
        assert!(g.object_view(first).tapped);
        assert!(!g.object_view(second).tapped);
        assert_eq!(g.object_view(third).zone, Zone::Hand);
        assert!(
            g.observe(p.other())
                .known_hand
                .contains(&g.object_ref(third))
        );
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
    }
}
