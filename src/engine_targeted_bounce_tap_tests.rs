use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 18] = [
    "Bigfin Bouncer",
    "Bounce Off",
    "Constrictor Sage",
    "Exclusion Mage",
    "Exosuit Savior",
    "Freeze in Place",
    "Grappling Kraken",
    "Iceridge Serpent",
    "Mental Modulation",
    "Mischievous Pup",
    "Peerless Ropemaster",
    "Proctor's Gaze",
    "Rimekin Recluse",
    "Ring of the Lucii",
    "Rowdy Snowballers",
    "Spider-Byte, Web Warden",
    "Stickytongue Sentinel",
    "Tidecaller Mentor",
];
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for s in &mut g.state.players {
        s.mana = [20; 6];
    }
    g
}
fn pt(g: &Game, id: CardId) -> (i128, i128) {
    let o = g.object_view(id);
    (o.power, o.toughness)
}
fn replay(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let e = g.snapshot();
    assert_eq!(f.snapshot(), e);
    g.restore(&s).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), e);
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}

fn choose(g: &mut Game, target: Option<CardId>) {
    let legal = g.legal_actions();
    let a = target
        .and_then(|id| {
            legal.iter().find(|a| {
                **a == Action::ChooseTarget {
                    target: Target::Card(id),
                }
            })
        })
        .or_else(|| {
            legal.iter().find(|a| {
                matches!(
                    a,
                    Action::ChooseSpellCost { .. }
                        | Action::FinishTargets
                        | Action::ChooseSearch { .. }
                )
            })
        })
        .or_else(|| {
            legal.iter().find(|a| {
                matches!(
                    a,
                    Action::ChooseTarget { .. }
                        | Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishSearch
                        | Action::ArrangeLibrary { .. }
                        | Action::ChooseOptional { accept: true }
                )
            })
        })
        .expect("fixture choice")
        .clone();
    replay(g, a);
}
fn settle(g: &mut Game, target: Option<CardId>) {
    g.prepare_triggers();
    for _ in 0..140 {
        if g.outcome().is_some() || g.state.stack.is_empty() && g.state.pending == Pending::Priority
        {
            return;
        }
        let p = g.actor().unwrap();
        let view = g.observe(p);
        if view.library_choice.is_some() || view.search_choice.is_some() {
            let other = g.observe(p.other());
            assert!(
                other.actions.is_empty()
                    && other.library_choice.is_none()
                    && other.search_choice.is_none()
            );
        }
        choose(g, target);
    }
    panic!("unsettled");
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(g, a);
}
fn cast_body(g: &mut Game, id: CardId, target: Option<CardId>) {
    replay(g, Action::AnnounceCast { card: id });
    for _ in 0..8 {
        if g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::PayCosts { .. }))
        {
            break;
        }
        choose(g, target);
    }
    pay(g);
    settle(g, target);
}
fn grave(g: &mut Game, p: Player, n: usize) -> Vec<CardId> {
    (0..n)
        .map(|_| named(g, p, "Forest", Zone::Graveyard))
        .collect()
}
fn put_trigger(g: &mut Game, target: Option<CardId>) {
    g.prepare_triggers();
    for _ in 0..8 {
        if g.state
            .stack
            .last()
            .is_some_and(|s| matches!(s.kind, StackKind::TriggeredAbility { .. }))
        {
            return;
        }
        choose(g, target);
    }
    panic!("trigger not stacked");
}
fn target_window(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..8 {
        if g.observe(g.actor().unwrap()).targeting_group.is_some() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PutTrigger { .. }))
            .expect("pending trigger target");
        replay(g, a);
    }
    panic!("target window");
}
#[test]
fn all_eighteen_full_bodies_cast_with_real_targets_complete_tails_and_intrinsic_rules() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[enemy as usize].tapped = true;
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let id = named(&mut g, p, name, Zone::Hand);
            if i == 17 {
                grave(&mut g, p, 7);
            }
            let target = if [4, 9, 16].contains(&i) { own } else { enemy };
            let library = g.state.players[p.index()].library.len();
            cast_body(&mut g, id, Some(target));
            if [0, 1, 3, 4, 7, 9, 10, 11, 12, 15, 16, 17].contains(&i) {
                assert_eq!(g.object_view(target).zone, Zone::Hand, "{name}");
                assert!(
                    g.observe(target_owner(&g, target).other())
                        .known_hand
                        .contains(&g.object_ref(target))
                );
            }
            if [2, 5, 14].contains(&i) {
                assert!(g.object_view(enemy).tapped);
                assert_eq!(
                    g.object_view(enemy).counters.stun,
                    if i == 5 { 3 } else { 1 }
                );
            }
            if i == 8 {
                assert!(g.object_view(enemy).tapped);
                assert_eq!(g.state.players[p.index()].library.len(), library - 1);
            }
            if i == 11 {
                assert_eq!(g.state.players[p.index()].library.len(), library - 1);
                assert!(g.state.objects.iter().any(|o| o.owner == p
                    && o.zone == Zone::Battlefield
                    && o.card.is_basic_land()
                    && o.tapped));
            }
            if i == 6 {
                let land = named(&mut g, p, "Forest", Zone::Hand);
                replay(&mut g, Action::PlayLand { card: land });
                settle(&mut g, Some(enemy));
                assert_eq!(g.object_view(enemy).counters.stun, 1);
            }
            if g.object_view(id).zone == Zone::Battlefield {
                let base = [
                    (3, 2),
                    (0, 0),
                    (4, 4),
                    (2, 2),
                    (2, 2),
                    (0, 0),
                    (5, 6),
                    (3, 3),
                    (0, 0),
                    (3, 1),
                    (4, 4),
                    (0, 0),
                    (3, 2),
                    (0, 0),
                    (2, 2),
                    (2, 2),
                    (3, 3),
                    (3, 3),
                ];
                if g.state.objects[id as usize].card.is_creature() {
                    assert_eq!(pt(&g, id), base[i], "{name}");
                }
                assert_eq!(g.keywords(id).contains(Keywords::FLYING), i == 4);
                assert_eq!(g.keywords(id).contains(Keywords::FLASH), i == 9);
                assert_eq!(g.keywords(id).contains(Keywords::REACH), i == 16);
                assert_eq!(g.keywords(id).contains(Keywords::MENACE), i == 17);
            }
        }
    }
}
fn target_owner(g: &Game, id: CardId) -> Player {
    g.state.objects[id as usize].owner
}
#[test]
fn opponent_bounce_trigger_cannot_target_own_creatures_lands_graves_or_hexproof_and_required_absence_is_safe()
 {
    for p in [Player::P0, Player::P1] {
        for i in [0, 3, 7] {
            let mut g = setup(p);
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
            let grave = named(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
            let hex = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[hex as usize].gained_keywords = Keywords::HEXPROOF;
            let id = named(&mut g, p, NAMES[i], Zone::Hand);
            cast_body(&mut g, id, None);
            assert!(g.state.stack.is_empty());
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let id2 = named(&mut g, p, NAMES[i], Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: id2 });
            pay(&mut g);
            replay(&mut g, Action::Pass);
            replay(&mut g, Action::Pass);
            target_window(&mut g);
            for target in [own, land, grave, hex] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
            }
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            settle(&mut g, Some(enemy));
            assert_eq!(g.object_view(enemy).zone, Zone::Hand);
        }
    }
}
#[test]
fn own_optional_bounces_exclude_the_source_and_allow_owned_land_or_zero_targets() {
    for p in [Player::P0, Player::P1] {
        for i in [4, 9, 16] {
            for select in [false, true] {
                let mut g = setup(p);
                let land = named(&mut g, p, "Forest", Zone::Battlefield);
                let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                let id = named(&mut g, p, NAMES[i], Zone::Hand);
                replay(&mut g, Action::AnnounceCast { card: id });
                pay(&mut g);
                replay(&mut g, Action::Pass);
                replay(&mut g, Action::Pass);
                target_window(&mut g);
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    },
                );
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
                if select {
                    replay(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(land),
                        },
                    );
                } else {
                    replay(&mut g, Action::FinishTargets);
                }
                settle(&mut g, None);
                assert_eq!(
                    g.object_view(land).zone,
                    if select {
                        Zone::Hand
                    } else {
                        Zone::Battlefield
                    }
                );
                assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            }
        }
    }
}
#[test]
fn bounce_off_accepts_vehicle_union_but_not_arbitrary_artifacts_or_lands_and_does_not_destroy_indestructible()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let vehicle = Card::catalogue()
            .find(|c| c.has_subtype("Vehicle") && !c.is_creature())
            .unwrap();
        let vehicle = spawn(&mut g, p.other(), vehicle, Zone::Battlefield);
        let artifact = named(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let spell = named(&mut g, p, NAMES[1], Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: spell });
        for id in [artifact, land] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(vehicle),
            },
        );
        pay(&mut g);
        settle(&mut g, None);
        assert_eq!(g.object_view(vehicle).zone, Zone::Hand);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[bear as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
        let spell = named(&mut g, p, NAMES[1], Zone::Hand);
        cast_body(&mut g, spell, Some(bear));
        assert_eq!(g.object_view(bear).zone, Zone::Hand);
    }
}
#[test]
fn tapped_target_revalidation_and_new_incarnation_invalidate_bounce_without_leaking_hand_knowledge()
{
    for p in [Player::P0, Player::P1] {
        for reset in [false, true] {
            let mut g = setup(p);
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[enemy as usize].tapped = true;
            let id = named(&mut g, p, NAMES[10], Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: id });
            pay(&mut g);
            replay(&mut g, Action::Pass);
            replay(&mut g, Action::Pass);
            put_trigger(&mut g, Some(enemy));
            if reset {
                g.move_object(enemy, Zone::Hand);
                g.move_object(enemy, Zone::Battlefield);
                g.state.objects[enemy as usize].tapped = true;
            } else {
                g.state.objects[enemy as usize].tapped = false;
            }
            settle(&mut g, None);
            assert_eq!(g.object_view(enemy).zone, Zone::Battlefield);
            assert!(!g.observe(p).known_hand.iter().any(|r| r.card == enemy));
        }
    }
}
#[test]
fn proctors_zero_target_still_searches_but_selected_illegal_target_fizzles_the_entire_spell() {
    for p in [Player::P0, Player::P1] {
        for select in [false, true] {
            let mut g = setup(p);
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = named(&mut g, p, NAMES[11], Zone::Hand);
            let library = g.state.players[p.index()].library.len();
            replay(&mut g, Action::AnnounceCast { card: spell });
            if select {
                replay(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
            } else {
                replay(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
            if select {
                g.move_object(enemy, Zone::Graveyard);
            }
            settle(&mut g, None);
            assert_eq!(
                g.state.players[p.index()].library.len(),
                library - usize::from(!select)
            );
            assert!(!g.state.land_played);
        }
    }
}
#[test]
fn tap_stun_already_tapped_targets_gain_counters_and_future_untaps_remove_one_without_untapping() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[enemy as usize].tapped = true;
        let spell = named(&mut g, p, NAMES[5], Zone::Hand);
        cast_body(&mut g, spell, Some(enemy));
        assert_eq!(g.object_view(enemy).counters.stun, 3);
        assert!(g.object_view(enemy).tapped);
        for left in [2, 1, 0] {
            g.state.objects[enemy as usize].untap();
            assert_eq!(g.object_view(enemy).counters.stun, left);
            assert!(g.object_view(enemy).tapped);
        }
        g.state.objects[enemy as usize].untap();
        assert!(!g.object_view(enemy).tapped);
    }
}
#[test]
fn stun_counter_overflow_restores_entire_resolution_including_tap_and_private_scry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[enemy as usize].counters.stun = u64::MAX;
        let spell = named(&mut g, p, NAMES[5], Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: spell });
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        pay(&mut g);
        replay(&mut g, Action::Pass);
        let saved = g.snapshot();
        let revision = g.revision();
        assert!(matches!(
            g.apply(g.actor().unwrap(), Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), revision);
        assert!(!g.object_view(enemy).tapped);
    }
}
#[test]
fn renew_exiles_its_grave_source_at_payment_and_resolves_after_source_departure_only_as_sorcery() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let id = named(&mut g, p, NAMES[2], Zone::Graveyard);
        let announce = Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.active = p.other();
        reject(&mut g, announce.clone());
        g.state.active = p;
        g.state.step = Step::Upkeep;
        reject(&mut g, announce.clone());
        g.state.step = Step::Main1;
        replay(&mut g, announce.clone());
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        pay(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Exile);
        settle(&mut g, None);
        assert!(g.object_view(enemy).tapped);
        assert_eq!(g.object_view(enemy).counters.stun, 1);
        reject(&mut g, announce);
    }
}
#[test]
fn tidecaller_intervening_threshold_is_checked_at_entry_and_again_before_resolution() {
    for p in [Player::P0, Player::P1] {
        for lose in [false, true] {
            let mut g = setup(p);
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let graves = grave(&mut g, p, 7);
            let id = named(&mut g, p, NAMES[17], Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: id });
            pay(&mut g);
            replay(&mut g, Action::Pass);
            replay(&mut g, Action::Pass);
            put_trigger(&mut g, Some(enemy));
            if lose {
                g.move_object(graves[0], Zone::Exile);
            }
            settle(&mut g, None);
            assert_eq!(
                g.object_view(enemy).zone,
                if lose { Zone::Battlefield } else { Zone::Hand }
            );
        }
        let mut g = setup(p);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        grave(&mut g, p, 6);
        let id = named(&mut g, p, NAMES[17], Zone::Hand);
        cast_body(&mut g, id, Some(enemy));
        assert_eq!(g.object_view(enemy).zone, Zone::Battlefield);
    }
}
#[test]
fn ring_mana_is_immediate_but_targeted_tap_pays_life_and_mana_and_uses_stack() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ring = named(&mut g, p, NAMES[13], Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::ActivateMana{card,..}if *card==ring))
            .unwrap();
        replay(&mut g, a);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 2]);
        assert!(g.state.stack.is_empty());
        assert!(g.object_view(ring).tapped);
        g.state.objects[ring as usize].tapped = false;
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: ring,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        let life = g.state.players[p.index()].life;
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].life, life - 1);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.object_view(ring).tapped);
        assert!(!g.object_view(enemy).tapped);
        settle(&mut g, None);
        assert!(g.object_view(enemy).tapped);
    }
}
#[test]
fn mental_modulation_cost_reduction_and_already_tapped_draw_follow_turn_and_target_legality() {
    for p in [Player::P0, Player::P1] {
        for own_turn in [false, true] {
            for invalidate in [false, true] {
                let mut g = setup(p);
                g.state.active = if own_turn { p } else { p.other() };
                let enemy = named(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
                g.state.objects[enemy as usize].tapped = true;
                let land = named(&mut g, p, "Forest", Zone::Battlefield);
                let spell = named(&mut g, p, NAMES[8], Zone::Hand);
                let library = g.state.players[p.index()].library.len();
                replay(&mut g, Action::AnnounceCast { card: spell });
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(land),
                    },
                );
                replay(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
                assert_eq!(
                    g.observe(p).payment_request.unwrap().generic,
                    u32::from(!own_turn)
                );
                pay(&mut g);
                if invalidate {
                    g.move_object(enemy, Zone::Hand);
                    g.move_object(enemy, Zone::Battlefield);
                }
                settle(&mut g, None);
                assert_eq!(
                    g.state.players[p.index()].library.len(),
                    library - usize::from(!invalidate)
                );
                assert_eq!(g.object_view(enemy).tapped, !invalidate);
            }
        }
    }
}
#[test]
fn targeted_stun_triggers_ward_and_declining_counters_the_whole_spell_including_scry() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            let spider = named(&mut g, p.other(), "Skyward Spider", Zone::Battlefield);
            let spell = named(&mut g, p, NAMES[5], Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: spell });
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(spider),
                },
            );
            pay(&mut g);
            for _ in 0..12 {
                if g.observe(p).resolution_payment.is_some() {
                    break;
                }
                choose(&mut g, Some(spider));
            }
            assert_eq!(g.observe(p).resolution_payment.unwrap().cost.generic, 2);
            if accept {
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayResolutionMana { .. }))
                    .unwrap();
                replay(&mut g, a);
            } else {
                replay(&mut g, Action::DeclineResolutionMana);
            }
            settle(&mut g, None);
            assert_eq!(g.object_view(spider).tapped, accept);
            assert_eq!(
                g.object_view(spider).counters.stun,
                if accept { 3 } else { 0 }
            );
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn bounce_detaches_equipment_sends_aura_to_grave_and_ceases_tokens_without_private_hand_leak() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let equipment = named(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        let aura = named(&mut g, p.other(), "Frantic Strength", Zone::Battlefield);
        let reference = g.object_ref(bear);
        for a in [equipment, aura] {
            g.state.objects[a as usize].attached_to = Some(reference);
        }
        let spell = named(&mut g, p, NAMES[1], Zone::Hand);
        cast_body(&mut g, spell, Some(bear));
        assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
        assert_eq!(g.object_view(equipment).zone, Zone::Battlefield);
        assert_eq!(g.object_view(equipment).attached_to, None);
        assert!(g.observe(p).known_hand.contains(&g.object_ref(bear)));
        let token = Card::catalogue()
            .find(|c| c.token_info().is_some() && c.is_creature())
            .unwrap();
        let token = spawn(&mut g, p.other(), token, Zone::Battlefield);
        let spell = named(&mut g, p, NAMES[1], Zone::Hand);
        cast_body(&mut g, spell, Some(token));
        assert_eq!(g.state.objects[token as usize].zone, Zone::Outside);
        assert!(!g.observe(p).known_hand.iter().any(|r| r.card == token));
        assert!(!g.observe(p).objects.iter().any(|o| o.id == token));
    }
}
#[test]
fn ring_funding_is_pure_exact_and_never_counts_its_targeted_tap_program_as_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, NAMES[13], Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        let saved = g.snapshot();
        let revision = g.revision();
        let goal = super::funding::FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 2,
            reserve: None,
            source: None,
        };
        assert!(g.can_fund(goal));
        assert!(!g.can_fund(super::funding::FundingGoal { generic: 3, ..goal }));
        assert!(!g.can_fund(super::funding::FundingGoal {
            generic: 0,
            required: [0, 1, 0, 0, 0, 0],
            ..goal
        }));
        assert!(g.can_fund(super::funding::FundingGoal {
            generic: 0,
            required: [0, 0, 0, 0, 0, 2],
            ..goal
        }));
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), revision);
    }
}
#[test]
fn grappling_landfall_ignores_opponent_land_and_owned_land_uses_one_targeted_stun_program() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p.other());
        named(&mut g, p, NAMES[6], Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let land = named(&mut g, p.other(), "Forest", Zone::Hand);
        replay(&mut g, Action::PlayLand { card: land });
        settle(&mut g, Some(enemy));
        assert_eq!(g.object_view(enemy).counters.stun, 0);
        assert!(!g.object_view(enemy).tapped);
        g.state.active = p;
        g.state.priority = p;
        g.state.land_played = false;
        let land = named(&mut g, p, "Forest", Zone::Hand);
        replay(&mut g, Action::PlayLand { card: land });
        settle(&mut g, Some(enemy));
        assert_eq!(g.object_view(enemy).counters.stun, 1);
        assert!(g.object_view(enemy).tapped);
    }
}
