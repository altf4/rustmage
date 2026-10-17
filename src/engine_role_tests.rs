//! Predefined Roles, frozen simultaneous events, paid spells and owned replay.
use super::tests::{scenario, spawn};
use super::*;
use crate::cards::RoleKind;
fn factory(kind: RoleKind) -> Card {
    Card::catalogue().find(|c| c.role() == Some(kind)).unwrap()
}
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
fn act(g: &mut Game, a: Action) {
    let p = g.actor().unwrap();
    let old = g.snapshot();
    let rev = g.revision();
    let i = g
        .legal_actions()
        .iter()
        .position(|x| *x == a)
        .unwrap_or_else(|| panic!("illegal {a:?}: {:?}", g.observe(p)));
    assert_eq!(g.apply(p.other(), a.clone()), Err(Error::WrongActor));
    assert_eq!(g.snapshot(), old);
    let mut child = {
        let parent = g.clone();
        parent.clone()
    };
    child.apply_index(p, rev, i).unwrap();
    g.apply(p, a.clone()).unwrap();
    assert_eq!(g.snapshot(), child.snapshot());
    for v in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(v)).unwrap(),
            serde_json::to_value(child.observe(v)).unwrap()
        );
    }
    let done = g.snapshot();
    g.restore(&old).unwrap();
    assert_eq!(g.apply_index(p, rev, i), Err(Error::StaleRevision));
    g.apply(p, a).unwrap();
    assert_eq!(g.snapshot(), done);
}
fn settle(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..180 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.announcing.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let a = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets))
                .or_else(|| {
                    g.legal_actions()
                        .into_iter()
                        .find(|a| !matches!(a, Action::Concede))
                })
                .unwrap()
        };
        act(g, a);
    }
    panic!("unsettled {:?}", g.observe(g.actor().unwrap()));
}
fn attach(
    g: &mut Game,
    p: Player,
    kind: RoleKind,
    target: ObjectRef,
    amount: u128,
) -> Vec<ObjectRef> {
    let start = g.state.roles.as_ref().map_or(0, |r| r.attached.len());
    g.create_attached_role_groups(
        p,
        &[TokenGroupView {
            token: factory(kind),
            amount,
            tapped: false,
        }],
        target,
    );
    g.state.roles.as_ref().map_or_else(Vec::new, |r| {
        r.attached[start..].iter().map(|v| v.aura).collect()
    })
}
#[test]
fn all_six_factories_have_real_aura_characteristics_and_correct_layered_bodies_both_seats() {
    for p in [Player::P0, Player::P1] {
        for kind in [
            RoleKind::Cursed,
            RoleKind::Monster,
            RoleKind::Royal,
            RoleKind::Sorcerer,
            RoleKind::Wicked,
            RoleKind::YoungHero,
        ] {
            let mut g = setup(p);
            let c = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            g.state.objects[c as usize].counters.plus_one = 2;
            let r = g.object_ref(c);
            let token = factory(kind);
            assert_eq!(token.kind_mask(), 32);
            assert_eq!(token.info().type_line, "Enchantment — Aura Role");
            assert_eq!(token.token_info().unwrap().colors, 0);
            let a = attach(&mut g, p, kind, r, 1)[0];
            assert_eq!(g.object_view(a.card).attached_to, Some(r));
            let expected = match kind {
                RoleKind::Cursed => 3,
                RoleKind::YoungHero => 5,
                _ => 6,
            };
            assert_eq!(
                g.stats(c),
                Stats {
                    power: expected,
                    toughness: expected
                }
            );
            assert_eq!(
                g.keywords(c).contains(Keywords::TRAMPLE),
                kind == RoleKind::Monster
            );
            g.state_based_actions();
            assert_eq!(g.object_view(a.card).zone, Zone::Battlefield);
        }
    }
}
#[test]
fn latest_role_per_controller_and_opposite_controller_roles_coexist() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let r = g.object_ref(c);
        let old = attach(&mut g, p, RoleKind::Monster, r, 1)[0];
        let foreign = attach(&mut g, p.other(), RoleKind::Cursed, r, 1)[0];
        let newest = attach(&mut g, p, RoleKind::Royal, r, 1)[0];
        assert_eq!(
            g.stats(c),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        g.state_based_actions();
        assert_ne!(g.object_view(old.card).zone, Zone::Battlefield);
        assert_eq!(g.object_view(foreign.card).zone, Zone::Battlefield);
        assert_eq!(g.object_view(newest.card).zone, Zone::Battlefield);
        assert_eq!(
            g.stats(c),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(!g.keywords(c).contains(Keywords::TRAMPLE));
        assert_eq!(g.observe(p).role_timestamps.len(), 2);
    }
}
#[test]
fn simultaneous_newest_choice_preserves_owned_batch_and_rejects_illegal_actor_or_reference() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let r = g.object_ref(c);
        let old = attach(&mut g, p, RoleKind::Monster, r, 1)[0];
        let a = attach(&mut g, p, RoleKind::Wicked, r, 2);
        g.state_based_actions();
        assert_eq!(g.state.pending, Pending::Role { player: p });
        assert_eq!(g.observe(p.other()).role_choices[0].objects, a);
        assert_eq!(g.object_view(old.card).zone, Zone::Battlefield);
        let saved = g.snapshot();
        assert_eq!(
            g.apply(p, Action::KeepRole { object: old }),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), saved);
        act(&mut g, Action::KeepRole { object: a[1] });
        assert_eq!(g.object_view(a[1].card).zone, Zone::Battlefield);
        assert_ne!(g.object_view(a[0].card).zone, Zone::Battlefield);
        assert_ne!(g.object_view(old.card).zone, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 19);
    }
}
#[test]
fn apnap_role_and_legend_choices_commit_with_simultaneous_lethal_and_detached_wicked_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let r = g.object_ref(c);
        let a = attach(&mut g, p, RoleKind::Wicked, r, 2);
        let legend = named(
            &mut g,
            p.other(),
            "Yargle, Glutton of Urborg",
            Zone::Battlefield,
        );
        named(
            &mut g,
            p.other(),
            "Yargle, Glutton of Urborg",
            Zone::Battlefield,
        );
        let doomed = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[doomed as usize].damage = 2;
        g.state_based_actions();
        assert_eq!(g.actor(), Some(p));
        assert_eq!(g.object_view(doomed).zone, Zone::Battlefield);
        act(&mut g, Action::KeepRole { object: a[0] });
        assert_eq!(g.actor(), Some(p.other()));
        assert_eq!(g.object_view(doomed).zone, Zone::Battlefield);
        let object = g.object_ref(legend);
        act(&mut g, Action::KeepLegend { object });
        assert_eq!(g.object_view(doomed).zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 19);
    }
}
#[test]
fn real_doubling_season_replacement_reaches_role_choice_and_stale_recipient_creates_nothing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let r = g.object_ref(c);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        g.stage_token_creation(
            p,
            factory(RoleKind::Monster),
            1,
            false,
            replacements::TokenCompletion::RoleAttach(r),
        );
        g.advance_replacements(false);
        g.state_based_actions();
        settle(&mut g);
        assert_eq!(g.observe(p).role_timestamps.len(), 1);
        assert!(g.keywords(c).contains(Keywords::TRAMPLE));
        g.move_object(c, Zone::Hand);
        let n = g.state.objects.len();
        attach(&mut g, p, RoleKind::Royal, r, 1);
        assert_eq!(g.state.objects.len(), n);
        g.state_based_actions();
        assert!(g.observe(p).role_timestamps.is_empty());
    }
}
#[test]
fn four_complete_adventure_spells_resolve_paid_and_release_exact_normal_face_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (name, kind) in [
            ("Besotted Knight // Betroth the Beast", RoleKind::Royal),
            ("Conceited Witch // Price of Beauty", RoleKind::Wicked),
            ("Ferocious Werefox // Guard Change", RoleKind::Monster),
            ("Vantress Transmuter // Croaking Curse", RoleKind::Cursed),
        ] {
            let mut g = setup(p);
            let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let id = named(&mut g, p, name, Zone::Hand);
            let reference = g.object_ref(id);
            let mana: u32 = g.state.players[p.index()].mana.iter().sum();
            act(&mut g, Action::AnnounceAdventure { card: id });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(c),
                },
            );
            settle(&mut g);
            assert!(g.state.players[p.index()].mana.iter().sum::<u32>() < mana);
            assert_eq!(g.object_view(id).zone, Zone::Exile);
            assert_ne!(g.object_ref(id), reference);
            assert!(g.can_play_adventure(p, id));
            assert_eq!(g.observe(p).role_timestamps.len(), 1);
            let aura = g.observe(p).role_timestamps[0].aura;
            assert_eq!(g.object_view(aura.card).card.role(), Some(kind));
            if kind == RoleKind::Cursed {
                assert!(g.object_view(c).tapped);
            }
            g.state.priority = p;
            act(&mut g, Action::AnnounceExile { card: id });
            settle(&mut g);
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert_eq!(g.object_view(id).controller, p);
        }
    }
}

