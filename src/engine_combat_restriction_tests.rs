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
                .or_else(|| {
                    actions.iter().find(|a| {
                        matches!(a, Action::ChooseSpellCost { .. } | Action::FinishTargets)
                    })
                })
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

fn settle(g: &mut Game) {
    settle_on(g, None);
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
fn cast(g: &mut Game, id: CardId, mode: Option<u16>, targets: &[Target]) {
    checked(g, Action::AnnounceCast { card: id });
    if let Some(mode) = mode {
        reject(g, Action::ChooseMode { mode: 3 });
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
    for _ in 0..16 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseSpellCost { .. } | Action::FinishTargets))
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

// These isolated declaration fixtures reuse the board produced by real paid
// resolution. They clear summoning sickness and set a declaration phase so each
// restriction can be tested independently of unrelated timing/tapping rules.
fn declarations(g: &Game, id: CardId, p: Player, attack: bool, block: bool) {
    let mut a = g.clone();
    a.state.active = p;
    a.state.priority = p;
    a.state.step = Step::Attackers;
    a.state.pending = Pending::Attackers;
    a.state.selected_attackers.clear();
    a.state.attackers.clear();
    a.state.objects[id as usize].summoning_sick = false;
    a.state.objects[id as usize].tapped = false;
    let walker = named(
        &mut a,
        p.other(),
        "Ajani, Caller of the Pride",
        Zone::Battlefield,
    );
    a.state.objects[walker as usize].counters.loyalty =
        a.state.objects[walker as usize].card.starting_loyalty();
    for action in [
        Action::Attack { card: id },
        Action::AttackPlaneswalker {
            card: id,
            defender: a.object_ref(walker),
        },
    ] {
        assert_eq!(
            a.legal_actions().contains(&action),
            attack,
            "attack {}",
            a.state.objects[id as usize].card.info().name
        );
        let mut branch = a.clone();
        if attack {
            checked(&mut branch, action)
        } else {
            reject(&mut branch, action)
        }
    }
    let mut b = g.clone();
    b.state.active = p.other();
    b.state.priority = p;
    b.state.step = Step::Blockers;
    b.state.pending = Pending::Blockers;
    b.state.blocks.clear();
    b.state.objects[id as usize].tapped = false;
    let foe = named(&mut b, p.other(), "Grizzly Bears", Zone::Battlefield);
    b.state.attackers = vec![Attacker {
        card: foe,
        defender: Some(CombatDefender::Player(p)),
        blocked: false,
    }];
    let action = Action::Block {
        blocker: id,
        attacker: foe,
    };
    assert_eq!(
        b.legal_actions().contains(&action),
        block,
        "block {}",
        b.state.objects[id as usize].card.info().name
    );
    if block {
        checked(&mut b, action)
    } else {
        reject(&mut b, action)
    }
}
#[test]
fn paid_pacifism_is_an_external_restriction_and_removal_restores_declarations() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let aura = named(&mut g, p, "Pacifism", Zone::Hand);
        let remove = named(&mut g, p, "Disenchant", Zone::Hand);
        let keywords = g.keywords(bear);
        cast(&mut g, aura, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(
            g.state.objects[aura as usize].attached_to,
            Some(g.object_ref(bear))
        );
        assert_eq!(g.keywords(bear), keywords);
        declarations(&g, bear, p.other(), false, false);
        cast(&mut g, remove, None, &[Target::Card(aura)]);
        settle(&mut g);
        assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
        declarations(&g, bear, p.other(), true, true);
    }
}
#[test]
fn mousers_other_artifact_condition_never_counts_itself_and_does_not_ban_blocks() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Mouser Mark III", Zone::Hand);
        let artifact = named(&mut g, p, "Mox Jasper", Zone::Hand);
        let remove = named(&mut g, p, "Disenchant", Zone::Hand);
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        declarations(&g, source, p, false, true);
        cast(&mut g, artifact, None, &[]);
        settle(&mut g);
        declarations(&g, source, p, true, true);
        cast(&mut g, remove, None, &[Target::Card(artifact)]);
        settle(&mut g);
        declarations(&g, source, p, false, true);
    }
}
#[test]
fn tiger_dillo_checks_other_continuous_power_and_never_its_own_four_power() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Tiger-Dillo", Zone::Hand);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let grow = named(&mut g, p, "Giant Growth", Zone::Hand);
        let shrink = named(&mut g, p, "Stab", Zone::Hand);
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        declarations(&g, source, p, false, false);
        cast(&mut g, grow, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.stats(bear).power, 5);
        declarations(&g, source, p, true, true);
        cast(&mut g, shrink, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.stats(bear).power, 3);
        declarations(&g, source, p, false, false);
    }
}
#[test]
fn technodrome_full_paid_sacrifice_draw_counter_and_temporary_power_threshold() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Technodrome", Zone::Hand);
        let food = named(&mut g, p, "Mox Jasper", Zone::Battlefield);
        let footwork = named(&mut g, p, "Fancy Footwork", Zone::Hand);
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        declarations(&g, source, p, false, false);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceActivate{card,..}if *card==source))
        );
        g.state.objects[source as usize].summoning_sick = false;
        let drawn = g.state.players[p.index()].cards_drawn_this_turn;
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::AnnounceActivate{card,ability:0,..}if *card==source))
            .unwrap();
        checked(&mut g, action);
        settle(&mut g);
        assert_eq!(g.object_view(food).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, drawn + 1);
        assert_eq!(g.state.objects[source as usize].counters.plus_one, 1);
        assert!(g.state.objects[source as usize].tapped);
        assert_eq!(g.stats(source).power, 4);
        declarations(&g, source, p, false, false);
        cast(&mut g, footwork, None, &[Target::Card(source)]);
        settle(&mut g);
        assert!(!g.state.objects[source as usize].tapped);
        assert_eq!(g.stats(source).power, 6);
        declarations(&g, source, p, true, true);
    }
}
#[test]
fn lion_turtle_complete_etb_mana_and_actual_lesson_spells_cross_exact_threshold() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "The Lion-Turtle", Zone::Hand);
        let first = named(&mut g, p, "Abandon Attachments", Zone::Hand);
        let second = named(&mut g, p, "Shared Roots", Zone::Hand);
        let third = named(&mut g, p, "Fancy Footwork", Zone::Hand);
        for _ in 0..3 {
            named(&mut g, p.other(), "Abandon Attachments", Zone::Graveyard);
        }
        let life = g.state.players[p.index()].life;
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 3);
        declarations(&g, source, p, false, false);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceMana{card,..}if *card==source))
        );
        cast(&mut g, first, None, &[]);
        settle(&mut g);
        cast(&mut g, second, None, &[]);
        settle(&mut g);
        declarations(&g, source, p, false, false);
        cast(&mut g, third, None, &[Target::Card(source)]);
        settle(&mut g);
        declarations(&g, source, p, true, true);
        g.state.objects[source as usize].summoning_sick = false;
        let mana = g.state.players[p.index()].mana;
        let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::AnnounceMana{card,production,..}if *card==source&&*production==[0,0,1,0,0,0])).unwrap();
        checked(&mut g, action);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana[2], mana[2] + 1);
        assert!(g.state.objects[source as usize].tapped);
    }
}
