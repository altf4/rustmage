//! Four whole candidates combine printed symbols, connive, modes and numeric cleanup.
use super::assurance::big_helpers::{checked, named, next_turn, settle, setup};
use super::*;
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..250 {
        if stop(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap_or_else(|| {
                panic!(
                    "combined unexpected {:?} {:?}",
                    g.state.pending, g.state.step
                )
            });
        checked(g, a);
    }
    panic!("combined stalled {:?}", g.state.pending);
}
fn cast(g: &mut Game, p: Player, name: &str, mana: Mana, targets: &[Target]) -> CardId {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
    let id = named(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    checked(g, Action::AnnounceCast { card: id });
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    while g
        .state
        .announcing
        .as_ref()
        .is_some_and(|a| a.item.card == id && a.item.kind == StackKind::Spell)
    {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn mode(g: &mut Game, n: u16) {
    until(g, |g| matches!(g.state.pending, Pending::Modes { .. }));
    checked(g, Action::ChooseMode { mode: n });
    settle(g);
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let before = g.snapshot();
    let rev = g.revision();
    let index = g.legal_actions().iter().position(|b| *b == a).unwrap();
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, rev, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
fn merfolk(g: &Game, p: Player) -> usize {
    (0..g.state.objects.len() as CardId)
        .filter(|&id| {
            let o = g.object_view(id);
            o.zone == Zone::Battlefield && o.controller == p && g.has_subtype(id, "Merfolk")
        })
        .count()
}
fn attack_ares(g: &mut Game, ares: CardId) {
    until(g, |g| g.state.pending == Pending::Attackers);
    checked(g, Action::Attack { card: ares });
    checked(g, Action::FinishAttackers);
}
#[test]
fn msh_tenth_combined_paid_blue_spells_connive_modes_and_two_natural_cleanup_limits() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for player in &mut g.state.players {
            player.mana = [0; 6];
        }
        let vision = cast(&mut g, p, "The Vision", [0, 0, 0, 0, 0, 4], &[]);
        settle(&mut g);
        cast(
            &mut g,
            p,
            "Baron Strucker, HYDRA Overlord",
            [0, 0, 1, 0, 0, 2],
            &[],
        );
        settle(&mut g);
        let namor = cast(&mut g, p, "Namor the Sub-Mariner", [0, 2, 0, 0, 0, 0], &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: false });
        settle(&mut g);
        assert_eq!(merfolk(&g, p), 1);
        assert_eq!(g.stats(namor).power, 1);
        cast(&mut g, p, "The Ten Rings", [0, 0, 0, 0, 0, 8], &[]);
        mode(&mut g, 1);
        assert_eq!(g.maximum_hand_size(p), Some(10));
        cast(
            &mut g,
            p,
            "I Am Iron Man",
            [0, 1, 0, 0, 0, 2],
            &[Target::Card(vision)],
        );
        mode(&mut g, 0);
        assert_eq!(merfolk(&g, p), 2);
        assert_eq!(g.stats(namor).power, 2);
        assert_eq!((g.stats(vision).power, g.stats(vision).toughness), (4, 4));
        let discard = named(&mut g, p, "Murder", Zone::Hand);
        let ares = cast(&mut g, p, "Ares, God of War", [0, 0, 1, 1, 0, 0], &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        g = orphan(g, Action::ChooseOptional { accept: true });
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectDiscard { .. })
        });
        g = orphan(g, Action::Discard { card: discard });
        settle(&mut g);
        assert_eq!(g.counter_count(ares, CounterKind::PlusOne), 1);
        cast(&mut g, p, "Divination", [0, 1, 0, 0, 0, 2], &[]);
        until(&mut g, |g| matches!(g.state.pending, Pending::Modes { .. }));
        g = orphan(g, Action::ChooseMode { mode: 2 });
        settle(&mut g);
        assert_eq!(merfolk(&g, p), 3);
        assert_eq!(g.stats(namor).power, 3);
        assert!(
            g.keywords(vision)
                .contains(Keywords::DOUBLE_STRIKE.union(Keywords::INDESTRUCTIBLE))
        );
        until(&mut g, |g| {
            g.state.step == Step::End
                && g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
        });
        assert_eq!(g.hand_count(p), 10);
        next_turn(&mut g);
        assert_eq!(g.state.active, p.other());
        assert_eq!(g.hand_count(p), 10);
        assert_eq!((g.stats(vision).power, g.stats(vision).toughness), (2, 5));
        assert!(!g.keywords(vision).contains(Keywords::DOUBLE_STRIKE));
        assert!(!g.keywords(vision).contains(Keywords::INDESTRUCTIBLE));
        assert!(g.state.modal_history.is_none());
        until(&mut g, |g| {
            g.state.step == Step::Main1 && g.state.pending == Pending::Priority
        });
        cast(
            &mut g,
            p,
            "Shock",
            [0, 0, 0, 1, 0, 0],
            &[Target::Player(p.other())],
        );
        mode(&mut g, 2);
        assert_eq!(g.hand_count(p), 11);
        assert_eq!(g.state.pending, Pending::Priority);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.step == Step::Main1 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.hand_count(p), 12);
        attack_ares(&mut g, ares);
        until(&mut g, |g| g.state.pending == Pending::Discard);
        for _ in 0..2 {
            assert_eq!(g.actor(), Some(p));
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Discard { .. }))
                .unwrap();
            g = orphan(g, action);
        }
        assert_eq!(g.hand_count(p), 10);
        assert_eq!(g.state.active, p.other());
    }
}