fn finish_announcement(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..80 {
        if g.state.pending == Pending::Priority
            && g.state.announcing.is_none()
            && g.state.ordering_triggers.is_empty()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets))
            .or_else(|| {
                g.legal_actions()
                    .into_iter()
                    .find(|a| !matches!(a, Action::Concede))
            })
            .unwrap();
        act(g, a);
    }
    panic!("announcement");
}
#[test]
fn royal_grants_recipient_controlled_ward_with_real_pay_decline_and_detached_source() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            for foreign_aura in [false, true] {
                let mut g = setup(p);
                let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let r = g.object_ref(c);
                let aura = attach(
                    &mut g,
                    if foreign_aura { p.other() } else { p },
                    RoleKind::Royal,
                    r,
                    1,
                )[0];
                let spell = named(&mut g, p.other(), "Murder", Zone::Hand);
                g.state.priority = p.other();
                act(&mut g, Action::AnnounceCast { card: spell });
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(c),
                    },
                );
                finish_announcement(&mut g);
                let packet = g.state.stack.last().unwrap();
                assert_eq!(packet.controller, p);
                assert_eq!(packet.card, c);
                assert!(packet.attachment_trigger.is_some());
                g.move_object(aura.card, Zone::Graveyard);
                for _ in 0..20 {
                    if g.observe(p.other()).resolution_payment.is_some() {
                        break;
                    }
                    act(&mut g, Action::Pass);
                }
                assert_eq!(g.actor(), Some(p.other()));
                assert_eq!(g.observe(p).resolution_payment.unwrap().cost.generic, 1);
                if pay {
                    let a = g
                        .legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::PayResolutionMana { .. }))
                        .unwrap();
                    act(&mut g, a);
                } else {
                    act(&mut g, Action::DeclineResolutionMana);
                }
                settle(&mut g);
                assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
                assert_eq!(
                    g.object_view(c).zone,
                    if pay {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    }
                );
            }
        }
    }
}
#[test]
fn granted_attack_uses_recipient_controller_private_scry_and_intervening_if_recheck() {
    for p in [Player::P0, Player::P1] {
        for kind in [RoleKind::Sorcerer, RoleKind::YoungHero] {
            for change in [false, true] {
                let mut g = setup(p);
                let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let r = g.object_ref(c);
                let aura = attach(&mut g, p.other(), kind, r, 1)[0];
                g.state.step = Step::Attackers;
                g.state.pending = Pending::Attackers;
                act(&mut g, Action::Attack { card: c });
                act(&mut g, Action::FinishAttackers);
                finish_announcement(&mut g);
                let packet = g.state.stack.last().unwrap();
                assert_eq!(packet.controller, p);
                assert_eq!(packet.card, c);
                // Leaving the grantor cannot erase a granted ability that has triggered.
                g.move_object(aura.card, Zone::Graveyard);
                if kind == RoleKind::YoungHero {
                    if change {
                        g.state.objects[c as usize].growth.toughness = 2;
                    }
                    settle(&mut g);
                    assert_eq!(
                        g.object_view(c).counters.plus_one,
                        if change { 0 } else { 1 }
                    );
                } else {
                    for _ in 0..30 {
                        if g.observe(p).library_choice.is_some() {
                            break;
                        }
                        act(&mut g, Action::Pass);
                    }
                    assert_eq!(g.actor(), Some(p));
                    assert!(g.observe(p).library_choice.is_some());
                    assert!(g.observe(p.other()).library_choice.is_none());
                    settle(&mut g);
                }
            }
        }
    }
}

