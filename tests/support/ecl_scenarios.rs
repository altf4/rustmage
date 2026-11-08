//! Shared scenario conventions retained from the accepted Lorwyn implementation.
#![allow(dead_code)]
use rustmage_core::cards::*;
use rustmage_core::engine::*;

pub(super) fn spawn(g: &mut Game, player: Player, card: Card, zone: Zone) -> CardId {
    crate::support::spawn(g, player, card, zone)
}

pub(super) fn scenario() -> Game {
    let mut game = crate::support::scenario(Player::P0);
    game.state.objects.clear();
    for player in &mut game.state.players {
        player.library.clear();
        player.mana = [0; 6];
    }
    for player in [Player::P0, Player::P1] {
        for _ in 0..20 {
            spawn(&mut game, player, Card::Forest, Zone::Library);
        }
    }
    game
}

pub(super) fn simple_cast(g: &mut Game, card: CardId, target: Option<Target>) {
    crate::support::cast(g, card, target);
}

pub(super) fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
pub(super) fn checked(g: &mut Game, action: Action) {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap_or_else(|| panic!("illegal independent {action:?} {:?}", g.state.pending));
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
    let evaluated = ability_evaluation::evaluate_abilities(g);
    for (i, object) in g.state.objects.iter().enumerate() {
        if object.zone == Zone::Battlefield {
            assert_eq!(
                evaluated.result.objects[i].keywords,
                g.keywords(i as CardId),
                "actual ordered keyword projection at {i}"
            );
        }
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
fn settle_on(g: &mut Game, chosen: Option<Target>) {
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
        let action = if let Some(target) =
            chosen.filter(|t| actions.contains(&Action::ChooseTarget { target: *t }))
        {
            Action::ChooseTarget { target }
        } else if g.state.pending == Pending::Priority {
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
pub(super) fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for state in &mut g.state.players {
        state.mana = [30; 6];
    }
    g
}

pub(super) fn settle(g: &mut Game) {
    settle_on(g, None);
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
pub(super) fn cast(g: &mut Game, id: CardId, mode: Option<u16>, targets: &[Target]) {
    checked(g, Action::AnnounceCast { card: id });
    if let Some(mode) = mode {
        reject(g, Action::ChooseMode { mode: u16::MAX });
        checked(g, Action::ChooseMode { mode });
    }
    for &target in targets {
        for _ in 0..3 {
            if g.legal_actions().contains(&Action::ChooseTarget { target }) {
                break;
            }
            checked(g, Action::FinishTargets);
        }
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets))
            .or_else(|| {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. }))
            })
            .unwrap();
        checked(g, a);
    }
    panic!("unpaid");
}

pub(super) fn next_turn(g: &mut Game) {
    let turn = g.state.turn;
    for _ in 0..100 {
        if g.state.turn != turn {
            return;
        }
        let actions = g.legal_actions();
        let action = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                )
            })
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        checked(g, action);
    }
    panic!("turn failed to advance");
}
pub(super) fn enter_growth(g: &mut Game, p: Player, land: CardId, color: usize) -> CardId {
    let aura = named(g, p, "Shimmerwilds Growth", Zone::Hand);
    cast(g, aura, None, &[Target::Card(land)]);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    assert_eq!(g.actor(), Some(p));
    checked(
        g,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::Color(rustmage_core::cards::ChoiceColor::ALL[color]),
        },
    );
    settle(g);
    aura
}
