//! Paid rules interactions for the five complete choice/link consumers.
use super::ability_timeline_tests::{cast, checked, setup};
use super::tests::spawn;
use super::*;
use crate::{CardName, PermanentType};

fn card(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn reject(g: &mut Game, action: Action) {
    let saved = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), revision);
}
fn step(g: &mut Game) {
    let actions = g.legal_actions();
    let action = if g.state.pending == Pending::Priority {
        Action::Pass
    } else {
        actions
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::PutTrigger { .. } | Action::FinishTargets | Action::PayCosts { .. }
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending))
    };
    checked(g, action);
}
fn until(g: &mut Game, predicate: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..40 {
        if predicate(g) {
            return;
        }
        step(g);
    }
    panic!("did not reach choice {:?}", g.state.pending);
}
fn settled(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.announcing.is_none()
    });
}
fn activate(g: &mut Game, id: CardId, ability: u16) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate{card,ability:i,..} if *card==id && *i==ability))
        .expect("legal paid ability");
    checked(g, a);
}
fn has_activation(g: &Game, id: CardId) -> bool {
    g.legal_actions().iter().any(|a| matches!(a,Action::Activate{card,..}|Action::AnnounceActivate{card,..}|Action::BeginActivate{card,..} if *card==id))
}
fn spyglass(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = card(g, p, "Sorcerous Spyglass", Zone::Hand);
    cast(g, id, None, &[]);
    until(g, |g| matches!(g.state.pending, Pending::EntryValue { .. }));
    checked(
        g,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::CardName(CardName::from_name(name).unwrap()),
        },
    );
    settled(g);
    id
}
fn pile(g: &mut Game, object: ObjectRef, side: LibraryPile) {
    checked(g, Action::AssignLibraryPile { object, pile: side });
}

