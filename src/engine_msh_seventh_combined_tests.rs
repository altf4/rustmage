//! All eight candidates interact through real paid actions in both seats.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..150 {
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
            .unwrap_or_else(|| panic!("unexpected choice {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("combined scenario stalled {:?}", g.state.pending);
}
fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn pay(g: &mut Game) {
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
}
#[test]
fn msh_seventh_combined_paid_discard_types_first_tap_stun_timing_blink_and_power_up() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Captain America, Living Legend");
        paid(&mut g, p, "Captain Mar-Vell, Space-Born");
        let skrull = paid(&mut g, p, "Undercover Skrull");
        let jessica = paid(&mut g, p, "Jessica Jones, Private Eye");
        let quick = paid(&mut g, p, "Quicksilver, Brash Blur");
        let stature = paid(&mut g, p, "Stature, Size Shifter");
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.step == Step::Main1 && g.state.pending == Pending::Priority
        });
        g.state.players[p.index()].mana = [30; 6];
        assert!(!g.object_view(jessica).summoning_sick);
        assert_eq!(g.stats(skrull).power, 1);
        let blocker = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        assert!(!g.can_block_pair(blocker, stature));

        // Real X endpoint/payment: later-turn XUU, with X exactly one.
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 1];
        checked(&mut g, activate(stature));
        checked(&mut g, Action::ChooseX { value: 1 });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.stats(stature).power, 2);
        assert!(g.can_block_pair(blocker, stature));
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 4];
        checked(&mut g, activate(quick));
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.keywords(quick).contains(Keywords::DOUBLE_STRIKE));
        g.state.players[p.index()].mana = [30; 6];

        let car = named(&mut g, p, "S.H.I.E.L.D. Flying Car", Zone::Hand);
        let discard_a = named(&mut g, p, "Jessica Jones, Private Eye", Zone::Hand);
        let discard_b = named(&mut g, p, "Quicksilver, Brash Blur", Zone::Hand);
        let thirst = named(&mut g, p, "Thirst for Knowledge", Zone::Hand);
        cast(&mut g, thirst, None, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        assert!(g.observe(p.other()).effect_choice.is_none());
        checked(&mut g, Action::DeclineEffectPayment);
        checked(&mut g, Action::Discard { card: discard_a });
        checked(&mut g, Action::Discard { card: discard_b });
        settle(&mut g);
        assert_eq!(g.stats(skrull).power, 3);
        assert!(g.has_subtype(skrull, "Elf") && g.has_subtype(skrull, "Hero"));

        // The first-tap untap removes Jessica's actual cost stun counter, leaving
        // her tapped; the underlying activation still exiles exactly her power.
        named(&mut g, p, "Forest", Zone::Library);
        let divination = named(&mut g, p, "Divination", Zone::Library);
        let library = g.state.players[p.index()].library.len();
        checked(&mut g, activate(jessica));
        pay(&mut g);
        settle(&mut g);
        assert!(g.object_view(jessica).tapped);
        assert_eq!(g.counter_count(jessica, CounterKind::Stun), 0);
        assert_eq!(g.state.players[p.index()].library.len(), library - 2);
        assert_eq!(g.object_view(divination).zone, Zone::Exile);

        // An actual opposing cast unlocks Mar-Vell before it resolves, allowing
        // Jessica's exiled sorcery to be cast in response with ordinary costs.
        checked(&mut g, Action::Pass);
        let shock = named(&mut g, p.other(), "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Player(p)]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::AnnounceExile { card: divination });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(divination).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].life, 18);

        // The paid Car blinks the powered-up Stature, then the newly typed Skrull
        // crews it and is untapped by Captain's independent first-tap packet.
        let old_stature = g.object_ref(stature);
        cast(&mut g, car, None, &[]);
        until(&mut g, |g| {
            g.state.announcing.as_ref().is_some_and(|a| {
                a.item.card == car && a.item.kind == (StackKind::TriggeredAbility { ability: 0 })
            })
        });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(stature),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(stature).zone, Zone::Exile);
        checked(&mut g, activate(car));
        let object = g.object_ref(skrull);
        checked(&mut g, Action::ChooseCrewer { object });
        checked(&mut g, Action::FinishCrew);
        settle(&mut g);
        assert!(g.is_creature(car));
        assert!(!g.object_view(skrull).tapped);
        assert!(g.first_tap(object));
        until(&mut g, |g| g.state.step == Step::End);
        settle(&mut g);
        assert_eq!(g.object_view(stature).zone, Zone::Battlefield);
        assert_ne!(g.object_ref(stature), old_stature);
        assert_eq!(g.counter_count(stature, CounterKind::PlusOne), 0);
        assert_eq!(g.stats(stature).power, 1);
        assert!(!g.can_block_pair(blocker, stature));
    }
}
