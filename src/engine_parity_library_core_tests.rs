//! Bounded shared-core continuations; both prospective printed consumers stay gated.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;

fn bounded_program(game: &mut Game, item: SpellView, program: &'static [Instruction]) {
    // Owned continuation fixture for a private opcode whose producers remain
    // gated. Production dispatch, target-group selection and expiry are real.
    game.state.resolving = Some(programs::Resolution {
        cascade: None,
        target_card_facts: None,
        immediate_blink_return: None,
        explore: None,
        evidence: None,
        connive: None,
        item,
        program,
        pc: 0,
        target_group: 0,
        moved_count: None,
        discard_after: None,
        discards: [0; 2],
        mana_payment: None,
        forage: None,
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        milled_selection: None,
        library: None,
        selection: None,
        target_library: None,
        search: None,
        optional_searches: None,
        choice: None,
    });
    game.run_program();
}

// Explicit owned objects for the two still-gated typed core consumers.
// Normal supported fixtures keep their existing refusal of gated cards.
fn gated(g: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap_or_else(|| panic!("unknown fixture {name}"));
    assert!(
        !card.is_supported(),
        "core-only fixture must remain gated: {name}"
    );
    assert!(matches!(
        name,
        "Thanos, the Mad Titan" | "Trickster's Stratagem"
    ));
    let id = g.state.objects.len() as CardId;
    g.state.objects.push(Object {
        copy_kind: CopyKind::None,
        card,
        owner,
        controller: owner,
        zone,
        generation: 0,
        tapped: false,
        summoning_sick: false,
        flashback: false,
        damage: 0,
        damaged_this_turn: false,
        dealt_damage_this_turn: false,
        deathtouch_since_sba: false,
        growth: Stats::default(),
        gained_keywords: Keywords::NONE,
        counters: InitialCounters::default(),
        used_abilities: 0,
        attached_to: None,
    });
    match zone {
        Zone::Library => g.state.players[owner.index()].library.push(id),
        Zone::Battlefield => g.register_battlefield_sources(id),
        Zone::Graveyard => g.register_graveyard_sources(id),
        _ => (),
    }
    id
}
fn hand_count(g: &Game, p: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Hand && o.owner == p)
        .count()
}
fn reject(g: &mut Game, action: Action) {
    let snapshot = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), snapshot);
    assert_eq!(g.revision(), revision);
}
fn parity(g: &mut Game, p: Player, source: CardId, item: Option<SpellView>) {
    let item = item.unwrap_or_else(|| g.activation_item(p, source, 0));
    bounded_program(
        g,
        item,
        &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 2,
            },
            Instruction::ChooseManaValueParityDestroyOthers,
        ],
    );
    assert_eq!(g.actor(), Some(p));
    assert_eq!(g.observe(p).decision, "mana_value_parity");
    assert_eq!(
        g.legal_actions(),
        vec![
            Action::Concede,
            Action::ChooseManaValueParity {
                parity: ManaValueParity::Odd
            },
            Action::ChooseManaValueParity {
                parity: ManaValueParity::Even
            },
        ]
    );
}
fn library_program(g: &mut Game, p: Player, target: CardId, own: Option<CardId>) {
    let source = gated(g, p, "Trickster's Stratagem", Zone::Stack);
    let mut item = g.activation_item(p, source, 0);
    item.kind = StackKind::Spell;
    item.targets.push(BoundTarget {
        group: 0,
        target: Target::Card(target),
        object: Some(g.object_ref(target)),
    });
    if let Some(own) = own {
        item.targets.push(BoundTarget {
            group: 1,
            target: Target::Card(own),
            object: Some(g.object_ref(own)),
        });
    }
    bounded_program(
        g,
        item,
        &[
            Instruction::OwnerLibrarySecondFromTop,
            Instruction::SelectGroup(1),
            Instruction::Connive {
                subject: Subject::Targets,
                amount: 1,
            },
        ],
    );
}
#[test]
fn msh_parity_library_core_both_parities_current_creatures_both_controllers_and_zero() {
    for p in [Player::P0, Player::P1] {
        for choice in [ManaValueParity::Odd, ManaValueParity::Even] {
            let mut g = setup(p);
            let source = gated(&mut g, p, "Thanos, the Mad Titan", Zone::Battlefield);
            let mut victims = Vec::new();
            for owner in [p, p.other()] {
                for (name, value) in [
                    ("@token/msh-moloid", 0),
                    ("Llanowar Elves", 1),
                    ("Bear Cub", 2),
                    ("Brazen Scourge", 3),
                    ("Hill Giant", 4),
                    ("Serra Angel", 5),
                    ("Shivan Dragon", 6),
                ] {
                    let id = named(&mut g, owner, name, Zone::Battlefield);
                    victims.push((id, value));
                }
            }
            let noncreature = named(&mut g, p.other(), "Arc Reactor", Zone::Battlefield);
            parity(&mut g, p, source, None);
            assert_eq!(g.object_view(source).counters.plus_one, 2);
            checked(&mut g, Action::ChooseManaValueParity { parity: choice });
            for (id, value) in victims {
                let dies = (value % 2 == 1) == (choice == ManaValueParity::Odd);
                assert_eq!(g.on_battlefield(id), !dies, "{value} {choice:?}");
            }
            assert!(g.on_battlefield(source) && g.on_battlefield(noncreature));
            assert_eq!(
                g.observe(p).creatures_died_this_turn,
                if choice == ManaValueParity::Odd { 6 } else { 8 }
            );
        }
    }
}
#[test]
fn msh_parity_library_core_departed_source_and_new_incarnation_do_not_cancel_choice() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = setup(p);
            let source = gated(&mut g, p, "Thanos, the Mad Titan", Zone::Battlefield);
            let item = g.activation_item(p, source, 0);
            let old = item.object;
            g.move_object(source, Zone::Exile);
            if blink {
                g.move_object(source, Zone::Battlefield);
            }
            assert_ne!(g.object_ref(source), old);
            let victim = named(&mut g, p.other(), "Brazen Scourge", Zone::Battlefield);
            parity(&mut g, p, source, Some(item));
            assert_eq!(g.object_view(source).counters.plus_one, 0);
            checked(
                &mut g,
                Action::ChooseManaValueParity {
                    parity: ManaValueParity::Odd,
                },
            );
            assert!(!g.on_battlefield(victim));
            assert!(!g.on_battlefield(source));
            assert_eq!(
                g.state.objects[source as usize].zone,
                if blink { Zone::Graveyard } else { Zone::Exile }
            );
        }
    }
}
#[test]
fn msh_parity_library_core_effective_copy_cost_exact_exclusion_and_indestructible() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = gated(&mut g, p, "Thanos, the Mad Titan", Zone::Battlefield);
        let item = g.activation_item(p, source, 0);
        let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.apply_temporary_copy(
            g.object_ref(victim),
            CopyRecipe::new(Card::from_name("Brazen Scourge").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        g.apply_temporary_copy(
            g.object_ref(source),
            CopyRecipe::new(Card::from_name("Brazen Scourge").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        let protected = named(&mut g, p.other(), "Brazen Scourge", Zone::Battlefield);
        g.grant_keyword_effects(
            &[protected],
            g.object_ref(protected),
            Keywords::INDESTRUCTIBLE,
        );
        parity(&mut g, p, source, Some(item));
        checked(
            &mut g,
            Action::ChooseManaValueParity {
                parity: ManaValueParity::Odd,
            },
        );
        assert!(g.on_battlefield(source) && g.on_battlefield(protected));
        assert!(!g.on_battlefield(victim));
        assert_eq!(g.state.objects[victim as usize].card.name(), "Bear Cub");
    }
}
#[test]
fn msh_parity_library_core_empty_set_compulsory_choice_ownership_and_atomicity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = gated(&mut g, p, "Thanos, the Mad Titan", Zone::Battlefield);
        let item = g.activation_item(p, source, 0);
        g.state.objects[source as usize].controller = p.other();
        let secret = named(&mut g, p, "Cancel", Zone::Hand);
        parity(&mut g, p, source, Some(item));
        let text = serde_json::to_value(g.observe(p.other())).unwrap();
        assert!(!g.observe(p.other()).objects.iter().any(|x| x.id == secret));
        let action = Action::ChooseManaValueParity {
            parity: ManaValueParity::Even,
        };
        assert_eq!(
            serde_json::from_value::<Action>(serde_json::to_value(&action).unwrap()).unwrap(),
            action
        );
        reject(&mut g, Action::ChooseOptional { accept: false });
        assert_eq!(serde_json::to_value(g.observe(p.other())).unwrap(), text);
        checked(&mut g, action);
        assert!(g.on_battlefield(source));
    }
}
#[test]
fn msh_parity_library_core_library_order_owner_tokens_knowledge_and_destination_legality() {
    for p in [Player::P0, Player::P1] {
        for stolen in [false, true] {
            for token in [false, true] {
                for n in [0usize, 1, 2, 4] {
                    for known in [false, true] {
                        for destination in [
                            LibraryDestination::SecondFromTop,
                            LibraryDestination::Bottom,
                        ] {
                            let mut g = setup(p);
                            let owner = if stolen { p } else { p.other() };
                            while let Some(&id) = g.state.players[owner.index()].library.last() {
                                g.move_object(id, Zone::Graveyard);
                            }
                            let mut before = Vec::new();
                            for _ in 0..n {
                                before.push(named(&mut g, owner, "Forest", Zone::Library));
                            }
                            if known && let Some(&id) = before.last() {
                                g.remember_library_top(owner, g.object_ref(id));
                            }
                            let target = named(
                                &mut g,
                                owner,
                                if token {
                                    "@token/msh-moloid"
                                } else {
                                    "Bear Cub"
                                },
                                Zone::Battlefield,
                            );
                            g.state.objects[target as usize].controller = p.other();
                            library_program(&mut g, p, target, None);
                            assert_eq!(g.actor(), Some(owner));
                            assert!(g.observe(owner).target_library.is_some());
                            assert!(g.observe(owner.other()).target_library.is_none());
                            let object = g.object_ref(target);
                            for bad in [
                                LibraryDestination::Top,
                                LibraryDestination::Graveyard,
                                LibraryDestination::Shuffle,
                            ] {
                                reject(
                                    &mut g,
                                    Action::ChooseLibraryDestination {
                                        object,
                                        destination: bad,
                                    },
                                );
                            }
                            let object = g.object_ref(target);
                            checked(
                                &mut g,
                                Action::ChooseLibraryDestination {
                                    object,
                                    destination,
                                },
                            );
                            let mut expected = before.clone();
                            if !token {
                                expected.insert(
                                    if destination == LibraryDestination::SecondFromTop {
                                        n.saturating_sub(1)
                                    } else {
                                        0
                                    },
                                    target,
                                );
                            }
                            assert_eq!(
                                g.state.players[owner.index()].library.as_slice(),
                                expected.as_slice()
                            );
                            assert_eq!(
                                g.observe(p).last_target_library_move.unwrap().destination,
                                destination
                            );
                            if known && n > 0 {
                                assert_eq!(
                                    g.public_library_top(owner).unwrap().card,
                                    *before.last().unwrap()
                                );
                            } else if n == 0
                                && !token
                                && destination == LibraryDestination::SecondFromTop
                            {
                                assert_eq!(g.public_library_top(owner).unwrap().card, target);
                            } else {
                                assert_eq!(g.public_library_top(owner), None);
                            }
                            assert!(!g.on_battlefield(target));
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn msh_parity_library_core_library_pause_keeps_independent_connive_group_and_private_hand() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let conniver = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let discard = named(&mut g, p, "Cancel", Zone::Hand);
        let count = hand_count(&g, p);
        library_program(&mut g, p, target, Some(conniver));
        assert_eq!(g.actor(), Some(p.other()));
        let object = g.object_ref(target);
        checked(
            &mut g,
            Action::ChooseLibraryDestination {
                object,
                destination: LibraryDestination::SecondFromTop,
            },
        );
        assert_eq!(g.actor(), Some(p));
        assert_eq!(hand_count(&g, p), count + 1); // Only connive draws; the bounded spell source starts on the stack.
        assert!(g.on_battlefield(conniver));
        assert_eq!(g.object_view(conniver).counters.plus_one, 0);
        assert!(!g.observe(p.other()).objects.iter().any(|v| v.id == discard));
        checked(&mut g, Action::Discard { card: discard });
        settle(&mut g);
        assert_eq!(g.object_view(conniver).counters.plus_one, 1);
        assert_eq!(g.state.objects[target as usize].zone, Zone::Library);
        assert!(g.on_battlefield(conniver));
    }
}

#[test]
fn msh_parity_library_core_simultaneous_conditional_indestructible_is_frozen_before_departures() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let doom = named(&mut g, p.other(), "Doctor Doom", Zone::Battlefield);
        let value = g.state.objects[doom as usize].card.fixed_mana_value();
        let choice = if value % 2 == 0 {
            ManaValueParity::Even
        } else {
            ManaValueParity::Odd
        };
        let recipe = CopyRecipe::new(
            Card::from_name(if value % 2 == 0 {
                "Bear Cub"
            } else {
                "Brazen Scourge"
            })
            .unwrap(),
        )
        .with_exceptions(CopyExceptions::ARTIFACT);
        let refs = g.create_copy_groups_inner(
            p.other(),
            &[TokenGroupView {
                token: recipe.definition,
                amount: 1,
                tapped: false,
            }],
            &[Some(recipe)],
            None,
        );
        g.advance_entries();
        settle(&mut g);
        assert!(g.keywords(doom).contains(Keywords::INDESTRUCTIBLE));
        let source = gated(&mut g, p, "Thanos, the Mad Titan", Zone::Battlefield);
        parity(&mut g, p, source, None);
        checked(&mut g, Action::ChooseManaValueParity { parity: choice });
        assert!(!g.on_battlefield(refs[0].card));
        assert!(g.on_battlefield(doom));
        assert!(!g.keywords(doom).contains(Keywords::INDESTRUCTIBLE));
    }
}
#[test]
fn msh_parity_library_core_counter_replacements_finish_before_mandatory_parity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for _ in 0..2 {
            let season = named(&mut g, p, "Doubling Season", Zone::Hand);
            cast(&mut g, season, None, &[]);
            settle(&mut g);
        }
        let source = gated(&mut g, p, "Thanos, the Mad Titan", Zone::Battlefield);
        let item = g.activation_item(p, source, 0);
        bounded_program(
            &mut g,
            item,
            &[
                Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: 2,
                },
                Instruction::ChooseManaValueParityDestroyOthers,
            ],
        );
        assert!(matches!(g.state.pending, Pending::Replacement { .. }));
        assert_eq!(g.object_view(source).counters.plus_one, 0);
        reject(
            &mut g,
            Action::ChooseManaValueParity {
                parity: ManaValueParity::Even,
            },
        );
        let mut n = 0;
        while matches!(g.state.pending, Pending::Replacement { .. }) {
            let effect = g
                .observe(g.actor().unwrap())
                .replacement_choice
                .unwrap()
                .effects[0];
            checked(
                &mut g,
                Action::ChooseReplacement {
                    source: effect.source,
                    ability: effect.ability,
                },
            );
            n += 1;
        }
        assert!(n > 0);
        assert_eq!(g.object_view(source).counters.plus_one, 8);
        assert_eq!(g.observe(p).decision, "mana_value_parity");
        checked(
            &mut g,
            Action::ChooseManaValueParity {
                parity: ManaValueParity::Even,
            },
        );
    }
}
#[test]
fn msh_parity_library_core_competing_zone_replacements_replay_frozen_batch_without_readding_counters()
 {
    for p in [Player::P0, Player::P1] {
        for effect in [
            ZoneReplacementKind::Exile,
            ZoneReplacementKind::RevealShuffle,
        ] {
            let mut g = setup(p);
            let rip = named(&mut g, p, "Rest in Peace", Zone::Hand);
            cast(&mut g, rip, None, &[]);
            settle(&mut g);
            let source = gated(&mut g, p, "Thanos, the Mad Titan", Zone::Battlefield);
            let progenitus = named(&mut g, p.other(), "Progenitus", Zone::Battlefield);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            parity(&mut g, p, source, None);
            checked(
                &mut g,
                Action::ChooseManaValueParity {
                    parity: ManaValueParity::Even,
                },
            );
            assert!(matches!(g.state.pending, Pending::ZoneReplacement { .. }));
            assert!(g.on_battlefield(progenitus) && g.on_battlefield(bear));
            assert_eq!(g.actor(), Some(p.other()));
            checked(&mut g, Action::ChooseZoneReplacement { effect });
            settle(&mut g);
            assert_eq!(
                g.state.objects[progenitus as usize].zone,
                if effect == ZoneReplacementKind::Exile {
                    Zone::Exile
                } else {
                    Zone::Library
                }
            );
            assert_eq!(g.state.objects[bear as usize].zone, Zone::Exile);
            assert_eq!(g.object_view(source).counters.plus_one, 2);
            assert_eq!(g.observe(p).creatures_died_this_turn, 0);
        }
    }
}
#[test]
fn msh_parity_library_core_current_modal_face_cost_and_copied_token_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let king = named(&mut g, p.other(), "King T'Challa", Zone::Battlefield);
        assert_eq!(g.state.objects[king as usize].card.fixed_mana_value(), 3);
        let mut transform = g.activation_item(p.other(), king, 0);
        transform.source_transform = Some(g.transform_serial(g.object_ref(king)));
        // Bounded existing transform continuation; the prospective consumer is still gated.
        g.transform_source(&transform);
        assert_eq!(g.state.objects[king as usize].card.fixed_mana_value(), 6);
        let recipe = g.copiable_recipe(g.object_ref(king));
        let refs = g.create_copy_groups_inner(
            p,
            &[TokenGroupView {
                token: recipe.definition,
                amount: 1,
                tapped: false,
            }],
            &[Some(recipe)],
            None,
        );
        g.advance_entries();
        settle(&mut g);
        let copy = refs[0].card;
        assert_eq!(
            g.copiable_recipe(g.object_ref(copy))
                .definition
                .fixed_mana_value(),
            6
        );
        let source = gated(&mut g, p, "Thanos, the Mad Titan", Zone::Battlefield);
        parity(&mut g, p, source, None);
        checked(
            &mut g,
            Action::ChooseManaValueParity {
                parity: ManaValueParity::Even,
            },
        );
        assert!(!g.on_battlefield(king) && !g.on_battlefield(copy));
        assert!(g.on_battlefield(source));
    }
}
