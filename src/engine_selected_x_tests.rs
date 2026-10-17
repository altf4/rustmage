//! Chosen-face scalar domains, real payments and frozen mass effects.
use super::tests::{scenario, spawn};
use super::*;
const NAME: &str = "Scavenger Regent // Exude Toxin";
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    g.state.players[p.index()].mana = [0, 0, 5, 0, 0, 0];
    g
}
fn act(g: &mut Game, a: Action) {
    let actor = g.actor().unwrap();
    let saved = g.snapshot();
    let rev = g.revision();
    let index = g.legal_actions().iter().position(|x| *x == a).unwrap();
    assert_eq!(g.apply(actor.other(), a.clone()), Err(Error::WrongActor));
    assert_eq!(saved, g.snapshot());
    let mut child = {
        let parent = g.clone();
        let c = parent.clone();
        drop(parent);
        c
    };
    child.apply_index(actor, rev, index).unwrap();
    g.apply(actor, a.clone()).unwrap();
    assert_eq!(g.snapshot(), child.snapshot());
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(viewer)).unwrap(),
            serde_json::to_value(child.observe(viewer)).unwrap()
        );
    }
    let done = g.snapshot();
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(actor, rev, index), Err(Error::StaleRevision));
    g.apply(actor, a).unwrap();
    assert_eq!(g.snapshot(), done);
}
fn choose(g: &mut Game) {
    let actions = g.legal_actions();
    let a = actions
        .iter()
        .find(|a| matches!(a, Action::FinishTargets | Action::FinishSearch))
        .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
        .unwrap()
        .clone();
    act(g, a);
}
fn stack(g: &mut Game, id: CardId) {
    for _ in 0..80 {
        if g.state.pending == Pending::Priority && g.state.stack.iter().any(|s| s.card == id) {
            return;
        }
        choose(g);
    }
    panic!("announcement");
}
fn settle(g: &mut Game) {
    for _ in 0..200 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        choose(g);
    }
    panic!("resolution");
}
#[test]
fn selected_x_uses_inset_bb_cost_and_freezes_non_dragon_creatures_both_seats() {
    for p in [Player::P0, Player::P1] {
        for x in 0..=3 {
            let mut g = setup(p);
            let normal = Card::from_name(NAME).unwrap();
            let inset = normal.omen_spell().unwrap();
            let id = spawn(&mut g, p, normal, Zone::Hand);
            let mut bears = Vec::new();
            let mut dragons = Vec::new();
            let mut changelings = Vec::new();
            for owner in [p, p.other()] {
                bears.push(spawn(
                    &mut g,
                    owner,
                    Card::from_name("Bear Cub").unwrap(),
                    Zone::Battlefield,
                ));
                dragons.push(spawn(&mut g, owner, normal, Zone::Battlefield));
                changelings.push(spawn(
                    &mut g,
                    owner,
                    Card::from_name("Three Tree Mascot").unwrap(),
                    Zone::Battlefield,
                ));
            }
            assert_eq!(
                g.scalar_x_intervals_for_definition(p, id, inset),
                vec![XInterval {
                    minimum: 0,
                    maximum: 3
                }]
            );
            act(&mut g, Action::AnnounceOmen { card: id });
            assert_eq!(g.observe(p).decision, "x");
            assert_eq!(
                g.x_choice().unwrap().intervals,
                vec![XInterval {
                    minimum: 0,
                    maximum: 3
                }]
            );
            let before = g.snapshot();
            assert!(g.apply(p, Action::ChooseX { value: 4 }).is_err());
            assert_eq!(before, g.snapshot());
            act(&mut g, Action::ChooseX { value: x });
            stack(&mut g, id);
            let spell = g.state.stack.iter().find(|s| s.card == id).unwrap();
            assert_eq!(spell.source_card, inset);
            assert_eq!(spell.mana_value(), 2 + u64::from(x));
            assert_eq!(spell.x, Some(x));
            assert_eq!(g.state.players[p.index()].mana, [0, 0, 3 - x, 0, 0, 0]);
            let count = g.state.objects.len();
            settle(&mut g);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Library);
            assert_eq!(g.state.objects.len(), count);
            assert_eq!(
                g.state.players[p.index()]
                    .library
                    .iter()
                    .filter(|&&c| c == id)
                    .count(),
                1
            );
            assert!(g.state.adventures.is_none());
            for bear in bears {
                if x >= 2 {
                    assert_eq!(g.state.objects[bear as usize].zone, Zone::Graveyard);
                } else {
                    assert_eq!(g.stats(bear).toughness, 2 - i128::from(x));
                }
            }
            for dragon in dragons {
                assert_eq!(g.state.objects[dragon as usize].zone, Zone::Battlefield);
                assert_eq!(g.stats(dragon).toughness, 4);
            }
            for changeling in changelings {
                assert_eq!(g.state.objects[changeling as usize].zone, Zone::Battlefield);
                assert_eq!(g.stats(changeling).toughness, 1);
            }
            let later = spawn(
                &mut g,
                p,
                Card::from_name("Bear Cub").unwrap(),
                Zone::Battlefield,
            );
            assert_eq!(g.stats(later).toughness, 2);
        }
    }
}
#[test]
fn selected_face_mana_value_counter_boundary_and_countered_omen_does_not_shuffle() {
    for p in [Player::P0, Player::P1] {
        for x in [0, 1] {
            let mut g = setup(p);
            let id = spawn(&mut g, p, Card::from_name(NAME).unwrap(), Zone::Hand);
            let response = spawn(
                &mut g,
                p.other(),
                Card::from_name("Runescale Stormbrood // Chilling Screech").unwrap(),
                Zone::Hand,
            );
            g.state.players[p.other().index()].mana = [30; 6];
            act(&mut g, Action::AnnounceOmen { card: id });
            act(&mut g, Action::ChooseX { value: x });
            stack(&mut g, id);
            act(&mut g, Action::Pass);
            assert_eq!(g.actor(), Some(p.other()));
            assert_eq!(
                g.legal_actions()
                    .contains(&Action::AnnounceOmen { card: response }),
                x == 0
            );
            if x == 0 {
                let library = g.state.players[p.index()].library.clone();
                act(&mut g, Action::AnnounceOmen { card: response });
                stack(&mut g, response);
                settle(&mut g);
                assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
                assert_eq!(g.state.players[p.index()].library, library);
                assert_eq!(g.state.objects[response as usize].zone, Zone::Library);
                assert!(g.state.adventures.is_none());
            }
        }
    }
}
#[test]
fn chosen_x_off_turn_permission_does_not_relax_funding_or_actor() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.active = p.other();
        let id = spawn(&mut g, p, Card::from_name(NAME).unwrap(), Zone::Hand);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceOmen { card: id })
        );
        spawn(
            &mut g,
            p,
            Card::from_name("Whirlwing Stormbrood // Dynamic Soar").unwrap(),
            Zone::Battlefield,
        );
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceOmen { card: id })
        );
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceOmen { card: id })
        );
    }
}

