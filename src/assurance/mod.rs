//! Bounded rules assurance. Card outcomes use ordinary checked actions; expensive
//! ownership/rollback contracts are checked once at representative decision roots.
use super::*;
pub(super) mod big_helpers;
mod cards;
mod contracts;
mod interactions;
mod regressions;

fn scenario(p: Player) -> Game {
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

fn put(g: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap_or_else(|| panic!("unknown fixture {name}"));
    assert!(card.is_supported(), "unsupported fixture {name}");
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
        damage: 0,
        damaged_this_turn: false,
        dealt_damage_this_turn: false,
        deathtouch_since_sba: false,
        growth: Stats::default(),
        gained_keywords: Keywords::NONE,
        counters: InitialCounters::default(),
        used_abilities: 0,
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
fn act(g: &mut Game, action: Action) {
    g.apply(g.actor().expect("nonterminal scenario"), action)
        .unwrap();
}
fn choose(g: &mut Game, predicate: impl Fn(&Action) -> bool) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(predicate)
        .unwrap_or_else(|| panic!("missing choice in {:?}", g.state.pending));
    act(g, a);
}
fn pay(g: &mut Game) {
    choose(g, |a| matches!(a, Action::PayCosts { .. }));
}
fn cast(g: &mut Game, id: CardId, target: Option<Target>) {
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

fn announce(g: &mut Game, id: CardId, targets: &[Target]) {
    act(g, Action::AnnounceCast { card: id });
    for &target in targets {
        act(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}
fn settle(g: &mut Game) {
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
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn stats(g: &Game, id: CardId) -> (i128, i128) {
    let s = g.stats(id);
    (s.power, s.toughness)
}

#[path = "../engine_sos_black_red_tests.rs"]
mod sos_black_red;
#[path = "../engine_sos_multicolor_tests.rs"]
mod sos_multicolor;

mod sos_white_blue;

mod sos_milled_play;
mod sos_multicolor_next;

mod sos_discard_followon;
mod sos_exile_search;
mod sos_hand_attack;
mod sos_moment;
mod sos_multicolor_more;
mod sos_nita;
mod sos_review_regressions;
mod sos_white_blue_final;

mod sos_applied;
mod sos_killian;
mod sos_white_blue_dragons;

mod sos_br_audit;
mod sos_life_x;
mod sos_original_multicolor_audit;
mod sos_storm;
mod sos_wu_audit;
mod sos_zaffai;

mod sos_cast_rollback;
mod sos_complete_games;

mod sos_evidence_funding;

mod sos_mana_rollback;

mod msh_a;
mod msh_b;
mod msh_c;
mod msh_contracts;
mod msh_d;
