//! Paid modal faces, identity-preserving transformation and existing-pool copies.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
fn pay(g: &mut Game) {
    for _ in 0..20 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::ChooseManaLotSpend { .. }
                )
            })
            .unwrap();
        checked(g, a);
    }
    panic!("unpaid");
}
fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn monica(g: &mut Game, p: Player, back: bool) -> CardId {
    let id = named(g, p, "Monica Rambeau", Zone::Hand);
    checked(
        g,
        if back {
            Action::AnnounceBack { card: id }
        } else {
            Action::AnnounceCast { card: id }
        },
    );
    pay(g);
    settle(g);
    id
}
fn settle_copy(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..100 {
        if matches!(g.state.pending, Pending::Priority)
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        let actions = g.legal_actions();
        let a=actions.iter().find(|a|matches!(a,Action::KeepLegend{object} if g.state.objects[object.card as usize].is_copy_token())).cloned()
   .or_else(||actions.iter().find(|a|matches!(a,Action::PayCosts{..}|Action::FinishTargets)).cloned())
   .unwrap_or_else(||if matches!(g.state.pending,Pending::Priority){Action::Pass}else{actions.into_iter().find(|a|!matches!(a,Action::Concede)).unwrap()});
        checked(g, a);
    }
    panic!("copy did not settle");
}
fn transform(g: &mut Game, id: CardId) {
    checked(g, activate(id));
    pay(g);
    settle(g);
}
#[test]
fn msh_faces_paid_both_faces_have_real_costs_names_and_stack_characteristics() {
    for p in [Player::P0, Player::P1] {
        for back in [false, true] {
            let mut g = setup(p);
            let id = named(&mut g, p, "Monica Rambeau", Zone::Hand);
            let front = Card::from_name("Monica Rambeau").unwrap();
            assert_eq!(front.identity(), "Monica Rambeau // Photon, Living Light");
            assert_eq!(front.cost(), ([1, 0, 0, 0, 0, 0], 2));
            g.state.players[p.index()].mana = if back {
                [2, 0, 0, 1, 0, 2]
            } else {
                [1, 0, 0, 0, 0, 2]
            };
            let a = if back {
                Action::AnnounceBack { card: id }
            } else {
                Action::AnnounceCast { card: id }
            };
            checked(&mut g, a);
            let request = g.announcement_payment_request(p).unwrap();
            assert_eq!(
                (request.required, request.generic),
                if back {
                    ([2, 0, 0, 1, 0, 0], 2)
                } else {
                    ([1, 0, 0, 0, 0, 0], 2)
                }
            );
            assert!(g.announcement_payment_request(p.other()).is_none());
            reject(&mut g, Action::PayCosts { payment: [0; 6] });
            let v = g.object_view(id);
            assert_eq!(
                v.name(),
                if back {
                    "Photon, Living Light"
                } else {
                    "Monica Rambeau"
                }
            );
            assert_eq!(v.card.fixed_mana_value(), if back { 5 } else { 3 });
            assert_eq!(v.colors, if back { 9 } else { 1 });
            pay(&mut g);
            settle(&mut g);
            let v = g.object_view(id);
            assert_eq!((v.power, v.toughness), if back { (4, 4) } else { (3, 3) });
            assert_eq!(v.face.unwrap().back, back);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(v.keywords.contains(Keywords::FLYING));
            assert_eq!(v.keywords.contains(Keywords::HEXPROOF), back);
        }
    }
}
#[test]
fn msh_faces_transform_is_sorcery_preserves_exact_object_damage_counters_growth_and_attachments() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = monica(&mut g, p, false);
        let boots = named(&mut g, p, "Swiftfoot Boots", Zone::Hand);
        cast(&mut g, boots, None, &[]);
        settle(&mut g);
        checked(&mut g, activate(boots));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
        pay(&mut g);
        settle(&mut g);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Card(id)]);
        settle(&mut g);
        let reference = g.object_ref(id);
        g.set_counters(
            id,
            Counters::of(CounterKind::PlusOne, 2).with(CounterKind::Menace, 1),
        );
        let before = g.object_view(id);
        g.state.objects[id as usize].tapped = true;
        g.state.step = Step::BeginCombat;
        reject(&mut g, activate(id));
        g.state.step = Step::Main1;
        transform(&mut g, id);
        let after = g.object_view(id);
        assert_eq!(after.reference, reference);
        assert_eq!(after.counters, before.counters);
        assert_eq!(after.damage, 2);
        assert!(after.keywords.contains(Keywords::MENACE));
        assert!(after.tapped);
        assert_eq!(after.summoning_sick, before.summoning_sick);
        assert_eq!(after.entered_this_turn, before.entered_this_turn);
        assert_eq!(after.power, before.power + 1);
        assert_eq!(g.state.objects[boots as usize].attached_to, Some(reference));
        assert_eq!(g.state.cast_sources.iter().filter(|&&x| x == id).count(), 1);
        assert_eq!(after.face.unwrap().transforms, 1);
        reject(&mut g, activate(id));
    }
}
#[test]
fn msh_faces_prowess_and_other_creature_counter_trigger_use_exact_current_face() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = monica(&mut g, p, false);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Giant Growth", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.object_view(id).power, 4);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 0);
        transform(&mut g, id);
        assert_eq!(g.object_view(id).power, 5);
        let spell = named(&mut g, p, "Giant Growth", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.object_view(id).power, 6);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
    }
}
#[test]
fn msh_faces_source_serial_guard_and_blink_never_rebind_old_transform_packets() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = monica(&mut g, p, false);
        checked(&mut g, activate(id));
        pay(&mut g);
        let packet = g.state.stack.last().unwrap().clone();
        assert_eq!(packet.source_transform, Some(0));
        settle(&mut g);
        let first = g.snapshot();
        g.transform_source(&packet);
        assert_eq!(g.snapshot(), first);
        let departed = g.move_object(id, Zone::Hand);
        assert_eq!(departed.before.name(), "Photon, Living Light");
        assert_eq!(g.object_view(id).name(), "Monica Rambeau");
        assert_eq!(g.object_view(id).card.fixed_mana_value(), 3);
        g.move_object(id, Zone::Battlefield);
        let reincarnation = g.object_ref(id);
        assert_ne!(reincarnation, packet.object);
        assert_eq!(g.object_view(id).face.unwrap().transforms, 0);
        g.transform_source(&packet);
        assert_eq!(g.object_ref(id), reincarnation);
        assert_eq!(g.object_view(id).name(), "Monica Rambeau");
    }
}
#[test]
fn msh_faces_free_graveyard_exile_and_library_permissions_use_selected_back() {
    for p in [Player::P0, Player::P1] {
        for origin in [Zone::Hand, Zone::Graveyard, Zone::Exile, Zone::Library] {
            let mut g = setup(p);
            let id = named(&mut g, p, "Monica Rambeau", origin);
            match origin {
                Zone::Hand => {
                    named(&mut g, p, "Omniscience", Zone::Battlefield);
                    g.state.players[p.index()].mana = [0; 6];
                    checked(
                        &mut g,
                        Action::AnnounceFree {
                            card: id,
                            face: CastFace::Back,
                            flash_surcharge: false,
                        },
                    );
                }
                Zone::Graveyard => {
                    g.grant_graveyard_permission(p, g.object_ref(id), GraveyardCastKind::Cast);
                    checked(&mut g, Action::AnnounceBack { card: id });
                }
                Zone::Exile => {
                    g.grant_exile_play_permission(p, g.object_ref(id), false);
                    checked(&mut g, Action::AnnounceBack { card: id });
                }
                Zone::Library => {
                    named(&mut g, p, "Vizier of the Menagerie", Zone::Battlefield);
                    checked(&mut g, Action::AnnounceBack { card: id });
                }
                _ => unreachable!(),
            };
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(id).name(), "Photon, Living Light");
        }
    }
}
#[test]
fn msh_faces_token_copy_enters_same_face_and_transforms_with_copiable_exceptions() {
    for p in [Player::P0, Player::P1] {
        for back in [false, true] {
            let mut g = setup(p);
            let id = monica(&mut g, p, back);
            let spell = named(&mut g, p, "Applied Geometry", Zone::Hand);
            cast(&mut g, spell, None, &[Target::Card(id)]);
            settle_copy(&mut g);
            let copy = (0..g.state.objects.len())
                .map(|i| i as CardId)
                .find(|&x| {
                    g.on_battlefield(x)
                        && g.object_view(x).is_token()
                        && g.object_view(x).face.is_some()
                })
                .unwrap();
            assert_eq!(g.object_view(copy).face.unwrap().back, back);
            assert_eq!(g.counter_count(copy, CounterKind::PlusOne), 6);
            assert!(g.has_subtype(copy, "Fractal"));
            assert!(!g.object_view(copy).card.is_token());
            if !back {
                transform(&mut g, copy);
                assert!(g.object_view(copy).face.unwrap().back);
                assert_eq!(g.counter_count(copy, CounterKind::PlusOne), 6);
                assert!(g.has_subtype(copy, "Fractal"));
                assert_eq!(g.object_view(copy).power, 6);
            }
        }
    }
}
#[test]
fn msh_faces_copied_permanent_spell_keeps_selected_face_and_delayed_copy_ability() {
    for p in [Player::P0, Player::P1] {
        for back in [false, true] {
            let mut g = setup(p);
            let id = named(&mut g, p, "Monica Rambeau", Zone::Hand);
            checked(
                &mut g,
                if back {
                    Action::AnnounceBack { card: id }
                } else {
                    Action::AnnounceCast { card: id }
                },
            );
            pay(&mut g);
            let sparks = named(&mut g, p, "Choreographed Sparks", Zone::Hand);
            cast(&mut g, sparks, Some(1), &[Target::Spell(id)]);
            settle_copy(&mut g);
            let copy = (0..g.state.objects.len())
                .map(|i| i as CardId)
                .find(|&x| {
                    g.on_battlefield(x)
                        && g.object_view(x).is_token()
                        && g.object_view(x).face.is_some()
                })
                .unwrap();
            if !back {
                transform(&mut g, copy);
            }
            assert_eq!(g.object_view(copy).name(), "Photon, Living Light");
            assert_eq!(g.object_view(copy).card.fixed_mana_value(), 5);
            assert!(g.object_view(copy).keywords.contains(Keywords::HASTE));
            assert_eq!(
                g.object_view(copy).face.unwrap().physical_card,
                Card::from_name("Monica Rambeau").unwrap()
            );
            g.state.step = Step::Main2;
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            settle(&mut g);
            assert_eq!(g.object_view(copy).zone, Zone::Outside);
            assert!(g.copy_recipe(g.object_ref(copy)).is_none());
        }
    }
}
#[test]
fn msh_faces_photon_back_is_not_an_independent_deck_card_and_timing_is_face_specific() {
    let front = Card::from_name("Monica Rambeau").unwrap();
    let back = front.back_face().unwrap();
    assert_eq!(back.face_parent(), Some(front));
    assert!(matches!(
        Game::new(Config {
            decks: [vec![back; 7], vec![Card::Forest; 7]],
            seed: 1,
            starting_player: Player::P0,
            deck_rules: DeckRules::Freeform
        }),
        Err(Error::InvalidDeck(_))
    ));
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Monica Rambeau", Zone::Hand);
        g.state.active = p.other();
        reject(&mut g, Action::AnnounceBack { card: id });
        reject(&mut g, Action::AnnounceCast { card: id });
        g.state.active = p;
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 2];
        reject(&mut g, Action::AnnounceBack { card: id });
        checked(&mut g, Action::AnnounceCast { card: id });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(id).name(), "Monica Rambeau");
    }
}