fn settle_targeted(g: &mut Game, target: CardId) {
    g.prepare_triggers();
    for _ in 0..180 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.announcing.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let a = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .iter()
                .find(|a| **a == Action::ChooseMode { mode: 2 })
                .or_else(|| {
                    actions.iter().find(|a| {
                        **a == Action::ChooseTarget {
                            target: Target::Card(target),
                        }
                    })
                })
                .or_else(|| actions.iter().find(|a| matches!(a, Action::FinishTargets)))
                .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                .unwrap()
                .clone()
        };
        act(g, a);
    }
    panic!("unsettled consumer {:?}", g.observe(g.actor().unwrap()));
}
#[test]
fn eight_whole_ordinary_consumers_use_real_paid_casts_triggers_and_activation_costs_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (name, kind) in [
            ("Charmed Clothier", RoleKind::Royal),
            ("Charming Scoundrel", RoleKind::Wicked),
            ("Embereth Veteran", RoleKind::YoungHero),
            ("Living Lectern", RoleKind::Sorcerer),
            ("Redtooth Genealogist", RoleKind::Royal),
            ("Royal Treatment", RoleKind::Royal),
            ("Spiteful Hexmage", RoleKind::Cursed),
            ("Splashy Spellcaster", RoleKind::Sorcerer),
        ] {
            let mut g = setup(p);
            let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let id = named(&mut g, p, name, Zone::Hand);
            let before: u32 = g.state.players[p.index()].mana.iter().sum();
            act(&mut g, Action::AnnounceCast { card: id });
            settle_targeted(&mut g, c);
            if matches!(name, "Embereth Veteran" | "Living Lectern") {
                let hand = g.observe(p).players[p.index()].hand_count;
                g.state.priority = p;
                act(
                    &mut g,
                    Action::AnnounceActivate {
                        card: id,
                        ability: 0,
                        counter_payment: Counters::default(),
                    },
                );
                settle_targeted(&mut g, c);
                assert_eq!(g.object_view(id).zone, Zone::Graveyard);
                if name == "Living Lectern" {
                    assert_eq!(g.observe(p).players[p.index()].hand_count, hand + 1);
                }
            } else if name == "Splashy Spellcaster" {
                let spell = named(&mut g, p, "Quick Study", Zone::Hand);
                g.state.priority = p;
                act(&mut g, Action::AnnounceCast { card: spell });
                settle_targeted(&mut g, c);
                assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            }
            assert!(g.state.players[p.index()].mana.iter().sum::<u32>() < before);
            assert_eq!(g.observe(p).role_timestamps.len(), 1, "{name}");
            let aura = g.observe(p).role_timestamps[0].aura;
            assert_eq!(g.object_view(aura.card).card.role(), Some(kind));
            assert_eq!(g.object_view(aura.card).attached_to, Some(g.object_ref(c)));
            if name == "Royal Treatment" {
                assert!(g.keywords(c).contains(Keywords::HEXPROOF));
            }
        }
    }
}

