use super::*;

// Exercise the gated authored definition without claiming public catalogue support.
fn charm(g: &mut Game, owner: Player) -> CardId {
    let id = put(g, owner, "Bear Cub", Zone::Hand);
    g.state.objects[id as usize].card = Card::from_name("Fatehold Charm").unwrap();
    id
}

fn cast_draw(g: &mut Game, owner: Player) {
    let spell = charm(g, owner);
    act(g, Action::AnnounceCast { card: spell });
    act(g, Action::ChooseMode { mode: 0 });
    pay(g);
}

fn jaces(g: &Game, player: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == player
                && o.is_token()
                && g.current_kinds(id as CardId) & 64 != 0
                && g.has_subtype(id as CardId, "Jace"))
            .then_some(id as CardId)
        })
        .collect()
}

#[test]
fn fatehold_empower_ignores_physical_and_opposing_jaces_then_reuses_own_token() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let physical = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[physical as usize].card = Card::from_name("Jace Reawakened").unwrap();
        g.state.objects[physical as usize].counters.loyalty = 4;
        g.create_tokens(p.other(), Card::JacePlaneswalkerToken, 1, false);
        let enemy = jaces(&g, p.other())[0];
        g.state.objects[enemy as usize].counters.loyalty = 4;
        let before = g.state.players[p.index()].library.len();
        cast_draw(&mut g, p);
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
        let own = jaces(&g, p);
        assert_eq!(own.len(), 1);
        assert_eq!(g.counter_count(own[0], CounterKind::Loyalty), 2);
        assert_eq!(g.counter_count(physical, CounterKind::Loyalty), 4);
        assert_eq!(g.counter_count(enemy, CounterKind::Loyalty), 4);
        cast_draw(&mut g, p);
        settle(&mut g);
        assert_eq!(jaces(&g, p), own);
        assert_eq!(g.counter_count(own[0], CounterKind::Loyalty), 4);
    }
}

#[test]
fn fatehold_empower_doubled_token_choice_is_nontargeted_and_transactional() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let before = g.state.players[p.index()].library.len();
        cast_draw(&mut g, p);
        for _ in 0..20 {
            if matches!(g.state.pending, Pending::EffectChoice { .. }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        let choices: Vec<_> = g
            .legal_actions()
            .into_iter()
            .filter_map(|action| match action {
                Action::ChooseJace { object } => Some(object),
                _ => None,
            })
            .collect();
        assert_eq!(choices.len(), 2);
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
        let illegal = g.object_ref(0);
        reject(&mut g, Action::ChooseJace { object: illegal });
        let snapshot = g.snapshot();
        super::contracts::branch_contract(g.clone());
        act(&mut g, Action::ChooseJace { object: choices[1] });
        settle(&mut g);
        assert_eq!(g.counter_count(choices[1].card, CounterKind::Loyalty), 4);
        assert_ne!(g.object_view(choices[0].card).zone, Zone::Battlefield);
        g.restore(&snapshot).unwrap();
        act(&mut g, Action::ChooseJace { object: choices[1] });
        settle(&mut g);
        assert_eq!(g.counter_count(choices[1].card, CounterKind::Loyalty), 4);
    }
}
