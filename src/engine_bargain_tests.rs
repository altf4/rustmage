use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 10] = [
    "Archon's Glory",
    "Candy Grapple",
    "Farsight Ritual",
    "Hamlet Glutton",
    "High Fae Negotiator",
    "Ice Out",
    "Johann's Stopgap",
    "Kellan's Lightblades",
    "Tenacious Tomeseeker",
    "Troublemaker Ouphe",
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
        s.mana = [30; 6];
    }
    g
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let e = g.snapshot();
    assert_eq!(fork.snapshot(), e);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), e);
}
fn choose(g: &mut Game, target: Option<Target>) {
    let actions = g.legal_actions();
    let a = target
        .and_then(|t| {
            actions
                .iter()
                .find(|a| **a == Action::ChooseTarget { target: t })
        })
        .or_else(|| {
            actions.iter().find(|a| {
                matches!(
                    a,
                    Action::PayCosts { .. }
                        | Action::PutTrigger { .. }
                        | Action::ChooseTarget { .. }
                        | Action::Sacrifice { .. }
                        | Action::ChooseSearch { .. }
                        | Action::FinishSearch
                        | Action::ArrangeLibrary { .. }
                        | Action::SelectLibrary { .. }
                        | Action::FinishLibrarySelection
                        | Action::FinishTargets
                        | Action::Pass
                )
            })
        })
        .unwrap_or_else(|| panic!("no continuation {:?} {actions:?}", g.state.pending))
        .clone();
    replay(g, a);
}
fn settle(g: &mut Game, t: Option<Target>) {
    g.prepare_triggers();
    for _ in 0..200 {
        if g.outcome().is_some()
            || g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.announcing.is_none()
                && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, t);
    }
    panic!("unsettled");
}
fn stack(g: &mut Game, id: CardId, sacrifice: Option<CardId>, target: Option<Target>) {
    replay(g, Action::AnnounceCast { card: id });
    assert!(matches!(g.state.pending, Pending::Bargain { .. }));
    let reference = sacrifice.map(|id| g.object_ref(id));
    replay(
        g,
        Action::ChooseBargain {
            sacrifice: reference,
        },
    );
    while g.state.announcing.is_some() {
        choose(g, target);
    }
    assert_eq!(
        g.state.stack.last().unwrap().cast.unwrap().bargain,
        Some(sacrifice.is_some())
    );
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let revision = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), revision);
}
#[test]
fn complete_bodies_both_branches_both_controllers() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            for (i, n) in NAMES.iter().enumerate() {
                let mut g = setup(p);
                let food = named(&mut g, p, "Candy Trail", Zone::Battlefield);
                let target = named(&mut g, p.other(), "Hamlet Glutton", Zone::Battlefield);
                let grave = named(&mut g, p, "Shock", Zone::Graveyard);
                let artifact = named(&mut g, p.other(), "Candy Trail", Zone::Battlefield);
                let spell = named(&mut g, p.other(), "Shock", Zone::Hand);
                if i == 5 {
                    g.state.priority = p.other();
                    replay(&mut g, Action::AnnounceCast { card: spell });
                    while g.state.announcing.is_some() {
                        choose(&mut g, Some(Target::Player(p)));
                    }
                    g.state.priority = p;
                }
                if i == 7 {
                    g.state.attackers.push(AttackDeclaration {
                        card: target,
                        defender: Some(CombatDefender::Player(p)),
                        blocked: false,
                    });
                }
                let id = named(&mut g, p, n, Zone::Hand);
                let t = match i {
                    0 | 1 | 6 | 7 => Some(Target::Card(target)),
                    5 => Some(Target::Spell(spell)),
                    _ => None,
                };
                let before = g.state.players[p.index()].life;
                stack(&mut g, id, pay.then_some(food), t);
                assert_eq!(
                    g.state.objects[food as usize].zone,
                    if pay {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    }
                );
                settle(
                    &mut g,
                    Some(Target::Card(if i == 8 { grave } else { artifact })),
                );
                match i {
                    0 => {
                        assert_eq!(g.stats(target).power, 8);
                        assert_eq!(
                            g.keywords(target).contains(Keywords::FLYING)
                                && g.keywords(target).contains(Keywords::LIFELINK),
                            pay
                        );
                    }
                    1 => assert_eq!(g.stats(target).power, if pay { 1 } else { 3 }),
                    2 => assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0),
                    3 => assert_eq!(g.state.players[p.index()].life, before + 3),
                    4 => assert_eq!(
                        g.state.players[p.index()].life,
                        before + if pay { 3 } else { 0 }
                    ),
                    5 => assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard),
                    6 => {
                        assert_eq!(g.state.objects[target as usize].zone, Zone::Hand);
                        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
                    }
                    7 => {
                        assert_eq!(
                            g.state.objects[target as usize].zone,
                            if pay {
                                Zone::Graveyard
                            } else {
                                Zone::Battlefield
                            }
                        );
                        if !pay {
                            assert_eq!(g.state.objects[target as usize].damage, 3);
                        }
                    }
                    8 => assert_eq!(
                        g.state.objects[grave as usize].zone,
                        if pay { Zone::Hand } else { Zone::Graveyard }
                    ),
                    9 => assert_eq!(
                        g.state.objects[artifact as usize].zone,
                        if pay { Zone::Exile } else { Zone::Battlefield }
                    ),
                    _ => unreachable!(),
                }
            }
        }
    }
}
#[test]
fn exact_union_choice_before_targets_is_private_and_rejects_shortcuts() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let artifact = named(&mut g, p, "Candy Trail", Zone::Battlefield);
        let enchantment = named(&mut g, p, "Warleader's Call", Zone::Battlefield);
        let creature = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Candy Trail", Zone::Battlefield);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let id = named(&mut g, p, "Archon's Glory", Zone::Hand);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::Cast{card,..}|Action::BeginCast{card,..} if *card==id))
        );
        replay(&mut g, Action::AnnounceCast { card: id });
        let obs = g.observe(p);
        assert_eq!(obs.decision, "bargain");
        assert!(obs.payment_request.is_none());
        assert!(obs.announcing.unwrap().targets.is_empty());
        assert!(g.observe(p.other()).actions.is_empty());
        let actions = g.legal_actions();
        for n in [artifact, enchantment] {
            assert!(actions.contains(&Action::ChooseBargain {
                sacrifice: Some(g.object_ref(n))
            }));
        }
        for n in [creature, foe, land, id] {
            let r = g.object_ref(n);
            reject(&mut g, Action::ChooseBargain { sacrifice: Some(r) });
        }
        let old = g.object_ref(artifact);
        let s = g.snapshot();
        assert_eq!(
            g.apply(p.other(), Action::ChooseBargain { sacrifice: None }),
            Err(Error::WrongActor)
        );
        assert_eq!(g.snapshot(), s);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
        reject(&mut g, Action::ChooseKicker { pay: true });
        g.move_object(artifact, Zone::Exile);
        g.move_object(artifact, Zone::Battlefield);
        reject(
            &mut g,
            Action::ChooseBargain {
                sacrifice: Some(old),
            },
        );
    }
}
#[test]
fn bargain_discount_is_joint_with_reserved_mana_resource_and_locks_before_sacrifice() {
    for p in [Player::P0, Player::P1] {
        for (name, base, discount, color) in
            [("Hamlet Glutton", 5, 2, 4), ("Johann's Stopgap", 3, 2, 1)]
        {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [0; 6];
            g.state.players[p.index()].mana[color] = if color == 4 { 2 } else { 1 };
            g.state.players[p.index()].mana[5] = base - discount;
            let resource = named(&mut g, p, "Candy Trail", Zone::Battlefield);
            let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let id = named(&mut g, p, name, Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: id });
            assert!(
                !g.legal_actions()
                    .contains(&Action::ChooseBargain { sacrifice: None })
            );
            let r = g.object_ref(resource);
            replay(&mut g, Action::ChooseBargain { sacrifice: Some(r) });
            while matches!(g.state.pending, Pending::Targets { .. }) {
                choose(&mut g, Some(Target::Card(target)));
            }
            assert_eq!(
                g.state
                    .announcing
                    .as_ref()
                    .unwrap()
                    .total_cost
                    .unwrap()
                    .generic,
                base - discount
            );
            choose(&mut g, None);
            assert_eq!(g.state.objects[resource as usize].zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn sacrificed_target_fizzles_without_partial_growth_or_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Johann's Stopgap", Zone::Hand);
        let target = named(&mut g, p, "Candy Trail", Zone::Battlefield);
        stack(&mut g, id, Some(target), Some(Target::Card(target)));
        settle(&mut g, None);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        assert_eq!(g.state.objects[target as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn historical_entry_qualification_survives_departure_but_never_reentry() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "High Fae Negotiator",
            "Tenacious Tomeseeker",
            "Troublemaker Ouphe",
        ] {
            let mut g = setup(p);
            let resource = named(&mut g, p, "Candy Trail", Zone::Battlefield);
            let grave = named(&mut g, p, "Shock", Zone::Graveyard);
            let foe = named(&mut g, p.other(), "Candy Trail", Zone::Battlefield);
            let id = named(&mut g, p, name, Zone::Hand);
            stack(&mut g, id, Some(resource), None);
            replay(&mut g, Action::Pass);
            replay(&mut g, Action::Pass);
            let target = Some(Target::Card(if name == "Tenacious Tomeseeker" {
                grave
            } else {
                foe
            }));
            while g.state.announcing.is_some() {
                choose(&mut g, target);
            }
            assert_eq!(g.state.stack.len(), 1);
            let old = g.object_ref(id);
            assert_eq!(g.state.stack[0].object, old);
            g.move_object(id, Zone::Exile);
            g.move_object(id, Zone::Battlefield);
            assert_ne!(g.object_ref(id), old);
            settle(&mut g, target);
            assert!(g.state.stack.is_empty());
            assert!(g.state.pending_triggers.is_empty());
            if name == "High Fae Negotiator" {
                assert_eq!(g.state.players[p.index()].life, 23);
            } else {
                assert_eq!(
                    g.state.objects[if name == "Tenacious Tomeseeker" {
                        grave
                    } else {
                        foe
                    } as usize]
                        .zone,
                    if name == "Tenacious Tomeseeker" {
                        Zone::Hand
                    } else {
                        Zone::Exile
                    }
                );
            }
        }
    }
}
#[test]
fn actual_exile_permissions_preserve_bargain_and_wrong_origins_reject() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let resource = named(&mut g, p, "Candy Trail", Zone::Battlefield);
            let target = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let id = named(&mut g, p, "Archon's Glory", Zone::Library);
            g.exile_top_play(p, 1, true);
            replay(&mut g, Action::AnnounceExile { card: id });
            let r = pay.then(|| g.object_ref(resource));
            replay(&mut g, Action::ChooseBargain { sacrifice: r });
            while g.state.announcing.is_some() {
                choose(&mut g, Some(Target::Card(target)));
            }
            let cast = g.state.stack.last().unwrap().cast.unwrap();
            assert_eq!(cast.origin, Zone::Exile);
            assert_eq!(cast.bargain, Some(pay));
            assert_eq!(cast.kicker, None);
            assert_eq!(cast.offspring, None);
            assert!(g.state.exile_permissions.is_empty());
            settle(&mut g, None);
        }
    }
}
#[test]
fn real_treasure_reservation_is_not_spendable_twice_and_queries_are_pure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 1];
        let factory = Card::catalogue()
            .find(|c| c.token_info().is_some_and(|t| t.name == "Treasure Token"))
            .unwrap();
        let treasure = spawn(&mut g, p, factory, Zone::Battlefield);
        let r = g.object_ref(treasure);
        let id = named(&mut g, p, "Farsight Ritual", Zone::Hand);
        let before = g.snapshot();
        for _ in 0..3 {
            assert!(
                g.legal_actions()
                    .contains(&Action::AnnounceCast { card: id })
            );
        }
        assert_eq!(g.snapshot(), before);
        replay(&mut g, Action::AnnounceCast { card: id });
        assert!(
            g.legal_actions()
                .contains(&Action::ChooseBargain { sacrifice: None })
        );
        reject(&mut g, Action::ChooseBargain { sacrifice: Some(r) });
        let other = named(&mut g, p, "Candy Trail", Zone::Battlefield);
        let r = g.object_ref(other);
        replay(&mut g, Action::ChooseBargain { sacrifice: Some(r) });
        assert_eq!(g.state.objects[other as usize].zone, Zone::Battlefield);
        let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,production,..} if *card==treasure && production[1]==1)).unwrap();
        replay(&mut g, action);
        assert!(matches!(
            g.state.objects[treasure as usize].zone,
            Zone::Graveyard | Zone::Outside
        ));
        choose(&mut g, None);
        assert_eq!(g.state.objects[other as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn creature_tokens_qualify_independently_of_card_kinds_and_depart_normally() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let factory = Card::catalogue()
            .find(|c| {
                c.is_creature() && c.is_token() && c.kind_mask() & 48 == 0 && c.is_supported()
            })
            .unwrap();
        g.create_tokens(p, factory, 1, false);
        let token = (g.state.objects.len() - 1) as CardId;
        g.state.pending_triggers.clear();
        g.state.pending = Pending::Priority;
        let id = named(&mut g, p, "High Fae Negotiator", Zone::Hand);
        stack(&mut g, id, Some(token), None);
        settle(&mut g, None);
        assert_eq!(g.state.objects[token as usize].zone, Zone::Outside);
        assert_eq!(g.state.players[p.index()].life, 23);
    }
}
#[test]
fn farsight_has_one_private_deferred_four_or_eight_window_and_short_library() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let resource = named(&mut g, p, "Candy Trail", Zone::Battlefield);
            let id = named(&mut g, p, "Farsight Ritual", Zone::Hand);
            stack(&mut g, id, pay.then_some(resource), None);
            replay(&mut g, Action::Pass);
            replay(&mut g, Action::Pass);
            assert!(matches!(
                g.state.pending,
                Pending::EffectLibrarySelection { .. }
            ));
            let frame = g
                .state
                .resolving
                .as_ref()
                .unwrap()
                .selection
                .as_ref()
                .unwrap();
            assert_eq!(frame.remaining.len(), if pay { 8 } else { 4 });
            assert_eq!(frame.minimum, 2);
            assert_eq!(frame.maximum, 2);
            assert!(g.observe(p.other()).library_selection.is_none());
            assert!(g.observe(p.other()).actions.is_empty());
            let original = g.state.players[p.index()].library.clone();
            let first = frame.remaining[0];
            let second = frame.remaining[1];
            replay(&mut g, Action::SelectLibrary { card: first.card });
            assert_eq!(g.state.players[p.index()].library, original);
            assert_eq!(g.object_ref(first.card), first);
            replay(&mut g, Action::SelectLibrary { card: second.card });
            if matches!(g.state.pending, Pending::EffectLibrarySelection { .. }) {
                replay(&mut g, Action::FinishLibrarySelection);
            }
            settle(&mut g, None);
            assert_eq!(g.state.players[p.index()].library.len(), original.len() - 2);
            assert_eq!(g.state.objects[first.card as usize].zone, Zone::Hand);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            for count in 0..=2 {
                let mut g = setup(p);
                while g.state.players[p.index()].library.len() > count {
                    let n = *g.state.players[p.index()].library.last().unwrap();
                    g.move_object(n, Zone::Exile);
                }
                let resource = named(&mut g, p, "Candy Trail", Zone::Battlefield);
                let id = named(&mut g, p, "Farsight Ritual", Zone::Hand);
                stack(&mut g, id, pay.then_some(resource), None);
                settle(&mut g, None);
                assert!(g.state.players[p.index()].library.is_empty());
                assert!(g.outcome().is_none());
                assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            }
        }
    }
}
#[test]
fn late_capacity_error_restores_payment_sacrifice_and_bargain_entry_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let resource = named(&mut g, p, "Candy Trail", Zone::Battlefield);
        let id = named(&mut g, p, "High Fae Negotiator", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: id });
        let r = g.object_ref(resource);
        replay(&mut g, Action::ChooseBargain { sacrifice: Some(r) });
        g.state.players[p.index()].spells_cast_this_turn = u64::MAX;
        let saved = g.snapshot();
        let revision = g.revision();
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        assert!(matches!(g.apply(p, pay), Err(Error::CapacityExceeded(_))));
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), revision);
        assert_eq!(g.state.objects[resource as usize].zone, Zone::Battlefield);
        let mut g = setup(p);
        let resource = named(&mut g, p, "Candy Trail", Zone::Battlefield);
        let id = named(&mut g, p, "High Fae Negotiator", Zone::Hand);
        stack(&mut g, id, Some(resource), None);
        replay(&mut g, Action::Pass);
        g.state.next_stack_id = u64::MAX;
        let saved = g.snapshot();
        assert!(matches!(
            g.apply(g.actor().unwrap(), Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Stack);
    }
}
#[test]
fn bargain_etb_targets_preserve_full_zone_owner_and_kind_filters() {
    for p in [Player::P0, Player::P1] {
        for name in ["Tenacious Tomeseeker", "Troublemaker Ouphe"] {
            let mut g = setup(p);
            let resource = named(&mut g, p, "Candy Trail", Zone::Battlefield);
            let spell = named(&mut g, p, "Shock", Zone::Graveyard);
            let sorcery = named(&mut g, p, "Divination", Zone::Graveyard);
            let wrong_kind = named(&mut g, p, "Bear Cub", Zone::Graveyard);
            let foreign_spell = named(&mut g, p.other(), "Shock", Zone::Graveyard);
            let own = named(&mut g, p, "Warleader's Call", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Candy Trail", Zone::Battlefield);
            let foe_enchantment = named(&mut g, p.other(), "Warleader's Call", Zone::Battlefield);
            let foe_creature = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let id = named(&mut g, p, name, Zone::Hand);
            stack(&mut g, id, Some(resource), None);
            replay(&mut g, Action::Pass);
            replay(&mut g, Action::Pass);
            while !matches!(g.state.pending, Pending::Targets { .. }) {
                choose(&mut g, None);
            }
            let good = if name == "Tenacious Tomeseeker" {
                vec![spell, sorcery]
            } else {
                vec![foe, foe_enchantment]
            };
            for n in [wrong_kind, foreign_spell, own, foe_creature] {
                assert!(!g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(n)
                }));
            }
            for &n in &good {
                assert!(g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(n)
                }));
            }
            choose(&mut g, Some(Target::Card(good[0])));
            while g.state.announcing.is_some() {
                choose(&mut g, None);
            }
            g.move_object(good[0], Zone::Exile);
            g.move_object(
                good[0],
                if name == "Tenacious Tomeseeker" {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                },
            );
            settle(&mut g, None);
            assert_eq!(
                g.state.objects[good[0] as usize].zone,
                if name == "Tenacious Tomeseeker" {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}
#[test]
fn suppressors_block_bargained_entry_triggers_and_ordinary_entries_are_unqualified() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "High Fae Negotiator",
            "Tenacious Tomeseeker",
            "Troublemaker Ouphe",
        ] {
            let mut g = setup(p);
            named(&mut g, p.other(), "Torpor Orb", Zone::Battlefield);
            let resource = named(&mut g, p, "Candy Trail", Zone::Battlefield);
            let id = named(&mut g, p, name, Zone::Hand);
            stack(&mut g, id, Some(resource), None);
            settle(&mut g, None);
            assert_eq!(g.state.players[p.index()].life, 20);
            assert!(g.state.pending_triggers.is_empty());
            assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
            let mut g = setup(p);
            let id = named(&mut g, p, name, Zone::Graveyard);
            g.move_object(id, Zone::Battlefield);
            assert!(g.state.pending_triggers.is_empty());
            assert_eq!(g.state.players[p.index()].life, 20);
        }
    }
}
#[test]
fn archon_growth_and_keywords_expire_at_real_cleanup() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let resource = named(&mut g, p, "Candy Trail", Zone::Battlefield);
            let target = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let id = named(&mut g, p, "Archon's Glory", Zone::Hand);
            stack(
                &mut g,
                id,
                pay.then_some(resource),
                Some(Target::Card(target)),
            );
            settle(&mut g, None);
            assert_eq!(g.stats(target).power, 4);
            assert_eq!(g.keywords(target).contains(Keywords::LIFELINK), pay);
            g.finish_cleanup();
            assert_eq!(g.stats(target).power, 2);
            assert!(!g.keywords(target).contains(Keywords::LIFELINK));
            assert!(!g.keywords(target).contains(Keywords::FLYING));
        }
    }
}
#[test]
fn ice_discount_requires_two_blue_after_bargain_and_exact_live_spell() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let resource = named(&mut g, p, "Candy Trail", Zone::Battlefield);
        let target = named(&mut g, p.other(), "Shock", Zone::Hand);
        g.state.priority = p.other();
        replay(&mut g, Action::AnnounceCast { card: target });
        while g.state.announcing.is_some() {
            choose(&mut g, Some(Target::Player(p)));
        }
        g.state.priority = p;
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 0];
        let id = named(&mut g, p, "Ice Out", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: id });
        assert!(
            !g.legal_actions()
                .contains(&Action::ChooseBargain { sacrifice: None })
        );
        let r = g.object_ref(resource);
        replay(&mut g, Action::ChooseBargain { sacrifice: Some(r) });
        choose(&mut g, Some(Target::Spell(target)));
        assert_eq!(
            g.observe(p).payment_request.unwrap().required,
            [0, 2, 0, 0, 0, 0]
        );
        reject(
            &mut g,
            Action::PayCosts {
                payment: [0, 1, 0, 0, 0, 1],
            },
        );
        choose(&mut g, None);
        settle(&mut g, None);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.objects[target as usize].zone, Zone::Graveyard);
    }
}