#[test]
fn paid_syr_armont_enchanted_lord_uses_creature_control_and_any_exact_aura_both_seats() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ally = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let ally_ref = g.object_ref(ally);
        attach(&mut g, p.other(), RoleKind::YoungHero, ally_ref, 1);
        let foe_ref = g.object_ref(foe);
        let foreign_aura = attach(&mut g, p, RoleKind::Royal, foe_ref, 1)[0];
        let source = named(&mut g, p, "Syr Armont, the Redeemer", Zone::Hand);
        let before = g.state.players[p.index()].mana;
        act(&mut g, Action::AnnounceCast { card: source });
        for _ in 0..100 {
            if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.announcing.is_none()
                && g.state.pending_triggers.is_empty()
            {
                break;
            }
            let actions = g.legal_actions();
            let a = if g.state.pending == Pending::Priority {
                Action::Pass
            } else {
                actions
                    .iter()
                    .find(|a| {
                        **a == Action::ChooseTarget {
                            target: Target::Card(ally),
                        }
                    })
                    .or_else(|| actions.iter().find(|a| matches!(a, Action::FinishTargets)))
                    .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                    .unwrap()
                    .clone()
            };
            act(&mut g, a);
        }
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert_eq!(
            before.iter().sum::<u32>() - g.state.players[p.index()].mana.iter().sum::<u32>(),
            5
        );
        assert!(before[0] > g.state.players[p.index()].mana[0]);
        assert!(before[4] > g.state.players[p.index()].mana[4]);
        assert!(g.is_enchanted(ally));
        assert!(g.is_enchanted(foe));
        assert!(!g.is_enchanted(source));
        assert_eq!(g.stats(ally).power, 4); // Bear + Monster + lord, foreign Young Hero has no flat bonus.
        assert_eq!(g.stats(foe).power, 3); // Opponent's creature gets no lord bonus despite our Aura.
        assert_eq!(g.stats(source).power, 4);
        g.state.objects[source as usize].controller = p.other();
        assert_eq!(g.stats(ally).power, 3);
        assert_eq!(g.stats(foe).power, 4);
        g.move_object(foreign_aura.card, Zone::Graveyard);
        assert!(!g.is_enchanted(foe));
        assert_eq!(g.stats(foe).power, 2);
    }
}

