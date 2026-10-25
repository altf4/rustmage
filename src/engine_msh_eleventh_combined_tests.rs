//! Paid scenario fixtures stress seven new cards plus the existing Vision.
//! This is not a Standard-legal deck/full-game coverage claim.
use super::assurance::big_helpers::{checked, named, setup};
use super::*;
const AGENT: &str = "U.S.Agent, John Walker";
const LAD: &str = "Iron Lad, Diverging Destiny";
const DEVIL: &str = "Daredevil, Man Without Fear";
const IRON: &str = "Ironheart, Clever Champion";
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    let bytes = g.state_bytes();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
    assert_eq!(g.state_bytes(), bytes);
}
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| {
        let mut observation = g.observe(p);
        // Restoring deliberately advances the branch revision.
        observation.revision = 0;
        serde_json::to_value(observation).unwrap()
    })
}
fn orphan(g: Game, action: Action) -> Game {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let before = g.snapshot();
    let before_views = views(&g);
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, action.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    let after = raw.snapshot();
    assert_eq!(indexed.snapshot(), after);
    assert_eq!(views(&raw), views(&indexed));
    raw.restore(&before).unwrap();
    assert_eq!(views(&raw), before_views);
    assert_eq!(
        raw.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    raw.apply(actor, action).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
fn finish_packet(g: &mut Game, packet: u64) {
    for _ in 0..20 {
        if g.state
            .announcing
            .as_ref()
            .is_none_or(|a| a.item.id != packet)
        {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::ChooseSpellCost { .. }
                )
            })
            .unwrap_or_else(|| panic!("packet {packet} stalled at {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("packet {packet} did not finish");
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, a);
}
fn select(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    checked(g, Action::ChooseImproviser { object });
}
// Never guess modal choices or choose optional/multi targets. Only later Daredevil
// attacks are declined by default. Every nontrivial packet is stopped explicitly.
fn drive(g: &mut Game, stop: impl Fn(&Game) -> bool) -> usize {
    let mut declined = 0;
    g.prepare_triggers();
    for _ in 0..600 {
        if stop(g) {
            return declined;
        }
        assert!(g.outcome().is_none(), "unexpected terminal state");
        let action = if matches!(g.state.pending, Pending::EffectOptional { .. }) {
            declined += 1;
            Action::ChooseOptional { accept: false }
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::Pass
                            | Action::PutTrigger { .. }
                            | Action::FinishAttackers
                            | Action::FinishBlockers
                    ) || matches!(a, Action::FinishTargets)
                        && g.state.announcing.as_ref().is_some_and(|a| a.spec.max == 0)
                })
                .unwrap_or_else(|| {
                    panic!(
                        "unhandled combined decision {:?} at {:?}",
                        g.state.pending, g.state.step
                    )
                })
        };
        checked(g, action);
    }
    panic!("combined progression stalled");
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
}
fn settle(g: &mut Game) {
    drive(g, quiet);
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn paid(g: &mut Game, p: Player, name: &str, mana: Mana, cost: Option<u16>) -> CardId {
    priority(g, p);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    let id = named(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    checked(g, Action::AnnounceCast { card: id });
    let packet = g.state.announcing.as_ref().unwrap().item.id;
    if let Some(cost) = cost {
        checked(g, Action::ChooseSpellCost { cost });
    }
    finish_packet(g, packet);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    settle(g);
    id
}
fn next_own_main(g: &mut Game, p: Player) {
    let turn = g.state.turn;
    drive(g, |g| {
        g.state.turn > turn
            && g.state.active == p
            && g.state.step == Step::Main1
            && quiet(g)
            && g.actor() == Some(p)
    });
}
fn trigger(g: &Game, id: CardId, ability: u16) -> bool {
    g.state.announcing.as_ref().is_some_and(|a| {
        a.item.card == id && a.item.kind == StackKind::TriggeredAbility { ability }
    })
}
#[test]
fn msh_eleventh_combined_paid_tokens_improvise_top_exile_modes_life_saga_and_natural_win() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for player in &mut g.state.players {
            player.mana = [0; 6];
        }
        // The top three are bottom-to-top Reactor, Forest, Vision. Existing
        // filler remains underneath for the ordinary draws through Saga IV.
        let revealed_arc = named(&mut g, p, "Arc Reactor", Zone::Library);
        let drawn_forest = named(&mut g, p, "Forest", Zone::Library);
        let vision = named(&mut g, p, "The Vision", Zone::Library);
        let spy = named(&mut g, p.other(), "S.H.I.E.L.D. Spy Kit", Zone::Battlefield);
        let mountain = named(&mut g, p.other(), "Mountain", Zone::Battlefield);
        let secret = named(&mut g, p.other(), "Murder", Zone::Hand);
        let feast = paid(&mut g, p, "Heroic Feast", [0, 0, 0, 0, 1, 2], None);
        let food = (0..g.state.objects.len() as CardId)
            .find(|&id| {
                g.on_battlefield(id)
                    && g.object_view(id).card.identity() == "@token/283fd83a9aec83a51b736926"
            })
            .unwrap();
        let agent = paid(&mut g, p, AGENT, [1, 0, 0, 0, 0, 3], Some(1));
        let shield = (0..g.state.objects.len() as CardId)
            .find(|&id| {
                g.on_battlefield(id) && g.object_view(id).card == Card::MshSturdyShieldToken
            })
            .unwrap();
        assert_eq!(g.object_view(shield).attached_to, Some(g.object_ref(agent)));
        let lad = paid(&mut g, p, LAD, [0, 1, 0, 0, 0, 2], None);
        assert_eq!(
            g.observe(p).known_library_top[p.index()].unwrap().object,
            g.object_ref(vision)
        );
        assert!(g.observe(p.other()).known_library_top[p.index()].is_none());
        let mut hidden = g.clone();
        hidden.state.objects[vision as usize].card = Card::from_name(IRON).unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            serde_json::to_value(hidden.observe(p.other())).unwrap()
        );
        hidden = g.clone();
        hidden.state.objects[secret as usize].card = Card::from_name("Shock").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p)).unwrap(),
            serde_json::to_value(hidden.observe(p)).unwrap()
        );
        let arc = named(&mut g, p, "Arc Reactor", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        checked(&mut g, Action::AnnounceCast { card: arc });
        checked(&mut g, Action::BeginImprovise);
        select(&mut g, food);
        let object = g.object_ref(shield);
        g = orphan(g, Action::ChooseImproviser { object });
        select(&mut g, lad);
        assert!(
            [food, shield, lad]
                .iter()
                .all(|&id| !g.object_view(id).tapped)
        );
        let duplicate = g.object_ref(food);
        reject(&mut g, Action::ChooseImproviser { object: duplicate });
        pay(&mut g);
        settle(&mut g);
        assert!(
            [food, shield, lad, arc]
                .iter()
                .all(|&id| g.object_view(id).tapped)
        );
        assert!(!g.object_view(agent).tapped);
        let dare = paid(&mut g, p, DEVIL, [1, 0, 0, 1, 0, 2], None);
        drive(&mut g, |g| {
            g.state.pending == Pending::Attackers && g.state.active == p
        });
        checked(&mut g, Action::Attack { card: dare });
        checked(&mut g, Action::FinishAttackers);
        drive(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: true });
        assert_eq!(g.object_view(vision).zone, Zone::Exile);
        assert_eq!((g.stats(dare).power, g.stats(dare).toughness), (5, 5));
        drive(&mut g, |g| g.state.step == Step::Main2 && quiet(g));
        assert_eq!(g.state.players[p.other().index()].life, 15);
        assert!(!g.object_view(dare).tapped);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        checked(&mut g, Action::AnnounceExile { card: vision });
        let packet = g.state.announcing.as_ref().unwrap().item.id;
        finish_packet(&mut g, packet);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().origin,
            Zone::Exile
        );
        settle(&mut g);
        assert_eq!(g.object_view(vision).zone, Zone::Battlefield);
        next_own_main(&mut g, p);
        assert_eq!(g.object_view(drawn_forest).zone, Zone::Hand);
        assert_eq!((g.stats(dare).power, g.stats(dare).toughness), (3, 4));
        assert_eq!(
            g.observe(p).known_library_top[p.index()].unwrap().object,
            g.object_ref(revealed_arc)
        );
        assert!(
            [arc, food, shield, lad, vision]
                .iter()
                .all(|&id| !g.object_view(id).tapped)
        );
        let iron = named(&mut g, p, IRON, Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        checked(&mut g, Action::AnnounceCast { card: iron });
        checked(
            &mut g,
            Action::ActivateMana {
                card: arc,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 0, 3],
            },
        );
        checked(&mut g, Action::BeginImprovise);
        select(&mut g, shield);
        pay(&mut g);
        settle(&mut g);
        assert!(g.object_view(iron).summoning_sick);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(
            [food, lad, vision]
                .iter()
                .all(|&id| !g.object_view(id).tapped)
        );
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: lad,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let packet = g.state.announcing.as_ref().unwrap().item.id;
        finish_packet(&mut g, packet);
        settle(&mut g);
        assert_eq!(g.object_view(revealed_arc).zone, Zone::Hand);
        assert!(
            g.observe(p.other())
                .known_hand
                .contains(&g.object_ref(revealed_arc))
        );
        assert_eq!(
            g.observe(p).last_library_reveal,
            g.observe(p.other()).last_library_reveal
        );
        assert_eq!(
            g.observe(p).last_library_reveal.unwrap().drawn,
            Some(g.object_ref(revealed_arc))
        );
        let saga = named(&mut g, p, "The Coming of Galactus", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 1, 0];
        checked(&mut g, Action::AnnounceCast { card: saga });
        checked(&mut g, Action::BeginImprovise);
        select(&mut g, vision);
        select(&mut g, iron);
        pay(&mut g);
        drive(&mut g, |g| {
            trigger(g, vision, 0) && matches!(g.state.pending, Pending::Modes { .. })
        });
        let mode_packet = g.state.announcing.as_ref().unwrap().item.id;
        checked(&mut g, Action::ChooseMode { mode: 1 });
        finish_packet(&mut g, mode_packet);
        drive(&mut g, |g| trigger(g, saga, 0));
        let chapter = g.state.announcing.as_ref().unwrap().item.id;
        assert!(g.keywords(vision).contains(Keywords::INDESTRUCTIBLE));
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(mountain),
            },
        );
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Card(spy),
            },
        );
        finish_packet(&mut g, chapter);
        settle(&mut g);
        assert_eq!(g.object_view(spy).zone, Zone::Graveyard);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: food,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let packet = g.state.announcing.as_ref().unwrap().item.id;
        reject(&mut g, Action::BeginImprovise);
        finish_packet(&mut g, packet);
        drive(&mut g, |g| trigger(g, feast, 1));
        let feast_packet = g.state.announcing.as_ref().unwrap().item.id;
        assert_eq!(g.state.announcing.as_ref().unwrap().spec.max, 3);
        assert_eq!(g.state.players[p.index()].life, 23);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(mountain),
            },
        );
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Card(agent),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(agent),
            },
        );
        for target in [dare, iron] {
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
        }
        finish_packet(&mut g, feast_packet);
        settle(&mut g);
        assert_eq!((g.stats(agent).power, g.stats(agent).toughness), (5, 5));
        assert_eq!((g.stats(dare).power, g.stats(dare).toughness), (4, 5));
        assert_eq!((g.stats(iron).power, g.stats(iron).toughness), (4, 5));
        for id in [agent, dare, iron] {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        }
        assert!(!g.on_battlefield(food));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        for n in 2..=4 {
            next_own_main(&mut g, p);
            assert_eq!(
                g.state.players[p.other().index()].life,
                15 - 2 * i128::from(n.min(3) - 1)
            );
            assert_eq!(g.on_battlefield(saga), n < 4);
        }
        let gal = (0..g.state.objects.len() as CardId)
            .find(|&id| g.on_battlefield(id) && g.object_view(id).card == Card::MshGalactusToken)
            .unwrap();
        assert_eq!(g.object_view(saga).zone, Zone::Graveyard);
        assert_eq!((g.stats(gal).power, g.stats(gal).toughness), (16, 16));
        assert!(g.object_view(gal).summoning_sick);
        next_own_main(&mut g, p);
        assert!(!g.object_view(gal).summoning_sick);
        let library_before = g.state.players[p.index()].library.len();
        drive(&mut g, |g| {
            g.state.pending == Pending::Attackers && g.state.active == p
        });
        checked(&mut g, Action::Attack { card: gal });
        checked(&mut g, Action::FinishAttackers);
        let mut declined = drive(&mut g, |g| trigger(g, gal, 0));
        let land_packet = g.state.announcing.as_ref().unwrap().item.id;
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(mountain),
            },
        );
        finish_packet(&mut g, land_packet);
        declined += drive(&mut g, |g| g.outcome().is_some());
        assert_eq!(declined, 1);
        assert_eq!(g.object_view(mountain).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].library.len(), library_before);
        assert_eq!(g.outcome(), Some(Outcome::Winner(p)));
        assert_eq!(g.state.players[p.other().index()].life, -5);
    }
}

