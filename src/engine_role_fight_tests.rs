//! Successful Role creation and a separately targeted exact-subject reflexive fight.
use super::tests::{scenario, spawn};
use super::*;
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
    for player in [p, p.other()] {
        for _ in 0..8 {
            named(&mut g, player, "Forest", Zone::Library);
        }
        for _ in 0..2 {
            named(&mut g, player, "Forest", Zone::Hand);
        }
    }
    g
}
fn act(g: &mut Game, a: Action) {
    let p = g.actor().unwrap();
    let before = g.snapshot();
    let revision = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|x| x == &a)
        .unwrap_or_else(|| panic!("illegal{a:?} {:?}", g.observe(p)));
    assert_eq!(g.apply(p.other(), a.clone()), Err(Error::WrongActor));
    assert_eq!(g.snapshot(), before);
    let mut child = {
        let parent = g.clone();
        parent.clone()
    };
    child.apply_index(p, revision, index).unwrap();
    g.apply(p, a.clone()).unwrap();
    assert_eq!(g.snapshot(), child.snapshot());
    for viewer in [p, p.other()] {
        assert_eq!(
            serde_json::to_value(g.observe(viewer)).unwrap(),
            serde_json::to_value(child.observe(viewer)).unwrap()
        );
    }
    let after = g.snapshot();
    g.restore(&before).unwrap();
    assert_eq!(g.apply_index(p, revision, index), Err(Error::StaleRevision));
    g.apply(p, a).unwrap();
    assert_eq!(g.snapshot(), after);
}
fn deny(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn idle(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
}

fn next(g: &Game) -> Action {
    if g.state.pending == Pending::Priority {
        return Action::Pass;
    }
    let actions = g.legal_actions();
    actions
        .iter()
        .find(|a| matches!(a, Action::FinishTargets))
        .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
        .unwrap()
        .clone()
}
fn settle(g: &mut Game) {
    for _ in 0..240 {
        if idle(g) {
            return;
        }
        let a = next(g);
        act(g, a);
    }
    panic!("unsettled {:?}", g.observe(g.actor().unwrap()));
}
fn paid(g: &mut Game, p: Player, recipient: CardId, foreign: bool) -> CardId {
    let spell = named(
        g,
        if foreign { p.other() } else { p },
        "Curse of the Werefox",
        if foreign { Zone::Exile } else { Zone::Hand },
    );
    if foreign {
        g.state.exile_permissions.push(ExilePlayPermission {
            object: g.object_ref(spell),
            player: p,
            expires_turn: g.state.turn,
        });
    }
    let before = g.state.players[p.index()].mana;
    act(
        g,
        if foreign {
            Action::AnnounceExile { card: spell }
        } else {
            Action::AnnounceCast { card: spell }
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(recipient),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    deny(g, Action::PayCosts { payment: [0; 6] });
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    act(g, pay);
    assert_eq!(
        before.iter().sum::<u32>() - g.state.players[p.index()].mana.iter().sum::<u32>(),
        3
    );
    assert!(before[4] > g.state.players[p.index()].mana[4]);
    spell
}
fn to_reflexive(g: &mut Game) -> SpellView {
    for _ in 0..240 {
        if let Some(a) = &g.state.announcing
            && matches!(a.item.kind, StackKind::TriggeredAbility { .. })
            && a.item.reflexive_subject.is_some()
        {
            return a.item.clone();
        }
        let a = next(g);
        act(g, a);
    }
    panic!("no reflexive {:?}", g.observe(g.actor().unwrap()));
}
fn announce_fight(g: &mut Game, target: Option<CardId>) {
    if let Some(id) = target {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    for _ in 0..50 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = next(g);
        act(g, a);
    }
    panic!("unfinished reflexive");
}
#[test]
fn paid_complete_body_late_optional_fight_foreign_ownership_lifelink_and_hexproof_both_seats() {
    for p in [Player::P0, Player::P1] {
        for foreign in [false, true] {
            for decline in [false, true] {
                let mut g = setup(p);
                let own = named(
                    &mut g,
                    if foreign { p.other() } else { p },
                    "Shattered Acolyte",
                    Zone::Battlefield,
                );
                g.state.objects[own as usize].controller = p;
                g.state.objects[own as usize].gained_keywords = Keywords::HEXPROOF;
                let foe = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
                g.state.objects[foe as usize].controller = p.other();
                let protected = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
                g.state.objects[protected as usize].gained_keywords = Keywords::HEXPROOF;
                let original = g.object_ref(own);
                let life = g.state.players[p.index()].life;
                let spell = paid(&mut g, p, own, foreign);
                let spell_object = g.object_ref(spell);
                assert!(
                    g.state
                        .stack
                        .last()
                        .unwrap()
                        .targets
                        .iter()
                        .all(|b| b.target == Target::Card(own))
                );
                let packet = to_reflexive(&mut g);
                assert_eq!(packet.reflexive_subject.as_ref().unwrap().object, original);
                assert_eq!(
                    (packet.controller, packet.card, packet.object),
                    (p, spell, spell_object)
                );
                assert_eq!(
                    packet.source_card,
                    Card::from_name("Curse of the Werefox").unwrap()
                );
                assert!(packet.targets.is_empty());
                assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
                assert_eq!(
                    g.object_view(spell).owner,
                    if foreign { p.other() } else { p }
                );
                assert_eq!(g.stats(own).power, 3);
                assert_eq!(g.object_view(foe).damage, 0);
                deny(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(protected),
                    },
                );
                deny(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(own),
                    },
                );
                announce_fight(&mut g, if decline { None } else { Some(foe) });
                settle(&mut g);
                assert_eq!(
                    g.state.players[p.index()].life,
                    life + if decline { 0 } else { 3 }
                );
                assert_eq!(g.object_view(own).zone, Zone::Battlefield);
                assert_eq!(g.object_view(own).damage, if decline { 0 } else { 2 });
                assert_eq!(
                    g.object_view(foe).zone,
                    if decline {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    }
                );
                assert_eq!(g.object_view(foe).owner, p);
            }
        }
    }
}
#[test]
fn successful_creation_is_one_reflexive_event_even_for_doubled_roles_and_extra_frog() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        named(&mut g, p, "Quina, Qu Gourmet", Zone::Battlefield);
        paid(&mut g, p, own, false);
        let packet = to_reflexive(&mut g);
        assert_eq!(
            packet.reflexive_subject.as_ref().unwrap().object,
            g.object_ref(own)
        );
        assert!(
            g.state
                .pending_triggers
                .iter()
                .all(|x| x.reflexive_subject.is_none())
        );
        assert!(g.state.stack.iter().all(|x| x.reflexive_subject.is_none()));
        let roles = g
            .state
            .roles
            .as_ref()
            .unwrap()
            .attached
            .iter()
            .filter(|r| g.on_battlefield(r.aura.card))
            .count();
        assert_eq!(roles, 1);
        let frogs = g
            .state
            .objects
            .iter()
            .filter(|o| {
                o.zone == Zone::Battlefield && o.card.is_token() && o.card.has_subtype("Frog")
            })
            .count();
        assert!(frogs >= 1);
        announce_fight(&mut g, None);
        settle(&mut g);
        assert!(g.state.pending_triggers.is_empty());
    }
}
#[test]
fn no_actual_attached_role_never_triggers_even_if_replacement_creates_a_frog() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        named(&mut g, p, "Quina, Qu Gourmet", Zone::Battlefield);
        paid(&mut g, p, own, false);
        for _ in 0..80 {
            if matches!(g.state.pending, Pending::Replacement { .. }) {
                break;
            }
            let a = next(&g);
            act(&mut g, a);
        }
        assert!(matches!(g.state.pending, Pending::Replacement { .. }));
        // Synthetic replacement-boundary mutation isolates stale exact attachment;
        // no ordinary player action is offered during this replacement choice.
        g.move_object(own, Zone::Graveyard);
        g.move_object(own, Zone::Battlefield);
        settle(&mut g);
        assert!(g.state.roles.as_ref().is_none_or(|r| r.attached.is_empty()));
        assert!(
            g.state
                .pending_triggers
                .iter()
                .all(|x| x.reflexive_subject.is_none())
        );
        assert!(g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
            && o.card.is_token()
            && o.card.has_subtype("Frog")));
    }
}
#[test]
fn exact_source_and_target_reincarnations_counter_and_current_fighter_control_are_independent() {
    for p in [Player::P0, Player::P1] {
        for mutation in 0..5 {
            let mut g = setup(p);
            let own = named(&mut g, p, "Shattered Acolyte", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let life = [g.state.players[0].life, g.state.players[1].life];
            paid(&mut g, p, own, true);
            to_reflexive(&mut g);
            if mutation == 3 {
                g.state.objects[own as usize].controller = p.other();
                g.state.objects[own as usize].gained_keywords = Keywords::HEXPROOF;
            }
            announce_fight(&mut g, Some(foe));
            match mutation {
                0 => {
                    g.move_object(own, Zone::Graveyard);
                    g.move_object(own, Zone::Battlefield);
                }
                1 => {
                    g.move_object(foe, Zone::Graveyard);
                    g.move_object(foe, Zone::Battlefield);
                }
                2 => {
                    let id = g.state.stack.last().unwrap().id;
                    assert!(g.counter_stack_item(id, false));
                }
                4 => {
                    let role = g.state.roles.as_ref().unwrap().attached[0].aura;
                    g.move_object(role.card, Zone::Graveyard);
                }
                _ => {}
            }
            settle(&mut g);
            if mutation <= 2 {
                assert_eq!(g.object_view(own).damage, 0);
                assert_eq!(g.object_view(foe).damage, 0);
                assert_eq!([g.state.players[0].life, g.state.players[1].life], life);
            }
            if mutation == 3 {
                assert_eq!(g.state.players[p.index()].life, life[p.index()]);
                assert_eq!(
                    g.state.players[p.other().index()].life,
                    life[p.other().index()] + 3
                );
                assert_eq!(g.object_view(own).damage, 2);
                assert_eq!(g.object_view(foe).zone, Zone::Graveyard);
            }
            if mutation == 4 {
                assert_eq!(g.state.players[p.index()].life, life[p.index()] + 2);
                assert_eq!(g.object_view(own).zone, Zone::Graveyard);
                assert_eq!(g.object_view(foe).zone, Zone::Graveyard);
            }
        }
    }
}

fn response(g: &mut Game, p: Player, name: &str, target: Target) {
    if g.actor() != Some(p) {
        act(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
    let spell = named(g, p, name, Zone::Hand);
    let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
    act(g, Action::AnnounceCast { card: spell });
    act(g, Action::ChooseTarget { target });
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    act(g, pay);
    assert_eq!(
        mana - g.state.players[p.index()].mana.iter().sum::<u32>(),
        if name == "Murder" { 3 } else { 2 }
    );
}
#[test]
fn actual_paid_murder_negate_and_disenchant_responses_preserve_success_and_live_fight_stats() {
    for p in [Player::P0, Player::P1] {
        for route in 0..3 {
            let mut g = setup(p);
            let own = named(&mut g, p, "Shattered Acolyte", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let life = g.state.players[p.index()].life;
            let spell = paid(&mut g, p, own, true);
            if route < 2 {
                response(
                    &mut g,
                    p.other(),
                    if route == 0 { "Murder" } else { "Negate" },
                    if route == 0 {
                        Target::Card(own)
                    } else {
                        Target::Spell(spell)
                    },
                );
                settle(&mut g);
                assert!(g.state.roles.as_ref().is_none_or(|r| r.attached.is_empty()));
                assert_eq!(
                    g.object_view(own).zone,
                    if route == 0 {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    }
                );
                assert_eq!(g.state.players[p.index()].life, life);
                assert_eq!(g.object_view(foe).damage, 0);
            } else {
                to_reflexive(&mut g);
                announce_fight(&mut g, Some(foe));
                let role = g.state.roles.as_ref().unwrap().attached[0].aura;
                response(&mut g, p.other(), "Disenchant", Target::Card(role.card));
                settle(&mut g);
                assert_eq!(g.state.players[p.index()].life, life + 2);
                assert_eq!(g.object_view(own).zone, Zone::Graveyard);
                assert_eq!(g.object_view(foe).zone, Zone::Graveyard);
            }
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(g.object_view(spell).owner, p.other());
        }
    }
}
#[test]
fn successful_role_with_no_opposing_creature_still_queues_one_zero_target_reflexive() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        paid(&mut g, p, own, false);
        let mut found = false;
        for _ in 0..80 {
            let packets = g
                .state
                .stack
                .iter()
                .filter(|x| x.reflexive_subject.is_some())
                .collect::<Vec<_>>();
            if !packets.is_empty() {
                assert_eq!(packets.len(), 1);
                assert!(packets[0].targets.is_empty());
                assert_eq!(
                    packets[0].reflexive_subject.as_ref().unwrap().object,
                    g.object_ref(own)
                );
                found = true;
                break;
            }
            let a = next(&g);
            act(&mut g, a);
        }
        assert!(found);
        settle(&mut g);
        assert_eq!(g.object_view(own).damage, 0);
        assert_eq!(g.stats(own).power, 3);
    }
}