#[test]
fn paid_tale_for_ages_buffs_only_live_enchanted_creatures_and_not_equipment_both_seats() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ally = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let equipment_only = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let recipe = Card::catalogue()
            .find(|card| card.is_supported() && card.equipment().is_some())
            .unwrap();
        let equipment = spawn(&mut g, p, recipe, Zone::Battlefield);
        g.state.objects[equipment as usize].attached_to = Some(g.object_ref(equipment_only));
        let ally_ref = g.object_ref(ally);
        let aura = attach(&mut g, p.other(), RoleKind::Cursed, ally_ref, 1)[0];
        let source = named(&mut g, p, "A Tale for the Ages", Zone::Hand);
        let before = g.state.players[p.index()].mana.iter().sum::<u32>();
        act(&mut g, Action::AnnounceCast { card: source });
        settle(&mut g);
        assert_eq!(
            before - g.state.players[p.index()].mana.iter().sum::<u32>(),
            2
        );
        assert_eq!(g.stats(ally).power, 3); // Cursed 1/1 plus enchanted +2/+2.
        assert!(!g.is_enchanted(equipment_only));
        let equipped_power = g.stats(equipment_only).power;
        g.move_object(aura.card, Zone::Graveyard);
        assert!(!g.is_enchanted(ally));
        assert_eq!(g.stats(ally).power, 2);
        assert_eq!(g.stats(equipment_only).power, equipped_power);
    }
}