#[test]
fn actual_paid_normal_scavenger_flying_and_discard_ward_remain_complete() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let id = spawn(&mut g, p, Card::from_name(NAME).unwrap(), Zone::Hand);
            let shock = spawn(&mut g, p.other(), Card::Shock, Zone::Hand);
            let resource = spawn(&mut g, p.other(), Card::Forest, Zone::Hand);
            g.state.players[p.other().index()].mana = [30; 6];
            act(&mut g, Action::AnnounceCast { card: id });
            stack(&mut g, id);
            settle(&mut g);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
            assert_eq!((g.stats(id).power, g.stats(id).toughness), (4, 4));
            assert!(g.keywords(id).contains(Keywords::FLYING));
            assert_eq!(g.state.players[p.index()].mana, [0, 0, 1, 0, 0, 0]);
            act(&mut g, Action::Pass);
            act(&mut g, Action::AnnounceCast { card: shock });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            stack(&mut g, shock);
            act(&mut g, Action::Pass);
            act(&mut g, Action::Pass);
            assert!(matches!(g.state.pending, Pending::EffectChoice { .. }));
            assert_eq!(g.actor(), Some(p.other()));
            assert!(g.observe(p).effect_choice.is_none());
            if pay {
                act(&mut g, Action::Discard { card: resource });
            } else {
                act(&mut g, Action::DeclineEffectPayment);
            }
            settle(&mut g);
            assert_eq!(g.state.objects[id as usize].damage, if pay { 2 } else { 0 });
            assert_eq!(
                g.state.objects[resource as usize].zone,
                if pay { Zone::Graveyard } else { Zone::Hand }
            );
            assert_eq!(g.state.objects[shock as usize].zone, Zone::Graveyard);
        }
    }
}

#[test]
fn selected_x_billion_wide_domain_and_paid_endpoint_are_bounded() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 0, u32::MAX, 0, 0, 0];
        let id = spawn(&mut g, p, Card::from_name(NAME).unwrap(), Zone::Hand);
        let bear = spawn(
            &mut g,
            p.other(),
            Card::from_name("Bear Cub").unwrap(),
            Zone::Battlefield,
        );
        act(&mut g, Action::AnnounceOmen { card: id });
        assert_eq!(
            g.x_choice().unwrap().intervals,
            vec![XInterval {
                minimum: 0,
                maximum: u32::MAX - 2
            }]
        );
        assert!(g.legal_actions().len() <= 4);
        let before = g.snapshot();
        assert!(g.apply(p, Action::ChooseX { value: u32::MAX }).is_err());
        assert_eq!(g.snapshot(), before);
        act(
            &mut g,
            Action::ChooseX {
                value: u32::MAX - 2,
            },
        );
        stack(&mut g, id);
        assert_eq!(
            g.state
                .stack
                .iter()
                .find(|s| s.card == id)
                .unwrap()
                .mana_value(),
            u64::from(u32::MAX)
        );
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Library);
    }
}
#[test]
fn scaled_temporary_mass_stats_expire_and_later_entrants_are_unaffected() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = spawn(&mut g, p, Card::from_name(NAME).unwrap(), Zone::Hand);
        let bear = spawn(
            &mut g,
            p,
            Card::from_name("Bear Cub").unwrap(),
            Zone::Battlefield,
        );
        act(&mut g, Action::AnnounceOmen { card: id });
        act(&mut g, Action::ChooseX { value: 1 });
        stack(&mut g, id);
        settle(&mut g);
        assert_eq!(g.stats(bear).toughness, 1);
        let later = spawn(
            &mut g,
            p,
            Card::from_name("Bear Cub").unwrap(),
            Zone::Battlefield,
        );
        assert_eq!(g.stats(later).toughness, 2);
        let turn = g.state.turn;
        for _ in 0..100 {
            if g.state.turn > turn {
                break;
            }
            choose(&mut g);
        }
        assert!(g.state.turn > turn);
        assert_eq!(g.stats(bear).toughness, 2);
        assert_eq!(g.stats(later).toughness, 2);
    }
}
