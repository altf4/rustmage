//! Seven whole candidates combine paid costs, declaration rules, LKI and exact blink.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..180 {
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
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("combined stalled {:?}", g.state.pending);
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
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn equip(g: &mut Game, kit: CardId, target: CardId) {
    checked(g, activate(kit));
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(g);
    settle(g);
}
fn put(g: &mut Game, card: CardId, ability: u16) {
    g.prepare_triggers();
    if let Some(id) = g
        .state
        .ordering_triggers
        .iter()
        .find(|a| a.card == card && a.kind == (StackKind::TriggeredAbility { ability }))
        .map(|a| a.id)
    {
        checked(g, Action::PutTrigger { id });
        pay(g);
    } else {
        assert!(
            g.state
                .stack
                .iter()
                .chain(g.state.announcing.iter().map(|a| &a.item))
                .any(|a| a.card == card && a.kind == (StackKind::TriggeredAbility { ability })),
            "packet not staged {card} {ability}"
        );
    }
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let before = g.snapshot();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|v| *v == a).unwrap();
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
#[test]
fn msh_eighth_combined_paid_costs_alone_attacks_lki_merfolk_and_tapped_equipment_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let thor = paid(&mut g, p, "The Mighty Thor, Jane Foster");
        let wings = paid(&mut g, p, "Captain America, Wings of Freedom");
        let whip = paid(&mut g, p, "Whiplash, Vengeful Engineer");
        assert!(g.object_view(whip).tapped);
        let crowd = paid(&mut g, p, "Crowd of True Believers");
        let attuma = paid(&mut g, p, "Attuma, Atlantean Warlord");
        let kit = paid(&mut g, p, "S.H.I.E.L.D. Spy Kit");
        let skrull = paid(&mut g, p, "Undercover Skrull");
        let discard = named(&mut g, p, "Bear Cub", Zone::Hand);
        let titania = named(&mut g, p, "Titania, Rugged Rumbler", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 2];
        checked(&mut g, Action::AnnounceCast { card: titania });
        checked(&mut g, Action::ChooseSpellCost { cost: 0 });
        let object = g.object_ref(discard);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
        g.state.players[p.index()].mana = [30; 6];
        equip(&mut g, kit, wings);
        assert_eq!(g.stats(wings).toughness, 2);
        assert_eq!(g.stats(skrull).power, 1);
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.step == Step::Main1 && g.state.pending == Pending::Priority
        });
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: wings });
        checked(&mut g, Action::FinishAttackers);
        put(&mut g, wings, 1);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectLibrary { .. })
        });
        assert!(!g.object_view(wings).tapped);
        assert!(g.observe(p).library_choice.is_some());
        assert!(g.observe(p.other()).library_choice.is_none());
        let top = *g.state.players[p.index()].library.last().unwrap();
        g = orphan(
            g,
            Action::ArrangeLibrary {
                card: top,
                destination: LibraryDestination::Bottom,
            },
        );
        assert!(g.state.stack.iter().any(|a| a.card == wings));
        checked(&mut g, activate(crowd));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(wings),
            },
        );
        pay(&mut g);
        until(&mut g, |g| {
            g.state.players[p.index()].life == 21 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.stats(wings).power, 5);
        g.state.players[p.index()].mana = [30; 6];
        let murder = named(&mut g, p, "Murder", Zone::Hand);
        cast(&mut g, murder, None, &[Target::Card(wings)]);
        until(&mut g, |g| {
            g.object_view(wings).zone == Zone::Graveyard && g.state.pending == Pending::Priority
        });
        assert_eq!(g.stats(skrull).power, 4);
        assert!(g.has_subtype(skrull, "Hero") && g.has_subtype(skrull, "Merfolk"));
        settle(&mut g);
        assert_eq!(g.stats(thor).power, 5);
        assert_eq!(g.stats(skrull).power, 6);
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        let ward_discard = named(&mut g, p.other(), "Forest", Zone::Hand);
        checked(&mut g, Action::Pass);
        g.state.players[p.other().index()].mana = [0, 0, 0, 1, 0, 0];
        let shock = named(&mut g, p.other(), "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Card(titania)]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionMana { .. })
        });
        assert_eq!(g.actor(), Some(p.other()));
        assert!(g.observe(p).effect_choice.is_none());
        assert!(g.observe(p.other()).effect_choice.is_some());
        g = orphan(g, Action::Discard { card: ward_discard });
        settle(&mut g);
        assert_eq!(g.object_view(ward_discard).zone, Zone::Graveyard);
        assert_eq!(g.object_view(titania).damage, 2);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        equip(&mut g, kit, whip);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.step == Step::Main1 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.stats(thor).power, 3);
        assert_eq!(g.stats(skrull).power, 4);
        let library = g.state.players[p.index()].library.len();
        let old_kit = g.object_ref(kit);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        for card in [whip, attuma, thor, skrull] {
            checked(&mut g, Action::Attack { card });
        }
        checked(&mut g, Action::FinishAttackers);
        put(&mut g, whip, 0);
        put(&mut g, attuma, 0);
        until(&mut g, |g| {
            g.state
                .announcing
                .as_ref()
                .is_some_and(|a| a.item.card == thor)
        });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(kit),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_ne!(g.object_ref(kit), old_kit);
        assert!(g.object_view(kit).tapped);
        assert_eq!(g.state.objects[kit as usize].attached_to, None);
        assert_eq!(g.state.players[p.index()].library.len(), library - 2);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        assert_eq!(g.stats(whip).power, 2);
        let before = g.snapshot();
        assert_eq!(
            g.apply(g.actor().unwrap(), activate(crowd)),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), before);
    }
}