#[test]
fn paid_return_triumphant_attaches_to_actual_new_object_and_stale_targets_do_not_rebind() {
    for p in [Player::P0, Player::P1] {
        for stale in [false, true] {
            let mut g = setup(p);
            let target = named(&mut g, p, "Bear Cub", Zone::Graveyard);
            let before = g.object_ref(target);
            let source = named(&mut g, p, "Return Triumphant", Zone::Hand);
            let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
            act(&mut g, Action::AnnounceCast { card: source });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            let saved = g.snapshot();
            assert_eq!(
                g.apply(p, Action::PayCosts { payment: [0; 6] }),
                Err(Error::IllegalAction)
            );
            assert_eq!(g.snapshot(), saved);
            let pay = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            act(&mut g, pay);
            assert_eq!(
                g.state.stack.last().unwrap().targets[0].object,
                Some(before)
            );
            if stale {
                g.move_object(target, Zone::Hand);
                g.move_object(target, Zone::Graveyard);
            }
            settle(&mut g);
            assert_eq!(
                mana - g.state.players[p.index()].mana.iter().sum::<u32>(),
                2
            );
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            assert_eq!(
                g.object_view(target).zone,
                if stale {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            let roles: Vec<_> = g
                .state
                .roles
                .as_ref()
                .map_or_else(Vec::new, |r| r.attached.iter().collect());
            if stale {
                assert!(roles.is_empty());
            } else {
                assert_ne!(g.object_ref(target), before);
                assert_eq!(roles.len(), 1);
                assert_eq!(roles[0].recipient, g.object_ref(target));
                assert_ne!(roles[0].recipient, before);
                assert_eq!(
                    g.state.objects[roles[0].aura.card as usize].card.role(),
                    Some(RoleKind::YoungHero)
                );
                assert_eq!(g.stats(target).power, 2);
            }
        }
    }
}

#[test]
fn role_move_completion_requires_actual_successful_battlefield_destination() {
    for p in [Player::P0, Player::P1] {
        for skipped in [false, true] {
            let mut g = setup(p);
            let target = named(&mut g, p, "Bear Cub", Zone::Graveyard);
            let before = g.object_ref(target);
            let mut movement = g.entry_move(target, Zone::Battlefield, false);
            movement.role_after = Some((factory(RoleKind::YoungHero), p));
            movement.skipped = skipped;
            if !skipped {
                movement.destination = Zone::Exile;
            }
            g.stage_entries(vec![movement], None);
            assert!(g.state.roles.as_ref().is_none_or(|r| r.attached.is_empty()));
            assert_eq!(
                g.object_view(target).zone,
                if skipped {
                    Zone::Graveyard
                } else {
                    Zone::Exile
                }
            );
            if skipped {
                assert_eq!(g.object_ref(target), before);
            }
        }
    }
}

#[test]
fn successful_return_role_creator_is_independent_of_returned_creature_controller() {
    for creator in [Player::P0, Player::P1] {
        let mut g = setup(creator);
        let owner = creator.other();
        let target = named(&mut g, owner, "Bear Cub", Zone::Graveyard);
        let mut movement = g.entry_move(target, Zone::Battlefield, false);
        movement.controller = owner;
        movement.role_after = Some((factory(RoleKind::Wicked), creator));
        g.stage_entries(vec![movement], None);
        g.advance_entries();
        g.advance_replacements(false);
        settle(&mut g);
        assert_eq!(g.object_view(target).controller, owner);
        let roles = &g.state.roles.as_ref().unwrap().attached;
        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].recipient, g.object_ref(target));
        let aura = g.object_view(roles[0].aura.card);
        assert_eq!((aura.owner, aura.controller), (creator, creator));
    }
}

fn role295_rat(g: &mut Game, owner: Player) -> CardId {
    let card = Card::catalogue()
        .find(|c| {
            c.is_token() && c.has_subtype("Rat") && c.info().keywords.contains(Keywords::CANT_BLOCK)
        })
        .unwrap();
    spawn(g, owner, card, Zone::Battlefield)
}
fn role295_frogs(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.card.is_token()
                && o.card.has_subtype("Frog"))
            .then_some(id as CardId)
        })
        .collect()
}
#[test]
fn paid_asinine_flash_cost_timing_foreign_rights_and_nontarget_hexproof_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (flash, foreign, reduced) in [
            (false, false, false),
            (true, false, false),
            (true, true, false),
            (true, true, true),
        ] {
            let mut g = setup(p);
            let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[foe as usize].gained_keywords = Keywords::HEXPROOF;
            let ally = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[ally as usize].controller = p;
            if reduced {
                named(&mut g, p, "Archmage of Runes", Zone::Battlefield);
            }
            let spell = named(
                &mut g,
                if foreign { p.other() } else { p },
                "Asinine Antics",
                if foreign { Zone::Exile } else { Zone::Hand },
            );
            if foreign {
                g.state.exile_permissions.push(ExilePlayPermission {
                    object: g.object_ref(spell),
                    player: p,
                    expires_turn: g.state.turn,
                });
            }
            if flash {
                g.state.active = p.other();
            }
            g.state.priority = p;
            g.state.players[p.index()].mana = [0, 2, 0, 0, 0, if reduced { 2 } else { 3 }];
            assert!(
                !g.legal_actions()
                    .contains(&Action::AnnounceFlashSurcharge { card: spell })
            );
            if flash {
                let before = g.snapshot();
                let action = if foreign {
                    Action::AnnounceExile { card: spell }
                } else {
                    Action::AnnounceCast { card: spell }
                };
                assert_eq!(g.apply(p, action), Err(Error::IllegalAction));
                assert_eq!(g.snapshot(), before);
            }
            let expected = if flash {
                if reduced { 5 } else { 6 }
            } else {
                4
            };
            g.state.players[p.index()].mana = [0, 2, 0, 0, 0, expected - 2];
            act(
                &mut g,
                if flash {
                    Action::AnnounceFlashSurcharge { card: spell }
                } else {
                    Action::AnnounceCast { card: spell }
                },
            );
            assert!(!matches!(g.state.pending, Pending::Targets { .. }));
            let cast = g.state.announcing.as_ref().unwrap().item.cast.unwrap();
            assert_eq!(cast.flash_surcharge, flash);
            assert_eq!(cast.method, CastMethod::Normal);
            let before = g.snapshot();
            assert_eq!(
                g.apply(p, Action::PayCosts { payment: [0; 6] }),
                Err(Error::IllegalAction)
            );
            assert_eq!(g.snapshot(), before);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.stats(foe).power, 1);
            assert_eq!(g.stats(ally).power, 2);
            let aura = g
                .state
                .roles
                .as_ref()
                .unwrap()
                .attached
                .iter()
                .find(|r| r.recipient == g.object_ref(foe))
                .unwrap()
                .aura;
            assert_eq!(g.object_view(aura.card).controller, p);
            assert_eq!(g.object_view(aura.card).owner, p);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(
                g.object_view(spell).owner,
                if foreign { p.other() } else { p }
            );
        }
    }
}

