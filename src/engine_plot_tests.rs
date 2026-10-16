use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 13] = [
    "Beastbond Outcaster",
    "Blacksnag Buzzard",
    "Cunning Coyote",
    "Djinn of Fool's Fall",
    "Irascible Wolverine",
    "Loan Shark",
    "Slickshot Show-Off",
    "Spinewoods Paladin",
    "Stagecoach Security",
    "Stingerback Terror",
    "Demonic Ruckus",
    "Rictus Robber",
    "Rise of the Varmints",
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
    g.state.players[p.index()].mana = [40; 6];
    g
}
fn replay(g: &mut Game, a: Action) {
    assert!(
        g.legal_actions().contains(&a),
        "{a:?} {:?}",
        g.state.pending
    );
    let old = g.snapshot();
    let rev = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let after = g.snapshot();
    assert_eq!(f.snapshot(), after);
    g.restore(&old).unwrap();
    assert_eq!(g.apply_index(p, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), after);
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
fn pick(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .min_by_key(|a| match a {
            Action::ChooseTarget {
                target: Target::Card(_),
            } => 0,
            Action::ArrangeLibrary {
                destination: LibraryDestination::Graveyard,
                ..
            } => 0,
            Action::ChooseSpellCost { .. } => 0,
            Action::PayCosts { .. } | Action::PayPlot { .. } => 1,
            Action::PutTrigger { .. } => 1,
            Action::ChooseMobilizeDefender {
                defender: CombatDefender::Player(_),
            } => 0,
            Action::FinishTargets => 2,
            Action::Pass => 3,
            Action::Concede => 10000,
            _ => 100,
        })
        .unwrap();
    replay(g, a);
}
fn idle(g: &Game) -> bool {
    g.state.stack.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.pending == Pending::Priority
}
fn settle(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..100 {
        if idle(g) {
            return;
        }
        pick(g);
    }
    panic!("unsettled {:?}", g.state.pending)
}

fn plot(g: &mut Game, p: Player, id: CardId) {
    let old = g.object_ref(id);
    let card = g.state.objects[id as usize].card;
    let spells = g.state.players[p.index()].spells_cast_this_turn;
    let entries = g.state.players[p.index()].nonland_permanents_entered_this_turn;
    let next = g.state.next_stack_id;
    replay(g, Action::AnnouncePlot { card: id });
    assert!(g.observe(p).plot_payment.is_some());
    assert!(g.observe(p.other()).plot_payment.is_none());
    assert_eq!(g.object_ref(id), old);
    assert_eq!(g.state.objects[id as usize].zone, Zone::Hand);
    assert!(g.state.stack.is_empty());
    assert!(g.state.announcing.is_none());
    assert_eq!(g.apply(p.other(), Action::Pass), Err(Error::WrongActor));
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayPlot { .. }))
        .unwrap();
    let before = g.state.players[p.index()].mana;
    replay(g, a);
    assert_eq!(g.state.priority, p);
    assert_eq!(g.state.players[p.index()].spells_cast_this_turn, spells);
    assert_eq!(
        g.state.players[p.index()].nonland_permanents_entered_this_turn,
        entries
    );
    assert_eq!(g.state.next_stack_id, next);
    assert_eq!(g.state.objects[id as usize].zone, Zone::Exile);
    assert_eq!(g.object_ref(id).generation, old.generation + 1);
    let perms = g.observe(p).plotted_cards;
    assert_eq!(perms, g.observe(p.other()).plotted_cards);
    assert!(
        perms.iter().any(|x| x.object == g.object_ref(id)
            && x.owner == p
            && x.plotted_turn == g.state.turn)
    );
    assert_eq!(
        before.iter().sum::<u32>() - g.state.players[p.index()].mana.iter().sum::<u32>(),
        card.plot_cost().unwrap().colored.iter().sum::<u32>() + card.plot_cost().unwrap().generic
    );
    reject(g, Action::AnnouncePlotted { card: id });
    reject(g, Action::AnnounceCast { card: id });
}
fn cast(g: &mut Game, id: CardId, plotted: bool) {
    if plotted {
        g.state.turn += 1;
    }
    replay(
        g,
        if plotted {
            Action::AnnouncePlotted { card: id }
        } else {
            Action::AnnounceCast { card: id }
        },
    );
    while g.state.announcing.is_some() {
        pick(g);
    }
    let spell = g
        .state
        .stack
        .iter()
        .find(|s| s.card == id && s.kind == StackKind::Spell)
        .unwrap();
    assert_eq!(
        spell.cast.unwrap().method,
        if plotted {
            CastMethod::Plot
        } else {
            CastMethod::Normal
        }
    );
    assert_eq!(
        spell.cast.unwrap().origin,
        if plotted { Zone::Exile } else { Zone::Hand }
    );
    assert_eq!(spell.x, None);
    assert!(!g.state.objects[id as usize].flashback);
    settle(g);
}
#[test]
fn all_thirteen_full_bodies_hardcast_and_plotted_both_seats() {
    for p in [Player::P0, Player::P1] {
        for plotted in [false, true] {
            for (i, n) in NAMES.iter().enumerate() {
                let mut g = setup(p);
                let ally = named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
                named(&mut g, p, "Bear Cub", Zone::Graveyard);
                g.state.creatures_died_this_turn = 1;
                g.state.players[p.index()].spells_cast_this_turn = 1;
                let id = named(&mut g, p, n, Zone::Hand);
                let library = g.state.players[p.index()].library.len();
                if plotted {
                    plot(&mut g, p, id);
                }
                cast(&mut g, id, plotted);
                assert_eq!(
                    g.state.objects[id as usize].zone,
                    if i == 12 {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    },
                    "{n}"
                );
                assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 2, "{n}");
                assert!(g.observe(p).plotted_cards.is_empty());
                match i {
                    0 | 5 => {
                        assert_eq!(g.state.players[p.index()].library.len(), library - 1, "{n}")
                    }
                    1 => assert_eq!((g.stats(id).power, g.stats(id).toughness), (3, 2)),
                    2 => assert_eq!((g.stats(ally).power, g.stats(ally).toughness), (11, 11)),
                    4 => assert_eq!(g.state.exile_permissions.len(), 1),
                    7 => assert_eq!(g.state.players[p.index()].life, 23),
                    8 => assert_eq!((g.stats(ally).power, g.stats(ally).toughness), (11, 11)),
                    9 => assert_eq!((g.stats(id).power, g.stats(id).toughness), (7, 7)),
                    10 => {
                        assert_eq!((g.stats(ally).power, g.stats(ally).toughness), (11, 11));
                        assert!(g.keywords(ally).contains(Keywords::MENACE));
                        assert!(g.keywords(ally).contains(Keywords::TRAMPLE));
                    }
                    11 | 12 => {
                        let tokens: Vec<_> = g
                            .state
                            .objects
                            .iter()
                            .filter(|o| {
                                o.owner == p && o.zone == Zone::Battlefield && o.card.is_token()
                            })
                            .collect();
                        assert_eq!(tokens.len(), 1, "{n}");
                        assert_eq!(tokens[0].card.colors(), if i == 11 { 6 } else { 16 });
                    }
                    _ => {}
                }
            }
        }
    }
}
#[test]
fn strict_turn_main_empty_stack_actor_and_exact_permission() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Djinn of Fool's Fall", Zone::Hand);
        for step in [Step::Upkeep, Step::End, Step::CombatDamage] {
            g.state.step = step;
            reject(&mut g, Action::AnnouncePlot { card: id });
        }
        g.state.step = Step::Main1;
        plot(&mut g, p, id);
        g.state.turn += 1;
        g.state.active = p.other();
        reject(&mut g, Action::AnnouncePlotted { card: id });
        g.state.active = p;
        for step in [Step::Upkeep, Step::End, Step::CombatDamage] {
            g.state.step = step;
            reject(&mut g, Action::AnnouncePlotted { card: id });
        }
        g.state.step = Step::Main2;
        assert!(
            g.legal_actions()
                .contains(&Action::AnnouncePlotted { card: id })
        );
        let old = g.object_ref(id);
        g.move_object(id, Zone::Hand);
        g.move_object(id, Zone::Exile);
        assert_ne!(old, g.object_ref(id));
        assert!(g.observe(p).plotted_cards.is_empty());
        reject(&mut g, Action::AnnouncePlotted { card: id });
    }
}
#[test]
fn unfunded_wrong_color_and_late_transition_capacity_restore_every_owned_field() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Spinewoods Paladin", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnouncePlot { card: id });
        g.state.players[p.index()].mana = [100, 0, 0, 0, 0, 0];
        reject(&mut g, Action::AnnouncePlot { card: id });
        g.state.players[p.index()].mana = [40; 6];
        replay(&mut g, Action::AnnouncePlot { card: id });
        g.state.objects[id as usize].generation = u64::MAX;
        g.state
            .plot
            .as_mut()
            .unwrap()
            .payment
            .as_mut()
            .unwrap()
            .object = g.object_ref(id);
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayPlot { .. }))
            .unwrap();
        let before = g.snapshot();
        let rev = g.revision();
        assert!(matches!(g.apply(p, a), Err(Error::CapacityExceeded(_))));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), rev);
        g.state.objects[id as usize].generation = 0;
        g.state
            .plot
            .as_mut()
            .unwrap()
            .payment
            .as_mut()
            .unwrap()
            .object = g.object_ref(id);
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayPlot { .. }))
            .unwrap();
        replay(&mut g, a);
        g.state.turn += 1;
        g.state.next_stack_id = u64::MAX;
        let before = g.snapshot();
        let rev = g.revision();
        assert!(matches!(
            g.apply(p, Action::AnnouncePlotted { card: id }),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), rev);
        assert_eq!(g.observe(p).plotted_cards.len(), 1);
    }
}
#[test]
fn real_land_mana_funds_special_action_without_consuming_hand_source_or_casting() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        let forest = named(&mut g, p, "Forest", Zone::Battlefield);
        let mountain = named(&mut g, p, "Mountain", Zone::Battlefield);
        let id = named(&mut g, p, "Slickshot Show-Off", Zone::Hand);
        let before = g.object_ref(id);
        replay(&mut g, Action::AnnouncePlot { card: id });
        reject(&mut g, Action::Pass);
        for land in [forest, mountain] {
            replay(&mut g, Action::TapMana { card: land });
            assert_eq!(g.object_ref(id), before);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Hand);
            assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 0);
            assert!(g.state.stack.is_empty());
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayPlot { .. }))
            .unwrap();
        replay(&mut g, a);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.priority, p);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 0);
    }
}
