//! Whole Role consumers, paid origins and exact owned continuation boundaries.
use super::tests::{scenario, spawn};
use super::*;
use crate::cards::RoleKind;
const NAMES: [&str; 12] = [
    "Become Brutes",
    "Cursed Courtier",
    "Cut In",
    "Diminisher Witch",
    "Eriette's Whisper",
    "Merry Bards",
    "Protective Parents",
    "Shatter the Oath",
    "Spellbook Vendor",
    "The Witch's Vanity",
    "Unassuming Sage",
    "Witch's Mark",
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
fn pick(g: &Game, targets: &[Target], pay: bool, bargain: Option<CardId>, discard: bool) -> Action {
    let actions = g.legal_actions();
    if g.state.pending == Pending::Priority {
        return Action::Pass;
    }
    if let Some(a) = actions
        .iter()
        .find(|a| matches!(a,Action::ChooseBargain{sacrifice}if sacrifice.map(|r|r.card)==bargain))
    {
        return a.clone();
    }
    if let Some(a) = actions.iter().find(|a| {
        if pay {
            matches!(a, Action::PayResolutionMana { .. })
        } else {
            matches!(a, Action::DeclineResolutionMana)
        }
    }) {
        return a.clone();
    }
    for target in targets {
        if let Some(a) = actions
            .iter()
            .find(|a| matches!(a,Action::ChooseTarget{target:t}if t==target))
        {
            return a.clone();
        }
    }
    if !discard
        && let Some(a) = actions
            .iter()
            .find(|a| matches!(a, Action::DeclineEffectPayment))
    {
        return a.clone();
    }
    if let Some(a) = actions.iter().find(|a| matches!(a, Action::FinishTargets)) {
        return a.clone();
    }
    actions
        .into_iter()
        .find(|a| {
            !matches!(
                a,
                Action::Concede
                    | Action::ChooseBargain { .. }
                    | Action::DeclineResolutionMana
                    | Action::DeclineEffectPayment
            )
        })
        .unwrap_or_else(|| panic!("no policy {:?}", g.observe(g.actor().unwrap())))
}
fn drive(g: &mut Game, targets: &[Target], pay: bool, bargain: Option<CardId>, discard: bool) {
    g.prepare_triggers();
    for _ in 0..250 {
        if idle(g) {
            return;
        }
        let a = pick(g, targets, pay, bargain, discard);
        act(g, a);
    }
    panic!("unsettled {:?}", g.observe(g.actor().unwrap()));
}
fn announce(g: &mut Game, p: Player, n: &str, foreign: bool) -> CardId {
    g.state.priority = p;
    let id = named(
        g,
        if foreign { p.other() } else { p },
        n,
        if foreign { Zone::Exile } else { Zone::Hand },
    );
    if foreign {
        g.state.exile_permissions.push(ExilePlayPermission {
            cast_only: false,
            without_mana: false,
            object: g.object_ref(id),
            player: p,
            expires_turn: g.state.turn,
            next_end_step: None,
        });
        g.state.exile_permissions.sort_by_key(|v| v.object.card);
        act(g, Action::AnnounceExile { card: id });
    } else {
        act(g, Action::AnnounceCast { card: id });
    }
    id
}
fn roles(g: &Game, recipient: CardId, kind: RoleKind, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.zone == Zone::Battlefield
                && o.card.role() == Some(kind)
                && o.controller == p
                && o.attached_to == Some(g.object_ref(recipient))
        })
        .map(|(i, o)| {
            assert_eq!(o.owner, p);
            i as CardId
        })
        .collect()
}
fn total_roles(g: &Game, kind: RoleKind) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| o.card.role() == Some(kind))
        .count()
}
fn complete_case(p: Player, index: usize, foreign: bool) {
    let name = NAMES[index];
    let mut g = setup(p);
    let ally = named(
        &mut g,
        if foreign { p.other() } else { p },
        "Bear Cub",
        Zone::Battlefield,
    );
    g.state.objects[ally as usize].controller = p;
    let foe = named(
        &mut g,
        if foreign { p } else { p.other() },
        if index == 9 { "Bear Cub" } else { "Hill Giant" },
        Zone::Battlefield,
    );
    g.state.objects[foe as usize].controller = p.other();
    let fodder = named(
        &mut g,
        if foreign { p.other() } else { p },
        "Candy Trail",
        Zone::Battlefield,
    );
    g.state.objects[fodder as usize].controller = p;
    let before = g.state.players[p.index()].mana.iter().sum::<u32>();
    let opponent_hand = g.hand_count(p.other());
    let before_draw = g.state.players[p.index()].cards_drawn_this_turn;
    let before_discard = g.state.players[p.index()].cards_discarded_this_turn;
    let source = announce(&mut g, p, name, foreign);
    let mut targets = vec![
        Target::Card(foe),
        Target::Player(p.other()),
        Target::Card(ally),
    ];
    if index == 0 {
        targets = vec![Target::Card(ally), Target::Card(foe)];
    }
    drive(
        &mut g,
        &targets,
        true,
        if index == 3 { Some(fodder) } else { None },
        true,
    );
    let (colored, generic) = Card::from_name(name).unwrap().cost();
    assert_eq!(
        before - g.state.players[p.index()].mana.iter().sum::<u32>(),
        colored.iter().sum::<u32>()
            + generic
            + if index == 5 {
                1
            } else if index == 10 {
                2
            } else {
                0
            },
        "{name} {p:?} {foreign}"
    );
    if index == 6 {
        g.state.priority = p.other();
        let murder = named(&mut g, p.other(), "Murder", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: murder });
        drive(
            &mut g,
            &[Target::Card(source), Target::Card(ally)],
            true,
            None,
            true,
        );
    }
    if index == 8 {
        g.begin_step(Step::BeginCombat);
        g.give_priority();
        drive(&mut g, &[Target::Card(ally)], true, None, true);
    }
    if index == 9 {
        assert_eq!(g.object_view(foe).zone, Zone::Graveyard);
        g.begin_step(Step::Main1);
        g.give_priority();
        drive(&mut g, &[Target::Card(ally)], true, None, true);
        assert!(g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
            && o.owner == p
            && o.card.token_info().is_some_and(|t| t.subtypes == "Food")));
        let food = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| {
                o.zone == Zone::Battlefield
                    && o.owner == p
                    && o.card.token_info().is_some_and(|t| t.subtypes == "Food")
            })
            .unwrap()
            .0 as CardId;
        g.state.priority = p;
        let life = g.state.players[p.index()].life;
        let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
        act(
            &mut g,
            Action::AnnounceActivate {
                card: food,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        drive(&mut g, &[], true, None, true);
        assert_eq!(
            mana - g.state.players[p.index()].mana.iter().sum::<u32>(),
            2
        );
        assert_eq!(g.state.players[p.index()].life, life + 3);
        assert!(!g.on_battlefield(food));
        g.begin_step(Step::Main1);
        g.give_priority();
        drive(&mut g, &[Target::Card(ally)], true, None, true);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    }
    let kind = match index {
        0 => RoleKind::Monster,
        1 | 3 => RoleKind::Cursed,
        2 | 5 | 6 => RoleKind::YoungHero,
        8 | 10 => RoleKind::Sorcerer,
        _ => RoleKind::Wicked,
    };
    let recipient = match index {
        1 | 10 => source,
        3 => foe,
        _ => ally,
    };
    assert_eq!(
        roles(&g, recipient, kind, p).len(),
        1,
        "{name} {p:?} foreign={foreign}"
    );
    if index == 0 {
        assert_eq!(roles(&g, foe, kind, p).len(), 1);
        assert!(g.keywords(ally).contains(Keywords::HASTE));
        assert!(g.keywords(foe).contains(Keywords::HASTE));
        assert!(g.keywords(foe).contains(Keywords::TRAMPLE));
    }
    if index == 1 {
        assert_eq!(g.stats(source).power, 1);
        assert!(g.keywords(source).contains(Keywords::LIFELINK));
    }
    if index == 2 || index == 7 {
        assert_eq!(g.object_view(foe).zone, Zone::Graveyard);
    }
    if index == 3 {
        assert_eq!(g.object_view(fodder).zone, Zone::Graveyard);
        assert_eq!(g.stats(foe).power, 1);
    }
    if index == 4 {
        assert_eq!(opponent_hand - g.hand_count(p.other()), 2);
    }
    if index == 8 {
        assert!(g.keywords(source).contains(Keywords::VIGILANCE));
    }
    if index == 11 {
        assert_eq!(
            g.state.players[p.index()].cards_drawn_this_turn - before_draw,
            2
        );
        assert_eq!(
            g.state.players[p.index()].cards_discarded_this_turn - before_discard,
            1
        );
    }
    assert_eq!(
        g.object_view(source).owner,
        if foreign { p.other() } else { p }
    );
    if g.on_battlefield(source) {
        assert_eq!(g.object_view(source).controller, p);
    }
}
#[test]
fn all_twelve_whole_role_consumers_are_paid_in_both_seats_and_foreign_origins() {
    for p in [Player::P0, Player::P1] {
        for foreign in [false, true] {
            for i in 0..NAMES.len() {
                complete_case(p, i, foreign);
            }
        }
    }
}
#[test]
fn cut_in_lethal_same_recipient_creates_role_before_sba_with_entry_death_and_counter_witnesses() {
    for p in [Player::P0, Player::P1] {
        for foreign in [false, true] {
            let mut g = setup(p);
            let recipient = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let marker = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            named(&mut g, p, "Rimefur Reindeer", Zone::Battlefield);
            named(&mut g, p, "Ashiok's Reaper", Zone::Battlefield);
            let savior = named(&mut g, p, "Savior of the Sleeping", Zone::Battlefield);
            let drawn = g.state.players[p.index()].cards_drawn_this_turn;
            announce(&mut g, p, "Cut In", foreign);
            drive(
                &mut g,
                &[Target::Card(recipient), Target::Card(marker)],
                true,
                None,
                true,
            );
            assert_eq!(g.object_view(recipient).zone, Zone::Graveyard);
            assert_eq!(total_roles(&g, RoleKind::YoungHero), 1);
            assert!(g.object_view(marker).tapped);
            assert_eq!(g.object_view(savior).counters.plus_one, 1);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, drawn + 1);
        }
    }
}

