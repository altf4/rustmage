//! Shared white-box scenario fixtures for integration targets. They are compiled
//! only by tests; ordinary consumers do not enable the core fixture feature.
#![allow(dead_code)]
use rustmage_core::cards::*;
use rustmage_core::engine::*;

pub(crate) fn scenario(p: Player) -> Game {
    let mut g = Game::new(Config {
        decks: [vec![Card::Forest; 7], vec![Card::Forest; 7]],
        seed: 20261023,
        starting_player: p,
        deck_rules: DeckRules::Freeform,
    })
    .unwrap();
    // Reset only the opening deck/hand. Fresh games have no registered sources.
    g.state.objects.clear();
    for player in &mut g.state.players {
        player.library.clear();
        player.cards_drawn_this_turn = 0;
        player.kept = true;
        player.mana = [8; 6];
    }
    g.state.pending = Pending::Priority;
    g.state.step = Step::Main1;
    g.state.turn = 3;
    g.state.active = p;
    g.state.priority = p;
    for owner in [Player::P0, Player::P1] {
        for _ in 0..8 {
            put(&mut g, owner, "Forest", Zone::Library);
        }
    }
    g
}

pub(crate) fn put(g: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap_or_else(|| panic!("unknown fixture {name}"));
    assert!(
        card.is_supported()
            || name == "Ghost Vacuum"
            || name == "Pyrrhic Strike"
            || name == "Spring-Loaded Sawblades"
            || name == "Braided Net"
            || name == "@test/manifest-observer",
        "unsupported fixture {name}"
    );
    let id = g.state.objects.len() as CardId;
    g.state.objects.push(Object {
        copy_kind: CopyKind::None,
        card,
        owner,
        controller: owner,
        zone,
        generation: 0,
        tapped: false,
        summoning_sick: false,
        flashback: false,
        cast_for_warp: false,
        damage: 0,
        damaged_this_turn: false,
        dealt_damage_this_turn: false,
        dealt_combat_damage_this_incarnation: false,
        deathtouch_since_sba: false,
        growth: Stats::default(),
        gained_keywords: Keywords::NONE,
        counters: InitialCounters::default(),
        used_abilities: 0,
        loyalty_activated_this_turn: false,
        class_level: 1,
        attached_to: None,
    });
    match zone {
        Zone::Library => g.state.players[owner.index()].library.push(id),
        Zone::Battlefield => g.register_battlefield_sources(id),
        Zone::Graveyard => g.register_graveyard_sources(id),
        _ => (),
    }
    id
}

pub(crate) fn act(g: &mut Game, action: Action) {
    g.apply(g.actor().expect("nonterminal scenario"), action.clone())
        .unwrap_or_else(|error| panic!("{action:?}: {error:?}; pending {:?}", g.state.pending));
}

pub(crate) fn choose(g: &mut Game, predicate: impl Fn(&Action) -> bool) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(predicate)
        .unwrap_or_else(|| panic!("missing choice in {:?}", g.state.pending));
    act(g, a);
}

pub(crate) fn pay(g: &mut Game) {
    choose(g, |a| matches!(a, Action::PayCosts { .. }));
}

pub(crate) fn cast(g: &mut Game, id: CardId, target: Option<Target>) {
    if let Some(action) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Cast{card,target:t,..}if *card==id&&*t==target))
    {
        act(g, action);
    } else {
        announce(g, id, &target.into_iter().collect::<Vec<_>>());
    }
}

pub(crate) fn announce(g: &mut Game, id: CardId, targets: &[Target]) {
    act(g, Action::AnnounceCast { card: id });
    for &target in targets {
        act(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}

pub(crate) fn settle(g: &mut Game) {
    for _ in 0..48 {
        if g.outcome().is_some()
            || (g.state.stack.is_empty()
                && g.state.resolving.is_none()
                && g.state.pending == Pending::Priority)
        {
            return;
        }
        assert_eq!(
            g.state.pending,
            Pending::Priority,
            "scenario needs an explicit choice"
        );
        act(g, Action::Pass);
    }
    panic!("48-action resolution budget exhausted");
}

pub(crate) fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}

pub(crate) fn stats(g: &Game, id: CardId) -> (i128, i128) {
    let s = g.stats(id);
    (s.power, s.toughness)
}