#[test]
fn linked_choices_curator_paid_uncounterable_private_partition_and_both_piles() {
    for p in [Player::P0, Player::P1] {
        for chosen in [LibraryPile::FaceDown, LibraryPile::FaceUp] {
            let mut g = setup(p);
            let top: Vec<_> = ["Plains", "Island", "Swamp", "Mountain", "Forest"]
                .into_iter()
                .map(|n| card(&mut g, p, n, Zone::Library))
                .collect();
            let sphinx = card(&mut g, p, "Curator of Destinies", Zone::Hand);
            let before_mana = g.state.players[p.index()].mana;
            cast(&mut g, sphinx, None, &[]);
            checked(&mut g, Action::Pass);
            let counter = card(&mut g, p.other(), "Essence Scatter", Zone::Hand);
            cast(&mut g, counter, None, &[Target::Spell(sphinx)]);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            assert_eq!(g.object_view(sphinx).zone, Zone::Stack);
            until(&mut g, |g| g.observe(p).library_piles.is_some());
            assert!(g.state.players[p.index()].mana.iter().sum::<u32>() < before_mana.iter().sum());
            assert!(g.object_view(sphinx).keywords.contains(Keywords::FLYING));
            assert!(g.observe(p.other()).library_piles.is_none());
            assert!(g.observe(p.other()).actions.is_empty());
            let refs: Vec<_> = top.iter().map(|&id| g.object_ref(id)).collect();
            reject(&mut g, Action::ChooseLibraryPile { pile: chosen });
            let wrong = g.object_ref(sphinx);
            reject(
                &mut g,
                Action::AssignLibraryPile {
                    object: wrong,
                    pile: chosen,
                },
            );
            for (i, &object) in refs.iter().enumerate() {
                pile(
                    &mut g,
                    object,
                    if i < 2 {
                        LibraryPile::FaceDown
                    } else {
                        LibraryPile::FaceUp
                    },
                );
            }
            assert_eq!(g.actor(), Some(p.other()));
            let public = g.observe(p.other()).library_piles.unwrap();
            assert_eq!(public.face_down_count, 2);
            assert!(public.face_down.is_none());
            assert!(public.remaining.is_empty());
            assert_eq!(public.face_up.len(), 3);
            assert!(
                refs[..2]
                    .iter()
                    .all(|r| !public.face_up.iter().any(|v| v.reference == *r))
            );
            let hand = g.hand_count(p);
            checked(&mut g, Action::ChooseLibraryPile { pile: chosen });
            settled(&mut g);
            assert_eq!(
                g.hand_count(p) - hand,
                if chosen == LibraryPile::FaceDown {
                    2
                } else {
                    3
                }
            );
            for (i, &id) in top.iter().enumerate() {
                let in_hand = (i < 2) == (chosen == LibraryPile::FaceDown);
                assert_eq!(
                    g.object_view(id).zone,
                    if in_hand { Zone::Hand } else { Zone::Graveyard }
                );
                assert_eq!(
                    g.observe(p.other()).objects.iter().any(|o| o.id == id),
                    !in_hand || i >= 2
                );
            }
            assert!(g.observe(p).library_piles.is_none());
        }
    }
}
#[test]
fn linked_choices_curator_empty_pile_short_library_and_departed_source() {
    for p in [Player::P0, Player::P1] {
        for number in [0, 1, 3] {
            let mut g = setup(p);
            let old = g.state.players[p.index()].library.clone();
            for id in old {
                g.move_object(id, Zone::Graveyard);
            }
            for _ in 0..number {
                card(&mut g, p, "Forest", Zone::Library);
            }
            let source = card(&mut g, p, "Curator of Destinies", Zone::Hand);
            cast(&mut g, source, None, &[]);
            until(&mut g, |g| {
                g.state.stack.iter().any(|i| {
                    i.card == source && matches!(i.kind, StackKind::TriggeredAbility { .. })
                })
            });
            g.move_object(source, Zone::Graveyard);
            until(&mut g, |g| g.observe(p).library_piles.is_some());
            while let Some(object) = g
                .observe(p)
                .library_piles
                .unwrap()
                .remaining
                .first()
                .map(|v| v.reference)
            {
                pile(&mut g, object, LibraryPile::FaceUp);
            }
            let hand = g.hand_count(p);
            checked(
                &mut g,
                Action::ChooseLibraryPile {
                    pile: LibraryPile::FaceDown,
                },
            );
            settled(&mut g);
            assert_eq!(g.hand_count(p), hand);
            assert_eq!(g.state.players[p.index()].library.len(), 0);
        }
    }
}
#[test]
fn linked_choices_brew_simultaneous_apnap_and_other_controller_owned_recovery() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let yours = card(&mut g, p, "Bear Cub", Zone::Battlefield);
        let stolen = card(&mut g, p, "Savannah Lions", Zone::Battlefield);
        g.record_control(g.object_ref(stolen), p.other(), None);
        let brew = card(&mut g, p, "Deadly Brew", Zone::Hand);
        cast(&mut g, brew, None, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        assert_eq!(g.actor(), Some(p));
        let yours_ref = g.object_ref(yours);
        checked(&mut g, Action::Sacrifice { object: yours_ref });
        assert_eq!(g.object_view(yours).zone, Zone::Battlefield);
        assert_eq!(g.actor(), Some(p.other()));
        assert_eq!(
            g.observe(p.other()).effect_choice.unwrap().selected[p.index()],
            vec![yours_ref]
        );
        reject(&mut g, Action::Sacrifice { object: yours_ref });
        let stolen_ref = g.object_ref(stolen);
        checked(&mut g, Action::Sacrifice { object: stolen_ref });
        assert_eq!(g.object_view(yours).zone, Zone::Graveyard);
        assert_eq!(g.object_view(stolen).zone, Zone::Graveyard);
        let view = g.observe(p).recovery_choice.unwrap();
        assert_eq!(view.candidates, vec![g.object_ref(stolen)]);
        let wrong = g.object_ref(yours);
        reject(
            &mut g,
            Action::RecoverMilled {
                object: Some(wrong),
            },
        );
        let chosen = g.object_ref(stolen);
        checked(
            &mut g,
            Action::RecoverMilled {
                object: Some(chosen),
            },
        );
        settled(&mut g);
        assert_eq!(g.object_view(stolen).zone, Zone::Hand);
        assert_eq!(g.object_view(yours).zone, Zone::Graveyard);
    }
}
#[test]
fn linked_choices_brew_token_or_stolen_sacrifice_qualifies_and_decline_is_optional() {
    for p in [Player::P0, Player::P1] {
        for token in [false, true] {
            let mut g = setup(p);
            let source = if token {
                Card::from_name("@token/47a984d1dc0d0363f137a7dd").unwrap()
            } else {
                Card::from_name("Bear Cub").unwrap()
            };
            let own = spawn(
                &mut g,
                if token { p } else { p.other() },
                source,
                Zone::Battlefield,
            );
            if !token {
                g.record_control(g.object_ref(own), p, None);
            }
            let recover = card(&mut g, p, "Forest", Zone::Graveyard);
            let brew = card(&mut g, p, "Deadly Brew", Zone::Hand);
            cast(&mut g, brew, None, &[]);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectChoice { .. })
            });
            let object = g.object_ref(own);
            checked(&mut g, Action::Sacrifice { object });
            assert_eq!(
                g.observe(p).recovery_choice.unwrap().candidates,
                vec![g.object_ref(recover)]
            );
            checked(&mut g, Action::RecoverMilled { object: None });
            settled(&mut g);
            assert_eq!(g.object_view(recover).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn linked_choices_liliana_paid_loyalty_zombie_and_simultaneous_death_draws() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let lili = card(&mut g, p, "Liliana, Dreadhorde General", Zone::Hand);
        cast(&mut g, lili, None, &[]);
        settled(&mut g);
        assert_eq!(g.object_view(lili).counters.loyalty, 6);
        activate(&mut g, lili, 0);
        assert_eq!(g.object_view(lili).counters.loyalty, 7);
        settled(&mut g);
        let zombie = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Battlefield && o.is_token())
            .map(|(id, _)| id as CardId)
            .unwrap();
        let z = g.object_view(zombie);
        assert_eq!((z.power, z.toughness, z.colors), (2, 2, 4));
        assert!(g.has_subtype(zombie, "Zombie"));
        assert!(!has_activation(&g, lili));
        g.state.objects[lili as usize].used_abilities = 0;
        let own = card(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foe = card(&mut g, p.other(), "Savannah Lions", Zone::Battlefield);
        activate(&mut g, lili, 1);
        assert_eq!(g.object_view(lili).counters.loyalty, 3);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        for id in [zombie, own] {
            let object = g.object_ref(id);
            checked(&mut g, Action::Sacrifice { object });
        }
        assert_eq!(g.object_view(own).zone, Zone::Battlefield);
        let draws = g.state.players[p.index()].cards_drawn_this_turn;
        let object = g.object_ref(foe);
        checked(&mut g, Action::Sacrifice { object });
        g.move_object(lili, Zone::Graveyard);
        settled(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn - draws, 2);
    }
}
#[test]
fn linked_choices_liliana_keep_multitype_can_choose_same_or_distinct_permanents() {
    for p in [Player::P0, Player::P1] {
        for same in [false, true] {
            let mut g = setup(p);
            let lili = card(&mut g, p, "Liliana, Dreadhorde General", Zone::Hand);
            cast(&mut g, lili, None, &[]);
            settled(&mut g);
            g.state.objects[lili as usize].counters.loyalty = 9;
            let artifact = card(&mut g, p.other(), "Meteor Golem", Zone::Battlefield);
            let creature = card(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let land = card(&mut g, p.other(), "Forest", Zone::Battlefield);
            let otherland = card(&mut g, p.other(), "Island", Zone::Battlefield);
            let enchantment = card(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
            let pw = card(
                &mut g,
                p.other(),
                "Ajani, Caller of the Pride",
                Zone::Battlefield,
            );
            g.state.objects[pw as usize].counters.loyalty = 4;
            activate(&mut g, lili, 2);
            until(&mut g, |g| g.observe(p).keep_types.is_some());
            assert_eq!(g.object_view(lili).zone, Zone::Graveyard);
            while let Some(view) = g.observe(p).keep_types {
                assert_eq!(g.actor(), Some(p.other()));
                let chosen = match view.permanent_type {
                    PermanentType::Artifact => artifact,
                    PermanentType::Creature => {
                        if same {
                            artifact
                        } else {
                            creature
                        }
                    }
                    PermanentType::Land => land,
                    PermanentType::Enchantment => enchantment,
                    PermanentType::Planeswalker => pw,
                    PermanentType::Battle => unreachable!(),
                };
                let bad = g.object_ref(lili);
                reject(&mut g, Action::KeepPermanentType { object: bad });
                let object = g.object_ref(chosen);
                checked(&mut g, Action::KeepPermanentType { object });
            }
            settled(&mut g);
            assert_eq!(g.object_view(artifact).zone, Zone::Battlefield);
            assert_eq!(
                g.object_view(creature).zone,
                if same {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(g.object_view(otherland).zone, Zone::Graveyard);
            for id in [land, enchantment, pw] {
                assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            }
        }
    }
}
#[test]
fn linked_choices_hoard_paid_optional_search_link_and_owner_hand_after_control_change() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let artifact = card(&mut g, p, "Mazemind Tome", Zone::Library);
        let dragon = card(&mut g, p, "Hoarding Dragon", Zone::Hand);
        cast(&mut g, dragon, None, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: true });
        assert!(g.observe(p.other()).search_choice.is_none());
        checked(&mut g, Action::ChooseSearch { card: artifact });
        checked(&mut g, Action::FinishSearch);
        settled(&mut g);
        assert_eq!(g.object_view(artifact).zone, Zone::Exile);
        assert_eq!(g.observe(p.other()).linked_exile.len(), 1);
        g.record_control(g.object_ref(dragon), p.other(), None);
        g.move_object(dragon, Zone::Graveyard);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        assert_eq!(g.actor(), Some(p.other()));
        checked(&mut g, Action::ChooseOptional { accept: true });
        settled(&mut g);
        assert_eq!(g.object_view(artifact).zone, Zone::Hand);
        assert_eq!(g.object_view(artifact).owner, p);
        assert!(g.observe(p).linked_exile.is_empty());
    }
}
#[test]
fn linked_choices_hoard_departure_before_search_and_reentry_never_rebinds() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let artifact = card(&mut g, p, "Mazemind Tome", Zone::Library);
        let dragon = card(&mut g, p, "Hoarding Dragon", Zone::Hand);
        cast(&mut g, dragon, None, &[]);
        until(&mut g, |g| {
            g.state
                .stack
                .iter()
                .any(|i| i.card == dragon && matches!(i.kind, StackKind::TriggeredAbility { .. }))
        });
        let old = g.object_ref(dragon);
        g.move_object(dragon, Zone::Graveyard);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: true });
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: true });
        checked(&mut g, Action::ChooseSearch { card: artifact });
        checked(&mut g, Action::FinishSearch);
        settled(&mut g);
        assert_eq!(g.observe(p).linked_exile[0].source, old);
        g.move_object(dragon, Zone::Battlefield);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: false });
        settled(&mut g);
        g.move_object(dragon, Zone::Graveyard);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: true });
        settled(&mut g);
        assert_eq!(g.object_view(artifact).zone, Zone::Exile);
    }
}
#[test]
fn linked_choices_spyglass_preentry_private_hand_regular_string_name_and_own_restriction() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let private = card(&mut g, p.other(), "Shivan Dragon", Zone::Hand);
        let mine = card(&mut g, p, "Shivan Dragon", Zone::Battlefield);
        let glass = card(&mut g, p, "Sorcerous Spyglass", Zone::Hand);
        cast(&mut g, glass, None, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EntryValue { .. })
        });
        assert_eq!(g.object_view(glass).zone, Zone::Stack);
        assert!(
            g.observe(p)
                .entry_value
                .unwrap()
                .opponent_hand
                .unwrap()
                .iter()
                .any(|o| o.id == private)
        );
        assert!(
            g.observe(p.other())
                .entry_value
                .unwrap()
                .opponent_hand
                .is_none()
        );
        assert!(!g.observe(p).objects.iter().any(|o| o.id == private));
        reject(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::Color(crate::ChoiceColor::Blue),
            },
        );
        let value = EntryChoiceValue::CardName(CardName::from_name("Shivan Dragon").unwrap());
        checked(&mut g, Action::ChooseEntryValue { value });
        settled(&mut g);
        assert!(!has_activation(&g, mine));
        let wire = serde_json::to_value(g.object_view(glass)).unwrap();
        assert_eq!(wire["entry_choice"]["CardName"], "Shivan Dragon");
        assert_eq!(g.object_view(mine).name, "Shivan Dragon");
        g.move_object(glass, Zone::Hand);
        assert_eq!(g.object_view(glass).entry_choice, None);
        assert!(has_activation(&g, mine));
    }
}
#[test]
fn linked_choices_spyglass_hand_graveyard_mana_and_ability_removal() {
    for p in [Player::P0, Player::P1] {
        for (name, zone) in [
            ("Daggermaw Megalodon", Zone::Hand),
            ("Rubblebelt Maverick", Zone::Graveyard),
        ] {
            let mut g = setup(p);
            let object = card(&mut g, p, name, zone);
            card(&mut g, p, "Bear Cub", Zone::Battlefield);
            assert!(has_activation(&g, object));
            let glass = spyglass(&mut g, p, name);
            assert!(!has_activation(&g, object));
            g.move_object(glass, Zone::Graveyard);
            assert!(has_activation(&g, object));
        }
        let mut g = setup(p);
        let elf = card(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        g.state.objects[elf as usize].summoning_sick = false;
        spyglass(&mut g, p, "Llanowar Elves");
        assert!(g.legal_actions().iter().any(
            |a| matches!(a,Action::TapMana{card}|Action::ActivateMana{card,..} if *card==elf)
        ));
        let shivan = card(&mut g, p, "Shivan Dragon", Zone::Battlefield);
        let glass = spyglass(&mut g, p, "Shivan Dragon");
        assert!(!has_activation(&g, shivan));
        let removal = card(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, removal, None, &[Target::Card(glass)]);
        settled(&mut g);
        assert!(has_activation(&g, shivan));
        assert_eq!(
            g.object_view(glass).entry_choice,
            Some(EntryChoiceValue::CardName(
                CardName::from_name("Shivan Dragon").unwrap()
            ))
        );
    }
}
#[test]
fn linked_choices_name_inventory_faces_invalid_names_and_cold_layout() {
    for name in [
        "Fire",
        "Ice",
        "Petty Theft",
        "Brazen Borrower",
        "Delver of Secrets",
        "Insectile Aberration",
        "Black Lotus",
        "Sorcerous Spyglass",
    ] {
        let chosen = CardName::from_name(name).unwrap();
        let json = serde_json::to_string(&chosen).unwrap();
        assert_eq!(serde_json::from_str::<CardName>(&json).unwrap(), chosen);
        assert_eq!(chosen.as_str(), name);
    }
    for name in [
        "Fire // Ice",
        "Brazen Borrower // Petty Theft",
        "A fake card name",
        "Zombie Token",
        "Absorb Energy",
    ] {
        assert!(CardName::from_name(name).is_none(), "{name}");
    }
    let g = setup(Player::P0);
    assert!(g.state.linked_choices.is_none());
    assert_eq!(std::mem::size_of::<Object>(), 144);
    assert_eq!(std::mem::size_of::<Pending>(), 40);
    assert_eq!(std::mem::size_of::<ManaProduction>(), 32);
}

#[test]
fn linked_choices_brew_replaced_sacrifice_counts_but_pending_aura_sba_is_not_recoverable() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let colossus = card(&mut g, p, "Darksteel Colossus", Zone::Battlefield);
        let recover = card(&mut g, p, "Forest", Zone::Graveyard);
        let brew = card(&mut g, p, "Deadly Brew", Zone::Hand);
        cast(&mut g, brew, None, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        let object = g.object_ref(colossus);
        checked(&mut g, Action::Sacrifice { object });
        assert_eq!(g.object_view(colossus).zone, Zone::Library);
        assert_eq!(
            g.observe(p).recovery_choice.unwrap().candidates,
            vec![g.object_ref(recover)]
        );
        checked(&mut g, Action::RecoverMilled { object: None });
        settled(&mut g);
        let bear = card(&mut g, p, "Bear Cub", Zone::Battlefield);
        let aura = card(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(bear)]);
        settled(&mut g);
        let brew = card(&mut g, p, "Deadly Brew", Zone::Hand);
        cast(&mut g, brew, None, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        let object = g.object_ref(bear);
        checked(&mut g, Action::Sacrifice { object });
        assert_eq!(g.object_view(aura).zone, Zone::Battlefield);
        assert!(
            !g.observe(p)
                .recovery_choice
                .unwrap()
                .candidates
                .iter()
                .any(|r| r.card == aura)
        );
        checked(&mut g, Action::RecoverMilled { object: None });
        settled(&mut g);
        assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
    }
}
#[test]
fn linked_choices_hoard_optional_decline_failure_to_find_and_stale_exile() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            let dragon = card(&mut g, p, "Hoarding Dragon", Zone::Hand);
            cast(&mut g, dragon, None, &[]);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectOptional { .. })
            });
            let rng = g.state.rng;
            let library = g.state.players[p.index()].library.clone();
            checked(&mut g, Action::ChooseOptional { accept });
            if accept {
                checked(&mut g, Action::FinishSearch);
                assert_ne!(g.state.rng, rng);
            } else {
                assert_eq!(g.state.rng, rng);
                assert_eq!(g.state.players[p.index()].library, library);
            }
            settled(&mut g);
            assert!(g.observe(p).linked_exile.is_empty());
        }
        let mut g = setup(p);
        let artifact = card(&mut g, p, "Mazemind Tome", Zone::Library);
        let dragon = card(&mut g, p, "Hoarding Dragon", Zone::Hand);
        cast(&mut g, dragon, None, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: true });
        checked(&mut g, Action::ChooseSearch { card: artifact });
        checked(&mut g, Action::FinishSearch);
        settled(&mut g);
        let exiled = g.object_ref(artifact);
        g.move_object(artifact, Zone::Hand);
        g.move_object(artifact, Zone::Exile);
        assert_ne!(g.object_ref(artifact), exiled);
        g.move_object(dragon, Zone::Graveyard);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: true });
        settled(&mut g);
        assert_eq!(g.object_view(artifact).zone, Zone::Exile);
        assert!(g.observe(p).linked_exile.is_empty());
    }
}
#[test]
fn linked_choices_time_stop_keeps_existing_hoard_and_abandons_only_suspended_choices() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let artifact = card(&mut g, p, "Mazemind Tome", Zone::Library);
        let dragon = card(&mut g, p, "Hoarding Dragon", Zone::Hand);
        cast(&mut g, dragon, None, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: true });
        checked(&mut g, Action::ChooseSearch { card: artifact });
        checked(&mut g, Action::FinishSearch);
        settled(&mut g);
        let links = g.observe(p).linked_exile;
        let stop = card(&mut g, p, "Time Stop", Zone::Hand);
        cast(&mut g, stop, None, &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.object_view(stop).zone, Zone::Exile);
        assert_eq!(g.observe(p).linked_exile, links);
        // No player receives priority during these choices. The cancellation hook
        // also works on an owned branch with a suspended resolution continuation.
        let mut suspended = setup(p);
        let curator = card(&mut suspended, p, "Curator of Destinies", Zone::Hand);
        cast(&mut suspended, curator, None, &[]);
        until(&mut suspended, |g| g.observe(p).library_piles.is_some());
        let library = suspended.state.players[p.index()].library.clone();
        let snapshot = suspended.snapshot();
        reject(&mut suspended, Action::Pass);
        let mut branch = {
            let parent = suspended.clone();
            parent.clone()
        };
        branch.end_turn_effect();
        assert!(branch.observe(p).library_piles.is_none());
        assert!(branch.state.linked_choices.is_none());
        assert_eq!(branch.state.players[p.index()].library, library);
        assert_eq!(suspended.snapshot(), snapshot);
        suspended.restore(&snapshot).unwrap();
        assert!(suspended.observe(p).library_piles.is_some());
    }
}

#[test]
fn linked_choices_name_policy_uses_visible_opponent_abilities_and_private_look() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hidden = card(&mut g, p.other(), "Shivan Dragon", Zone::Hand);
        card(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let glass = card(&mut g, p, "Sorcerous Spyglass", Zone::Hand);
        cast(&mut g, glass, None, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EntryValue { .. })
        });
        assert!(!g.observe(p).objects.iter().any(|o| o.id == hidden));
        let actions = g.legal_actions();
        let chosen = actions[crate::policy::smoke_index(&g, &actions)].clone();
        assert_eq!(
            chosen,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CardName(CardName::from_name("Shivan Dragon").unwrap())
            }
        );
        checked(&mut g, chosen);
        settled(&mut g);
    }
}