#[test]
fn msh_faces_selected_back_funds_with_hero_restricted_mana_and_rejects_artifact_only() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Monica Rambeau", Zone::Hand);
        let tower = named(&mut g, p, "Avengers Tower", Zone::Battlefield);
        g.state.players[p.index()].mana = [1, 0, 0, 1, 0, 2];
        checked(
            &mut g,
            Action::ActivateMana {
                card: tower,
                ability: 1,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [1, 0, 0, 0, 0, 0],
            },
        );
        checked(&mut g, Action::AnnounceBack { card: id });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(id).name(), "Photon, Living Light");
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let mut h = setup(p);
        let id = named(&mut h, p, "Monica Rambeau", Zone::Hand);
        let helper = named(&mut h, p, "Hydraulic Helper", Zone::Battlefield);
        h.state.objects[helper as usize].summoning_sick = false;
        h.state.players[p.index()].mana = [2, 0, 0, 1, 0, 1];
        checked(
            &mut h,
            Action::ActivateMana {
                card: helper,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 1, 0, 0, 0, 0],
            },
        );
        reject(&mut h, Action::AnnounceBack { card: id });
    }
}
#[test]
fn msh_faces_queued_front_prowess_survives_transform_and_back_abilities_can_be_removed() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = monica(&mut g, p, false);
        let growth = named(&mut g, p, "Giant Growth", Zone::Hand);
        cast(&mut g, growth, None, &[Target::Card(id)]);
        g.prepare_triggers();
        let transform_packet = g.activation_item(p, id, 0); // paid activation covered separately; isolate queued-definition lifetime.
        g.transform_source(&transform_packet);
        settle(&mut g);
        assert_eq!(g.object_view(id).power, 8); //4+3 spell+1 queued frontprowess.
        assert_eq!(g.state.cast_sources.iter().filter(|&&s| s == id).count(), 1);
        let ice = named(&mut g, p, "Frozen in Ice", Zone::Hand);
        cast(&mut g, ice, None, &[Target::Card(id)]);
        settle(&mut g);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let before = g.counter_count(bear, CounterKind::PlusOne);
        let growth = named(&mut g, p, "Giant Growth", Zone::Hand);
        cast(&mut g, growth, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), before);
    }
}
#[test]
fn msh_faces_back_countered_spell_and_bounce_restore_front_metadata_and_private_hand() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Monica Rambeau", Zone::Hand);
        checked(&mut g, Action::AnnounceBack { card: id });
        pay(&mut g);
        checked(&mut g, Action::Pass);
        let cancel = named(&mut g, p.other(), "Cancel", Zone::Hand);
        cast(&mut g, cancel, None, &[Target::Spell(id)]);
        settle(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert_eq!(g.object_view(id).name(), "Monica Rambeau");
        assert_eq!(g.object_view(id).card.cost(), ([1, 0, 0, 0, 0, 0], 2));
        g.move_object(id, Zone::Battlefield);
        assert!(!g.object_view(id).face.unwrap().back);
        g.state.priority = p;
        g.state.active = p;
        transform(&mut g, id);
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(id)]);
        settle(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Hand);
        assert_eq!(g.object_view(id).name(), "Monica Rambeau");
        let public = g.observe(p.other());
        assert_eq!(
            public.objects.iter().find(|o| o.id == id).unwrap().name(),
            "Monica Rambeau"
        );
        let secret = named(&mut g, p, "Monica Rambeau", Zone::Hand);
        assert!(g.observe(p.other()).objects.iter().all(|o| o.id != secret));
    }
}
#[test]
fn msh_faces_existing_foreign_copy_uses_new_controller_and_both_faces() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = monica(&mut g, p, false);
        g.set_counters(id, Counters::of(CounterKind::PlusOne, 2));
        let echo = named(&mut g, p, "Echocasting Symposium", Zone::Hand);
        cast(
            &mut g,
            echo,
            None,
            &[Target::Player(p.other()), Target::Card(id)],
        );
        settle(&mut g);
        let copy = (0..g.state.objects.len())
            .map(|i| i as CardId)
            .find(|&x| {
                g.on_battlefield(x)
                    && g.state.objects[x as usize].controller == p.other()
                    && g.object_view(x).face.is_some()
            })
            .unwrap();
        assert!(g.object_view(copy).is_token());
        assert_eq!(g.object_view(copy).power, 3);
        assert_eq!(g.counter_count(copy, CounterKind::PlusOne), 0);
        let reference = g.object_ref(copy);
        g.state.active = p.other();
        g.state.priority = p.other();
        g.state.step = Step::Main1;
        transform(&mut g, copy);
        assert_eq!(g.object_ref(copy), reference);
        assert_eq!(g.object_view(copy).controller, p.other());
        assert_eq!(g.object_view(copy).name(), "Photon, Living Light");
        let own = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let other = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p.other(), "Giant Growth", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Card(own)]);
        settle(&mut g);
        assert_eq!(g.counter_count(own, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(other, CounterKind::PlusOne), 0);
    }
}
#[test]
fn msh_faces_artifact_copy_exception_and_delayed_sacrifice_survive_transform() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = monica(&mut g, p, false);
        let spell = named(&mut g, p, "Molten Duplication", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Card(id)]);
        settle_copy(&mut g);
        let copy = (0..g.state.objects.len())
            .map(|i| i as CardId)
            .find(|&x| {
                g.on_battlefield(x)
                    && g.object_view(x).is_token()
                    && g.object_view(x).face.is_some()
            })
            .unwrap();
        assert!(g.object_view(copy).kinds & 16 != 0);
        transform(&mut g, copy);
        assert!(g.object_view(copy).kinds & 16 != 0);
        assert!(g.object_view(copy).keywords.contains(Keywords::HASTE));
        assert_eq!(
            g.copy_recipe(g.object_ref(copy)).unwrap().definition,
            Card::MshPhotonLivingLight
        );
        g.state.step = Step::Main2;
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        settle(&mut g);
        assert_eq!(g.object_view(copy).zone, Zone::Outside);
        assert!(g.state.face_changes.is_none());
    }
}