#[test]
fn msh_eleventh_combined_paid_plot_changed_top_defers_until_owned_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for player in &mut g.state.players {
            player.mana = [0; 6];
        }
        paid(&mut g, p, LAD, [0, 1, 0, 0, 0, 2], None);
        let below = named(&mut g, p, "Shock", Zone::Library);
        let old_top = named(&mut g, p, "Forest", Zone::Library);
        let plotted = named(&mut g, p, "Slickshot Show-Off", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
        let visible = g.object_ref(old_top);
        assert_eq!(
            g.observe(p).known_library_top[p.index()].unwrap().object,
            visible
        );
        checked(&mut g, Action::AnnouncePlot { card: plotted });
        assert_eq!(g.state.pending, Pending::PlotPayment { player: p });
        assert_eq!(
            g.observe(p).known_library_top[p.index()].unwrap().object,
            visible
        );
        assert_eq!(
            g.state
                .library_top
                .as_ref()
                .unwrap()
                .operation
                .as_ref()
                .unwrap()
                .starting_top[p.index()],
            Some(visible)
        );
        // Deliberate phase-boundary probe using the same production movement as
        // the existing activation fixture, not a naturally fundable top change.
        g.move_object(old_top, Zone::Exile);
        assert!(g.observe(p).known_library_top[p.index()].is_none());
        assert!(g.state.library_top.as_ref().unwrap().operation.is_some());
        assert!(g.plot_payment().is_some());
        let mut hidden = g.clone();
        hidden.state.objects[below as usize].card = Card::from_name("Giant Growth").unwrap();
        assert_eq!(views(&g), views(&hidden));
        assert_eq!(g.legal_actions(), hidden.legal_actions());
        reject(&mut g, Action::PayPlot { payment: [0; 6] });
        let paid_object = g.object_ref(plotted);
        g = orphan(
            g,
            Action::PayPlot {
                payment: [0, 0, 0, 1, 0, 1],
            },
        );
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.plot_payment().is_none());
        assert!(
            g.state
                .library_top
                .as_ref()
                .is_none_or(|s| s.operation.is_none())
        );
        assert_eq!(
            g.observe(p).known_library_top[p.index()].unwrap().object,
            g.object_ref(below)
        );
        assert!(g.observe(p.other()).known_library_top[p.index()].is_none());
        assert_eq!(g.object_view(plotted).zone, Zone::Exile);
        let actual = g.object_ref(plotted);
        assert_ne!(actual, paid_object);
        assert_eq!(g.state.plot.as_ref().unwrap().cards[0].object, actual);
        assert_eq!(g.state.plot.as_ref().unwrap().cards[0].owner, p);
    }
}