fn finish_announcement(g: &mut Game, targets: &[Target], bargain: Option<CardId>) {
    for _ in 0..80 {
        if g.state.announcing.is_none() && g.state.pending == Pending::Priority {
            return;
        }
        let a = pick(g, targets, true, bargain, true);
        act(g, a);
    }
    panic!("announcement");
}
fn wait_payment(g: &mut Game, targets: &[Target]) {
    for _ in 0..100 {
        if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
            return;
        }
        let a = pick(g, targets, true, None, true);
        act(g, a);
    }
    panic!("no optional payment");
}
fn reenter_under(g: &mut Game, id: CardId, p: Player) {
    let mut incoming = g.entry_move(id, Zone::Battlefield, g.enters_tapped_for(id, p));
    incoming.controller = p;
    g.stage_entries(vec![incoming], None);
    g.advance_entries();
}
#[test]
fn inline_source_roles_never_target_and_do_not_rebind_after_source_leave_and_reentry() {
    for p in [Player::P0, Player::P1] {
        for foreign in [false, true] {
            for name in ["Cursed Courtier", "Unassuming Sage"] {
                let mut g = setup(p);
                let ally = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let source = announce(&mut g, p, name, foreign);
                finish_announcement(&mut g, &[], None);
                act(&mut g, Action::Pass);
                act(&mut g, Action::Pass);
                assert!(g.on_battlefield(source));
                assert!(
                    g.state
                        .stack
                        .iter()
                        .any(|v| v.card == source && v.targets.is_empty())
                );
                let original = g.object_ref(source);
                g.move_object(source, Zone::Hand);
                reenter_under(&mut g, source, p);
                assert_ne!(g.object_ref(source), original);
                g.prepare_triggers();
                for _ in 0..150 {
                    if idle(&g) {
                        break;
                    }
                    let pay = g
                        .state
                        .resolving
                        .as_ref()
                        .is_some_and(|r| r.item.object == original);
                    let a = pick(&g, &[], pay, None, true);
                    assert!(!matches!(a, Action::ChooseTarget { .. }));
                    act(&mut g, a);
                }
                assert!(idle(&g));
                let kind = if name == "Cursed Courtier" {
                    RoleKind::Cursed
                } else {
                    RoleKind::Sorcerer
                };
                assert_eq!(
                    total_roles(&g, kind),
                    if name == "Cursed Courtier" { 1 } else { 0 }
                );
                assert!(roles(&g, ally, kind, p).is_empty());
            }
        }
    }
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let source = announce(&mut g, p, "Unassuming Sage", false);
            wait_payment(&mut g, &[]);
            assert!(g.state.resolving.as_ref().unwrap().item.targets.is_empty());
            let other = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            deny(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(other),
                },
            );
            drive(&mut g, &[], pay, None, true);
            assert_eq!(
                roles(&g, source, RoleKind::Sorcerer, p).len(),
                usize::from(pay)
            );
        }
    }
}
#[test]
fn paid_reflexive_roles_bind_late_then_offer_priority_even_after_source_leaves() {
    for p in [Player::P0, Player::P1] {
        for name in ["Merry Bards", "Spellbook Vendor"] {
            for foreign in [false, true] {
                let mut g = setup(p);
                let early = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let late = named(&mut g, p, "Hill Giant", Zone::Battlefield);
                let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                let source = announce(&mut g, p, name, foreign);
                if name == "Spellbook Vendor" {
                    drive(&mut g, &[], true, None, true);
                    g.state.active = p.other();
                    g.begin_step(Step::BeginCombat);
                    g.give_priority();
                    drive(&mut g, &[], true, None, true);
                    assert_eq!(total_roles(&g, RoleKind::Sorcerer), 0);
                    g.state.active = p;
                    g.begin_step(Step::BeginCombat);
                    g.give_priority();
                }
                wait_payment(&mut g, &[]);
                assert!(g.state.resolving.as_ref().unwrap().item.targets.is_empty());
                deny(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(early),
                    },
                );
                g.move_object(source, Zone::Hand);
                let before = g.state.players[p.index()].mana.iter().sum::<u32>();
                let payment = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayResolutionMana { .. }))
                    .unwrap();
                act(&mut g, payment);
                assert_eq!(
                    before - g.state.players[p.index()].mana.iter().sum::<u32>(),
                    1
                );
                assert!(g.state.announcing.as_ref().unwrap().trigger);
                g.move_object(early, Zone::Hand);
                deny(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(early),
                    },
                );
                deny(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(foe),
                    },
                );
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(late),
                    },
                );
                assert_eq!(g.state.pending, Pending::Priority);
                let kind = if name == "Merry Bards" {
                    RoleKind::YoungHero
                } else {
                    RoleKind::Sorcerer
                };
                assert_eq!(total_roles(&g, kind), 0);
                act(&mut g, Action::Pass);
                assert_eq!(total_roles(&g, kind), 0);
                drive(&mut g, &[], true, None, true);
                assert_eq!(roles(&g, late, kind, p).len(), 1);
                assert_eq!(g.object_view(source).zone, Zone::Hand);
            }
        }
    }
    for p in [Player::P0, Player::P1] {
        for name in ["Merry Bards", "Spellbook Vendor"] {
            let mut g = setup(p);
            let ally = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            announce(&mut g, p, name, false);
            if name == "Spellbook Vendor" {
                drive(&mut g, &[], true, None, true);
                g.begin_step(Step::BeginCombat);
                g.give_priority();
            }
            drive(&mut g, &[Target::Card(ally)], false, None, true);
            assert_eq!(
                total_roles(
                    &g,
                    if name == "Merry Bards" {
                        RoleKind::YoungHero
                    } else {
                        RoleKind::Sorcerer
                    }
                ),
                0
            );
        }
    }
}
#[test]
fn independent_role_target_groups_revalidate_exact_incarnations_and_all_illegal_fizzles() {
    for p in [Player::P0, Player::P1] {
        for name in ["Cut In", "Shatter the Oath"] {
            for dead_primary in [false, true] {
                for dead_role in [false, true] {
                    let mut g = setup(p);
                    let foe = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
                    let ally = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                    announce(&mut g, p, name, false);
                    finish_announcement(&mut g, &[Target::Card(foe), Target::Card(ally)], None);
                    if dead_primary {
                        g.move_object(foe, Zone::Hand);
                        g.move_object(foe, Zone::Battlefield);
                    }
                    if dead_role {
                        g.move_object(ally, Zone::Hand);
                        g.move_object(ally, Zone::Battlefield);
                    }
                    drive(&mut g, &[], true, None, true);
                    assert_eq!(g.on_battlefield(foe), dead_primary);
                    assert_eq!(
                        roles(
                            &g,
                            ally,
                            if name == "Cut In" {
                                RoleKind::YoungHero
                            } else {
                                RoleKind::Wicked
                            },
                            p
                        )
                        .len(),
                        usize::from(!dead_role)
                    );
                }
            }
        }
    }
    for p in [Player::P0, Player::P1] {
        for invalid in [false, true] {
            let mut g = setup(p);
            let foe = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            announce(&mut g, p, "Cut In", false);
            finish_announcement(&mut g, &[Target::Card(foe)], None);
            if invalid {
                g.move_object(foe, Zone::Hand);
                g.move_object(foe, Zone::Battlefield);
            }
            drive(&mut g, &[], true, None, true);
            assert_eq!(g.on_battlefield(foe), invalid);
            assert_eq!(total_roles(&g, RoleKind::YoungHero), 0);
        }
    }
}
#[test]
fn witch_mark_optional_discard_is_private_and_independent_role_survives_decline_but_not_fizzle() {
    for p in [Player::P0, Player::P1] {
        for choose_role in [false, true] {
            for invalidate in [false, true] {
                for discard in [false, true] {
                    let mut g = setup(p);
                    let ally = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                    let before_draw = g.state.players[p.index()].cards_drawn_this_turn;
                    let before_discard = g.state.players[p.index()].cards_discarded_this_turn;
                    announce(&mut g, p, "Witch's Mark", false);
                    finish_announcement(
                        &mut g,
                        &if choose_role {
                            vec![Target::Card(ally)]
                        } else {
                            vec![]
                        },
                        None,
                    );
                    if choose_role && invalidate {
                        g.move_object(ally, Zone::Hand);
                        g.move_object(ally, Zone::Battlefield);
                    }
                    for _ in 0..100 {
                        if idle(&g) {
                            break;
                        }
                        if matches!(g.state.pending, Pending::EffectChoice { .. }) {
                            assert!(g.observe(p).effect_choice.is_some());
                            assert!(g.observe(p.other()).effect_choice.is_none());
                            assert!(g.observe(p.other()).actions.is_empty());
                            let opponent_hand = g
                                .state
                                .objects
                                .iter()
                                .enumerate()
                                .find(|(_, o)| o.owner == p.other() && o.zone == Zone::Hand)
                                .unwrap()
                                .0 as CardId;
                            deny(
                                &mut g,
                                Action::Discard {
                                    card: opponent_hand,
                                },
                            );
                        }
                        let a = pick(&g, &[], true, None, discard);
                        act(&mut g, a);
                    }
                    assert!(idle(&g));
                    let effect = discard && !(choose_role && invalidate);
                    assert_eq!(
                        g.state.players[p.index()].cards_drawn_this_turn - before_draw,
                        if effect { 2 } else { 0 }
                    );
                    assert_eq!(
                        g.state.players[p.index()].cards_discarded_this_turn - before_discard,
                        u64::from(effect)
                    );
                    assert_eq!(
                        roles(&g, ally, RoleKind::Wicked, p).len(),
                        usize::from(choose_role && !invalidate)
                    );
                }
            }
        }
    }
}
#[test]
fn become_brutes_two_opposing_recipients_queue_replacements_and_keep_one_new_role_each() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ally = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        announce(&mut g, p, "Become Brutes", true);
        drive(
            &mut g,
            &[Target::Card(ally), Target::Card(foe)],
            true,
            None,
            true,
        );
        assert_eq!(total_roles(&g, RoleKind::Monster), 4);
        for recipient in [ally, foe] {
            assert_eq!(roles(&g, recipient, RoleKind::Monster, p).len(), 1);
            assert!(g.keywords(recipient).contains(Keywords::HASTE));
            assert!(g.keywords(recipient).contains(Keywords::TRAMPLE));
            assert_eq!(g.stats(recipient).power, 3);
        }
    }
}
#[test]
fn bargain_requires_actual_sacrifice_and_does_not_inherit_on_reentry_but_old_trigger_survives() {
    for p in [Player::P0, Player::P1] {
        for bargain in [false, true] {
            let mut g = setup(p);
            let fodder = named(&mut g, p, "Candy Trail", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let source = announce(&mut g, p, "Diminisher Witch", true);
            finish_announcement(&mut g, &[], if bargain { Some(fodder) } else { None });
            act(&mut g, Action::Pass);
            act(&mut g, Action::Pass);
            assert!(g.on_battlefield(source));
            g.move_object(source, Zone::Hand);
            reenter_under(&mut g, source, p);
            drive(&mut g, &[Target::Card(foe)], true, None, true);
            assert_eq!(
                roles(&g, foe, RoleKind::Cursed, p).len(),
                usize::from(bargain)
            );
            assert_eq!(g.on_battlefield(fodder), !bargain);
        }
    }
}

#[test]
fn whisper_player_group_remains_live_when_optional_role_incarnation_is_lost() {
    for p in [Player::P0, Player::P1] {
        for choose_role in [false, true] {
            let mut g = setup(p);
            let ally = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[ally as usize].controller = p;
            let hand = g.hand_count(p.other());
            announce(&mut g, p, "Eriette's Whisper", true);
            let mut targets = vec![Target::Player(p.other())];
            if choose_role {
                targets.push(Target::Card(ally));
            }
            finish_announcement(&mut g, &targets, None);
            if choose_role {
                g.move_object(ally, Zone::Hand);
                reenter_under(&mut g, ally, p);
            }
            drive(&mut g, &[], true, None, true);
            assert_eq!(hand - g.hand_count(p.other()), 2);
            assert_eq!(total_roles(&g, RoleKind::Wicked), 0);
        }
    }
}