#[test]
fn paid_mystical_tether_optional_flash_keeps_linked_return_and_foreign_owner_routing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.active = p.other();
        let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p.other(), "Mystical Tether", Zone::Exile);
        g.state.exile_permissions.push(ExilePlayPermission {
            object: g.object_ref(spell),
            player: p,
            expires_turn: g.state.turn,
        });
        let before = g.state.players[p.index()].mana;
        act(&mut g, Action::AnnounceFlashSurcharge { card: spell });
        for _ in 0..120 {
            if g.object_view(foe).zone == Zone::Exile {
                break;
            }
            let actions = g.legal_actions();
            let a = if g.state.pending == Pending::Priority {
                Action::Pass
            } else {
                actions
                    .iter()
                    .find(|a| {
                        **a == Action::ChooseTarget {
                            target: Target::Card(foe),
                        }
                    })
                    .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                    .unwrap()
                    .clone()
            };
            act(&mut g, a);
        }
        assert_eq!(g.object_view(foe).zone, Zone::Exile);
        assert_eq!(
            before.iter().sum::<u32>() - g.state.players[p.index()].mana.iter().sum::<u32>(),
            5
        );
        assert_eq!(g.object_view(spell).controller, p);
        assert_eq!(g.object_view(spell).owner, p.other());
        g.move_object(spell, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(foe).zone, Zone::Battlefield);
        assert_eq!(g.object_view(foe).controller, p.other());
        assert_eq!(g.object_view(spell).controller, p.other());
    }
}
#[test]
fn paid_sewer_witch_mass_role_event_binds_all_actual_rats_and_adds_only_one_extra_frog() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Quina, Qu Gourmet", Zone::Battlefield);
        let rat = role295_rat(&mut g, p.other());
        g.state.objects[rat as usize].controller = p;
        let mascot = named(&mut g, p, "Three Tree Mascot", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foe = role295_rat(&mut g, p.other());
        let source = named(&mut g, p, "Twisted Sewer-Witch", Zone::Hand);
        let before = g.state.players[p.index()].mana;
        act(&mut g, Action::AnnounceCast { card: source });
        settle(&mut g);
        assert_eq!(
            before.iter().sum::<u32>() - g.state.players[p.index()].mana.iter().sum::<u32>(),
            5
        );
        assert!(before[2] - g.state.players[p.index()].mana[2] >= 2);
        let roles = &g.state.roles.as_ref().unwrap().attached;
        assert_eq!(roles.len(), 3);
        for id in [rat, mascot] {
            assert!(roles.iter().any(|r| r.recipient == g.object_ref(id)));
        }
        assert!(
            !roles
                .iter()
                .any(|r| [bear, foe, source].contains(&r.recipient.card))
        );
        assert!(
            roles.iter().all(
                |r| g.state.objects[r.aura.card as usize].card.role() == Some(RoleKind::Wicked)
            )
        );
        assert!(roles.iter().all(|r| r.timestamp == roles[0].timestamp));
        let frogs = role295_frogs(&g, p);
        assert_eq!(frogs.len(), 2); // Rat event + ONE mass Role event.
        assert!(
            frogs
                .iter()
                .all(|id| g.object_view(*id).attached_to.is_none())
        );
    }
}
#[test]
fn paid_mass_roles_preserve_group_indices_through_doubling_and_partial_or_all_stale_recipients() {
    for p in [Player::P0, Player::P1] {
        for all_stale in [false, true] {
            let mut g = setup(p);
            let mascot = named(&mut g, p, "Three Tree Mascot", Zone::Battlefield);
            role295_rat(&mut g, p);
            let doubling = named(&mut g, p, "Doubling Season", Zone::Battlefield);
            named(&mut g, p, "Quina, Qu Gourmet", Zone::Battlefield);
            let source = named(&mut g, p, "Twisted Sewer-Witch", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: source });
            let mut frozen = Vec::new();
            for _ in 0..180 {
                if let Some(choice) = g.observe(p).replacement_choice
                    && choice.tokens.iter().any(|t| t.token.role().is_some())
                {
                    frozen = g
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter_map(|(id, o)| {
                            (o.zone == Zone::Battlefield
                                && o.controller == p
                                && g.has_subtype(id as CardId, "Rat"))
                            .then_some(g.object_ref(id as CardId))
                        })
                        .collect();
                    assert_eq!(choice.role_attachments.len(), choice.tokens.len());
                    assert_eq!(
                        choice
                            .role_attachments
                            .into_iter()
                            .flatten()
                            .collect::<Vec<_>>(),
                        frozen
                    );
                    let other = g.observe(p.other()).replacement_choice.unwrap();
                    assert_eq!(
                        other
                            .role_attachments
                            .into_iter()
                            .flatten()
                            .collect::<Vec<_>>(),
                        frozen
                    );
                    break;
                }
                let actions = g.legal_actions();
                let a = if g.state.pending == Pending::Priority {
                    Action::Pass
                } else {
                    actions.iter().find(|a|matches!(a,Action::ChooseReplacement{source,..}if source.card==doubling))
                        .or_else(||actions.iter().find(|a|!matches!(a,Action::Concede))).unwrap().clone()
                };
                act(&mut g, a);
            }
            assert_eq!(frozen.len(), 4);
            let old_mascot = g.object_ref(mascot);
            if all_stale {
                for r in &frozen {
                    g.move_object(r.card, Zone::Exile);
                }
            } else {
                g.move_object(mascot, Zone::Graveyard);
                g.move_object(mascot, Zone::Battlefield);
                assert_ne!(g.object_ref(mascot), old_mascot);
            }
            assert!(
                g.observe(p)
                    .replacement_choice
                    .unwrap()
                    .role_attachments
                    .contains(&Some(old_mascot))
            );
            for _ in 0..180 {
                if g.state.pending == Pending::Priority
                    && g.state.stack.is_empty()
                    && g.state.resolving.is_none()
                    && g.state.pending_triggers.is_empty()
                {
                    break;
                }
                let actions = g.legal_actions();
                let a = if g.state.pending == Pending::Priority {
                    Action::Pass
                } else {
                    actions.iter().find(|a|matches!(a,Action::ChooseReplacement{source,..}if source.card==doubling))
                        .or_else(||actions.iter().find(|a|!matches!(a,Action::Concede))).unwrap().clone()
                };
                act(&mut g, a);
            }
            let roles = g
                .state
                .roles
                .as_ref()
                .map_or_else(Vec::new, |r| r.attached.clone());
            assert_eq!(roles.len(), if all_stale { 0 } else { 3 });
            assert!(
                roles
                    .iter()
                    .all(|r| frozen.contains(&r.recipient) && r.recipient != old_mascot)
            );
            if !all_stale {
                for recipient in frozen.iter().filter(|r| **r != old_mascot) {
                    assert_eq!(
                        roles.iter().filter(|r| r.recipient == *recipient).count(),
                        1
                    );
                }
            }
            let frogs = role295_frogs(&g, p);
            assert_eq!(frogs.len(), 2);
            assert!(
                frogs
                    .iter()
                    .all(|id| g.object_view(*id).attached_to.is_none())
            );
        }
    }
}
