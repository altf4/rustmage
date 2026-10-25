//! Complete hybrid creature and every-created-Equipment exact-source continuation.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::msh_galactus_tests::{finish, paid, reject, tokens, until};
use super::*;
const NAME: &str = "U.S.Agent, John Walker";
fn start(g: &mut Game, p: Player, color: usize) -> CardId {
    let s = named(g, p, NAME, Zone::Hand);
    g.state.players[p.index()].mana = [0; 6];
    g.state.players[p.index()].mana[color] = 1;
    g.state.players[p.index()].mana[5] = 3;
    checked(g, Action::AnnounceCast { card: s });
    checked(
        g,
        Action::ChooseSpellCost {
            cost: u16::from(color == 0),
        },
    );
    finish(g);
    s
}
fn agent(g: &mut Game, p: Player) -> CardId {
    let s = start(g, p, 2);
    settle(g);
    s
}
fn stack_etb(g: &mut Game, s: CardId) {
    until(g, |g| {
        g.state
            .stack
            .iter()
            .any(|i| i.card == s && i.kind == StackKind::TriggeredAbility { ability: 0 })
    });
}
fn shields(g: &Game, p: Player) -> Vec<CardId> {
    tokens(g, p, Card::MshSturdyShieldToken)
}
fn announce(shield: CardId) -> Action {
    Action::AnnounceActivate {
        card: shield,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn equip(g: &mut Game, shield: CardId, target: CardId) {
    checked(g, announce(shield));
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    finish(g);
    settle(g);
}
#[test]
fn msh_us_agent_both_paid_hybrid_choices_exact_shield_and_creature() {
    for p in [Player::P0, Player::P1] {
        for color in [0, 2] {
            let mut g = setup(p);
            let s = start(&mut g, p, color);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let v = g.object_view(s);
            assert_eq!((v.power, v.toughness, v.colors), (4, 4, 5));
            assert!(v.card.is_legendary());
            for ty in ["Human", "Soldier", "Hero"] {
                assert!(v.has_subtype(ty));
            }
            let sh = shields(&g, p);
            assert_eq!(sh.len(), 1);
            let t = g.object_view(sh[0]);
            assert_eq!(t.name(), "Sturdy Shield");
            assert_eq!((t.kinds, t.colors), (16, 0));
            assert!(t.has_subtype("Equipment"));
            assert!(!t.card.is_legendary());
            assert_eq!(t.card.fixed_mana_value(), 0);
            assert!(!t.tapped);
            assert_eq!(t.keywords, Keywords::NONE);
            assert_eq!(t.attached_to, Some(g.object_ref(s)));
            assert_eq!(t.card.activations().len(), 1);
        }
    }
}
#[test]
fn msh_us_agent_insufficient_mana_and_foreign_cast_are_atomic() {
    for p in [Player::P0, Player::P1] {
        for mana in [[0, 0, 0, 0, 0, 4], [0, 0, 1, 0, 0, 2]] {
            let mut g = setup(p);
            let s = named(&mut g, p, NAME, Zone::Hand);
            let foreign = named(&mut g, p.other(), NAME, Zone::Hand);
            g.state.players[p.index()].mana = mana;
            reject(&mut g, Action::AnnounceCast { card: s });
            reject(&mut g, Action::AnnounceCast { card: foreign });
            assert!(shields(&g, p).is_empty());
        }
    }
}
#[test]
fn msh_us_agent_doubling_attaches_every_new_shield_leaves_old_shield_unchanged() {
    for p in [Player::P0, Player::P1] {
        for seasons in 1..=2 {
            let mut g = setup(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.create_tokens(p, Card::MshSturdyShieldToken, 1, false);
            let old = shields(&g, p)[0];
            assert!(g.attach_equipment(g.object_ref(old), g.object_ref(bear)));
            for _ in 0..seasons {
                named(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            let s = agent(&mut g, p);
            let n = 1i128 << seasons;
            assert_eq!(
                g.stats(s),
                Stats {
                    power: 3 + n,
                    toughness: 2 + 2 * n
                }
            );
            assert_eq!(shields(&g, p).len(), 1 + (1 << seasons));
            assert_eq!(g.object_view(old).attached_to, Some(g.object_ref(bear)));
            for sh in shields(&g, p).into_iter().filter(|&sh| sh != old) {
                assert_eq!(g.object_view(sh).attached_to, Some(g.object_ref(s)));
            }
        }
    }
}
#[test]
fn msh_us_agent_source_paid_bounce_creates_unattached_and_old_packet_never_rebinds() {
    for p in [Player::P0, Player::P1] {
        for returned in [false, true] {
            let mut g = setup(p);
            let s = start(&mut g, p, 2);
            stack_etb(&mut g, s);
            let old = g.object_ref(s);
            if g.actor() != Some(p.other()) {
                checked(&mut g, Action::Pass);
            }
            g.state.players[p.other().index()].mana = [30; 6];
            let u = named(&mut g, p.other(), "Unsummon", Zone::Hand);
            cast(&mut g, u, None, &[Target::Card(s)]);
            until(&mut g, |g| g.object_view(s).zone == Zone::Hand);
            if returned {
                g.move_object(s, Zone::Battlefield);
                assert_ne!(g.object_ref(s), old);
            }
            settle(&mut g);
            let ss = shields(&g, p);
            assert_eq!(ss.len(), if returned { 2 } else { 1 });
            assert_eq!(
                ss.iter()
                    .filter(|&&sh| g.object_view(sh).attached_to.is_none())
                    .count(),
                1
            );
        }
    }
}
#[test]
fn msh_us_agent_control_change_keeps_trigger_controller_and_cross_controller_attachment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = start(&mut g, p, 2);
        stack_etb(&mut g, s);
        g.change_controller(g.object_ref(s), p.other());
        settle(&mut g);
        assert_eq!(g.object_view(s).controller, p.other());
        let sh = shields(&g, p);
        assert_eq!(sh.len(), 1);
        assert!(shields(&g, p.other()).is_empty());
        assert_eq!(g.object_view(sh[0]).attached_to, Some(g.object_ref(s)));
        assert_eq!(
            g.stats(s),
            Stats {
                power: 4,
                toughness: 4
            }
        );
    }
}
#[test]
fn msh_us_agent_nontarget_attachment_ignores_hexproof_and_ward_but_obeys_protection() {
    for p in [Player::P0, Player::P1] {
        for keywords in [Keywords::HEXPROOF, Keywords::PROTECTION_EVERYTHING] {
            let mut g = setup(p);
            let s = start(&mut g, p, 2);
            stack_etb(&mut g, s);
            g.state.objects[s as usize].gained_keywords = keywords;
            settle(&mut g);
            let sh = shields(&g, p)[0];
            assert_eq!(
                g.object_view(sh).attached_to.is_some(),
                keywords == Keywords::HEXPROOF
            );
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn msh_us_agent_attachment_recipient_no_longer_creature_creates_unattached() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = start(&mut g, p, 2);
        stack_etb(&mut g, s);
        // Bounded ordinary Aura setup; queued ETB survives later ability removal.
        let aura = named(&mut g, p, "Imprisoned in the Moon", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(s));
        settle(&mut g);
        assert!(!g.is_creature(s));
        assert_eq!(shields(&g, p).len(), 1);
        assert!(g.object_view(shields(&g, p)[0]).attached_to.is_none());
    }
}
#[test]
fn msh_us_agent_paid_equip_two_retargets_only_own_creature_at_sorcery_speed() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = agent(&mut g, p);
        let sh = shields(&g, p)[0];
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.priority = p;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(&mut g, announce(sh));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        checked(&mut g, announce(sh));
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(foreign),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(sh),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(
            g.stats(s),
            Stats {
                power: 3,
                toughness: 2
            }
        );
        assert_eq!(
            g.stats(own),
            Stats {
                power: 3,
                toughness: 4
            }
        );
        g.state.active = p.other();
        g.state.priority = p;
        reject(&mut g, announce(sh));
    }
}
#[test]
fn msh_us_agent_equip_target_control_change_and_source_leave_revalidate() {
    for p in [Player::P0, Player::P1] {
        for source_leaves in [false, true] {
            let mut g = setup(p);
            let s = agent(&mut g, p);
            let sh = shields(&g, p)[0];
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.players[p.index()].mana = [30; 6];
            g.state.priority = p;
            checked(&mut g, announce(sh));
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            finish(&mut g);
            if source_leaves {
                g.move_object(sh, Zone::Exile);
            } else {
                g.change_controller(g.object_ref(own), p.other());
            }
            settle(&mut g);
            assert_eq!(
                g.stats(own),
                Stats {
                    power: 2,
                    toughness: 2
                }
            );
            assert_eq!(g.stats(s).power, if source_leaves { 3 } else { 4 });
        }
    }
}
#[test]
fn msh_us_agent_equipment_animation_detaches_and_source_removal_loses_bonus() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = agent(&mut g, p);
        let sh = shields(&g, p)[0];
        g.state.players[p.index()].mana = [30; 6];
        g.state.priority = p;
        paid(&mut g, p, "I Am Iron Man", &[Target::Card(sh)]);
        assert!(g.is_creature(sh));
        assert!(g.object_view(sh).attached_to.is_none());
        assert_eq!(
            g.stats(s),
            Stats {
                power: 3,
                toughness: 2
            }
        );
        g.state.roles.as_mut().unwrap().animations.clear();
        g.state.players[p.index()].mana = [30; 6];
        g.state.priority = p;
        equip(&mut g, sh, s);
        assert_eq!(g.stats(s).power, 4);
        g.move_object(sh, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.stats(s).power, 3);
    }
}
#[test]
fn msh_us_agent_replacement_addition_non_equipment_stays_unattached() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Worldwalker Helm", Zone::Battlefield);
        let s = agent(&mut g, p);
        let sh = shields(&g, p);
        assert_eq!(sh.len(), 1);
        assert_eq!(g.object_view(sh[0]).attached_to, Some(g.object_ref(s)));
        let map = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| {
                o.zone == Zone::Battlefield && o.is_token() && o.card != Card::MshSturdyShieldToken
            })
            .unwrap()
            .0 as CardId;
        assert!(g.object_view(map).attached_to.is_none());
        assert_eq!(g.stats(s).power, 4);
    }
}
#[test]
fn msh_us_agent_full_hidden_observations_and_orphan_replacement_branches_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let secret = named(&mut g, p, NAME, Zone::Hand);
        let mut alternate = g.clone();
        alternate.state.objects[secret as usize].card = Card::from_name("Bear Cub").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            serde_json::to_value(alternate.observe(p.other())).unwrap()
        );
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        named(&mut g, p, "Worldwalker Helm", Zone::Battlefield);
        let s = start(&mut g, p, 2);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Replacement { .. })
        });
        let before = g.snapshot();
        let actor = g.actor().unwrap();
        let revision = g.revision();
        let actions = g.legal_actions();
        let action = actions
            .iter()
            .find(|a| !matches!(a, Action::Concede))
            .unwrap()
            .clone();
        let index = actions.iter().position(|a| *a == action).unwrap();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        raw.apply(actor, action.clone()).unwrap();
        indexed.apply_index(actor, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        settle(&mut raw);
        settle(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.stats(s).power, 5);
        raw.restore(&before).unwrap();
        raw.apply(actor, action).unwrap();
        settle(&mut raw);
        assert_eq!(raw.snapshot(), indexed.snapshot());
    }
}
#[test]
fn msh_us_agent_real_hero_restricted_funding_cast_and_artifact_only_cannot_pay() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let tower = named(&mut g, p, "Avengers Tower", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        let s = named(&mut g, p, NAME, Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: s });
        checked(&mut g, Action::ChooseSpellCost { cost: 0 });
        checked(
            &mut g,
            Action::ActivateMana {
                card: tower,
                ability: 1,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 1, 0, 0, 0],
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(shields(&g, p).len(), 1);
        g.state.priority = p;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        let castle = named(&mut g, p, "Castle Doom", Zone::Battlefield);
        g.state.objects[tower as usize].tapped = true;
        let other = named(&mut g, p, NAME, Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: other });
        assert!(!g.object_view(castle).tapped);
    }
}
#[test]
fn msh_us_agent_live_shield_power_funds_actual_crew_four() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let vehicle = named(&mut g, p, "Dependable Quinjet", Zone::Battlefield);
        let s = start(&mut g, p, 2);
        stack_etb(&mut g, s);
        assert_eq!(g.stats(s).power, 3);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: vehicle,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        settle(&mut g);
        g.state.priority = p;
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: vehicle,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        finish(&mut g);
        let object = g.object_ref(s);
        checked(&mut g, Action::ChooseCrewer { object });
        checked(&mut g, Action::FinishCrew);
        settle(&mut g);
        assert!(g.is_creature(vehicle));
        assert!(g.object_view(s).tapped);
    }
}
