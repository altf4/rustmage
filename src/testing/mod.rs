//! Card coverage: shared rules once, ordinary printed outcomes per card feature.
//! Fixtures arrange state; all behavior under test goes through checked actions.
use super::*;

mod card_cases;
mod cards;
mod core;
mod mechanics;

fn game() -> Game {
    let mut g = Game::new(Config {
        decks: [vec![Card::Forest; 7], vec![Card::Forest; 7]],
        seed: 1,
        starting_player: Player::P0,
        deck_rules: DeckRules::Freeform,
    })
    .unwrap();
    g.state.objects.clear();
    for player in &mut g.state.players {
        player.library.clear();
        player.cards_drawn_this_turn = 0;
        player.kept = true;
        player.mana = [8; 6];
    }
    g.state.turn = 3;
    g.state.step = Step::Main1;
    g.state.pending = Pending::Priority;
    for player in [Player::P0, Player::P1] {
        for _ in 0..8 {
            put(&mut g, player, "Forest", Zone::Library);
        }
    }
    g
}

fn put(g: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap_or_else(|| panic!("unknown fixture: {name}"));
    assert!(card.is_supported(), "unsupported fixture: {name}");
    let id = g.state.objects.len() as CardId;
    g.state.objects.push(Object {
        card,
        copy_kind: CopyKind::None,
        owner: player,
        controller: player,
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
        Zone::Library => g.state.players[player.index()].library.push(id),
        Zone::Battlefield => g.register_battlefield_sources(id),
        Zone::Graveyard => g.register_graveyard_sources(id),
        _ => (),
    }
    id
}

fn act(g: &mut Game, action: Action) {
    g.apply(g.actor().expect("nonterminal fixture"), action.clone())
        .unwrap_or_else(|e| panic!("{action:?}: {e:?}; pending {:?}", g.state.pending));
}

fn choose(g: &mut Game, predicate: impl Fn(&Action) -> bool) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(predicate)
        .unwrap_or_else(|| panic!("missing choice in {:?}", g.state.pending));
    act(g, action);
}

fn pay(g: &mut Game) {
    choose(g, |a| matches!(a, Action::PayCosts { .. }));
}

fn cast(g: &mut Game, card: CardId, targets: &[Target]) {
    act(g, Action::AnnounceCast { card });
    if matches!(g.state.pending, Pending::SpellCost { .. }) {
        act(g, Action::ChooseSpellCost { cost: 0 });
    }
    for &target in targets {
        act(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}

fn settled(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.announcing.is_none()
}

fn resolve(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..32 {
        if settled(g) || g.outcome().is_some() {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("resolution exceeded 32 actions: {:?}", g.state.pending);
}

fn attack(g: &mut Game, card: CardId) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(g, Action::Attack { card });
    act(g, Action::FinishAttackers);
}

fn stats(g: &Game, id: CardId) -> (i128, i128) {
    let stats = g.stats(id);
    (stats.power, stats.toughness)
}
