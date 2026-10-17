use super::tests::{scenario, spawn};
use super::*;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn checked(g: &mut Game, action: Action) {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let before = g.snapshot();
    assert_eq!(
        g.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(g.snapshot(), before);
    let mut branch = {
        let parent = g.clone();
        parent.clone()
    };
    branch.apply_index(actor, revision, index).unwrap();
    g.apply(actor, action.clone()).unwrap();
    assert_eq!(g.snapshot(), branch.snapshot());
    for player in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(player)).unwrap(),
            serde_json::to_value(branch.observe(player)).unwrap()
        );
    }
    let after = g.snapshot();
    g.restore(&before).unwrap();
    assert_eq!(
        g.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    g.apply(actor, action).unwrap();
    assert_eq!(g.snapshot(), after);
}
fn settle(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..100 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .or_else(|| actions.iter().find(|a| matches!(a, Action::FinishTargets)))
                .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                .unwrap()
                .clone()
        };
        checked(g, action);
    }
    panic!("unsettled {:?}", g.state.pending);
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    g.state.players[p.index()].mana = [30; 6];
    g
}

fn paid_blessing(p: Player) -> (Game, CardId, CardId, CardId) {
    let mut g = setup(p);
    let recipient = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    let blessing = named(&mut g, p, "Lord Skitter's Blessing", Zone::Hand);
    let before = g.state.players[p.index()].mana;
    checked(&mut g, Action::AnnounceCast { card: blessing });
    settle(&mut g);
    assert!(g.on_battlefield(blessing));
    assert!(g.state.players[p.index()].mana.iter().sum::<u32>() < before.iter().sum::<u32>());
    let aura = *g
        .state
        .attachment_sources
        .iter()
        .find(|&&id| {
            let o = &g.state.objects[id as usize];
            o.card.role() == Some(crate::cards::RoleKind::Wicked)
                && o.attached_to == Some(g.object_ref(recipient))
        })
        .unwrap();
    assert!(g.is_enchanted(recipient));
    (g, blessing, recipient, aura)
}
fn draw_step(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.turn = 2;
    g.state.step = Step::Upkeep;
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    assert_eq!(g.state.step, Step::Draw);
}
fn paid_destroy(g: &mut Game, p: Player, target: CardId) {
    // Native fixture supplies mana; announcement and printed costs are real actions.
    g.state.players[p.index()].mana = [30; 6];
    g.state.priority = p;
    let spell = named(g, p, "Disenchant", Zone::Hand);
    checked(g, Action::AnnounceCast { card: spell });
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        checked(g, Action::FinishTargets);
    }
    for _ in 0..30 {
        if !g.on_battlefield(target) {
            return;
        }
        let actions = g.legal_actions();
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                .unwrap()
                .clone()
        };
        checked(g, action);
    }
    panic!("Disenchant failed");
}
#[test]
fn paid_blessing_full_role_and_extra_draw_after_normal_draw_both_seats() {
    for p in [Player::P0, Player::P1] {
        let (mut g, blessing, recipient, _) = paid_blessing(p);
        let hand = g.hand_count(p);
        let life = g.state.players[p.index()].life;
        let opponent_life = g.state.players[p.other().index()].life;
        draw_step(&mut g, p);
        assert_eq!(
            g.hand_count(p),
            hand + 1,
            "normal turn action precedes priority"
        );
        assert!(g.state.stack.iter().any(|item| item.card == blessing));
        assert_eq!(g.state.players[p.index()].life, life);
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 2);
        assert_eq!(g.state.players[p.index()].life, life - 1);
        assert_eq!(g.state.players[p.other().index()].life, opponent_life);
        assert_eq!(g.stats(recipient).power, 2);
    }
}
#[test]
fn enchanted_guard_rechecks_and_source_departure_does_not_counter_trigger() {
    for p in [Player::P0, Player::P1] {
        for destroy_source in [false, true] {
            let (mut g, blessing, _, aura) = paid_blessing(p);
            let hand = g.hand_count(p);
            let life = g.state.players[p.index()].life;
            draw_step(&mut g, p);
            paid_destroy(&mut g, p, if destroy_source { blessing } else { aura });
            assert!(g.state.stack.iter().any(|item| item.card == blessing));
            settle(&mut g);
            assert_eq!(g.hand_count(p), hand + 1 + usize::from(destroy_source));
            assert_eq!(
                g.state.players[p.index()].life,
                life - i128::from(destroy_source)
            );
        }
    }
}
#[test]
fn aura_controller_is_irrelevant_but_equipment_or_other_players_creature_do_not_qualify() {
    for p in [Player::P0, Player::P1] {
        for case in 0..4 {
            let (mut g, blessing, recipient, aura) = paid_blessing(p);
            match case {
                0 => {
                    g.state.objects[aura as usize].controller = p.other();
                }
                1 => {
                    g.move_object(aura, Zone::Graveyard);
                    let equipment = named(&mut g, p, "Goldvein Pick", Zone::Battlefield);
                    g.state.objects[equipment as usize].attached_to = Some(g.object_ref(recipient));
                }
                2 => {
                    // Auras attached to opponents' creatures do not satisfy your guard.
                    g.state.objects[recipient as usize].controller = p.other();
                }
                _ => {
                    g.move_object(aura, Zone::Graveyard);
                }
            }
            settle(&mut g);
            let hand = g.hand_count(p);
            let life = g.state.players[p.index()].life;
            draw_step(&mut g, p);
            assert_eq!(g.state.stack.iter().any(|i| i.card == blessing), case == 0);
            settle(&mut g);
            assert_eq!(g.hand_count(p), hand + 1 + usize::from(case == 0));
            assert_eq!(
                g.state.players[p.index()].life,
                life - i128::from(case == 0)
            );
        }
    }
}
#[test]
fn exact_attachment_incarnation_and_nonrole_aura_qualify_without_new_state() {
    for p in [Player::P0, Player::P1] {
        let (mut g, _, recipient, aura) = paid_blessing(p);
        let old = g.object_ref(recipient);
        let aura_card = Card::ALL
            .iter()
            .copied()
            .find(|c| c.aura().is_some() && c.role().is_none())
            .unwrap();
        let other = spawn(&mut g, p.other(), aura_card, Zone::Battlefield);
        g.state.objects[other as usize].attached_to = Some(old);
        g.move_object(aura, Zone::Graveyard);
        assert!(g.is_enchanted(recipient));
        let bytes = g.state_bytes();
        let clone = g.clone();
        assert_eq!(clone.state_bytes(), bytes);
        assert!(clone.is_enchanted(recipient));
        g.move_object(recipient, Zone::Exile);
        g.move_object(recipient, Zone::Battlefield);
        assert_ne!(g.object_ref(recipient), old);
        assert!(
            !g.is_enchanted(recipient),
            "stale Aura link cannot enchant new incarnation"
        );
        assert!(clone.is_enchanted(recipient));
    }
}
