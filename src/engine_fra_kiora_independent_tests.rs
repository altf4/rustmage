use super::*;

fn loyalty_walker(g: &mut Game, player: Player) -> CardId {
    g.create_tokens(player, Card::JacePlaneswalkerToken, 1, false);
    let id = g
        .state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, object)| {
            (object.zone == Zone::Battlefield
                && object.controller == player
                && object.is_token()
                && g.has_subtype(id as CardId, "Jace"))
            .then_some(id as CardId)
        })
        .unwrap();
    g.state.objects[id as usize].counters.loyalty = 9;
    id
}

fn activate_jace(g: &mut Game, walker: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: walker,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
}

fn select_kiora_target(g: &mut Game, target: CardId) {
    for _ in 0..48 {
        if g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(target),
        }) {
            break;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}

#[test]
fn countered_loyalty_still_qualifies_attack_on_planeswalker() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let kiora = put(&mut g, player, "Kiora of Salt and Sand", Zone::Battlefield);
        let attacker = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let own_walker = loyalty_walker(&mut g, player);
        let defender = loyalty_walker(&mut g, player.other());
        activate_jace(&mut g, own_walker);
        let ability = g
            .state
            .stack
            .iter()
            .find(|item| item.source.is_some_and(|source| source.id == own_walker))
            .unwrap()
            .id;
        assert!(g.counter_stack_item(ability, false));
        assert!(
            g.observe(player.other()).players[player.index()].loyalty_ability_activated_this_turn
        );
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        let attack = Action::AttackPlaneswalker {
            card: attacker,
            defender: g.object_ref(defender),
        };
        assert!(g.legal_actions().contains(&attack));
        act(&mut g, attack);
        act(&mut g, Action::FinishAttackers);
        select_kiora_target(&mut g, attacker);
        assert!(g.state.stack.iter().any(|item| item.source.is_some_and(|source| source.id == kiora)));
        settle(&mut g);
        assert!(!g.object_view(attacker).tapped);
        assert!(g.keywords(attacker).contains(Keywords::CANT_BE_BLOCKED));
    }
}

#[test]
fn opposing_loyalty_activation_does_not_qualify_kiora() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player.other());
        let kiora = put(&mut g, player, "Kiora of Salt and Sand", Zone::Battlefield);
        let attacker = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let opposing_walker = loyalty_walker(&mut g, player.other());
        activate_jace(&mut g, opposing_walker);
        settle(&mut g);
        assert!(
            g.observe(player).players[player.other().index()].loyalty_ability_activated_this_turn
        );
        assert!(
            !g.observe(player.other()).players[player.index()].loyalty_ability_activated_this_turn
        );
        g.state.active = player;
        g.state.priority = player;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        assert!(
            !g.state
                .pending_triggers
                .iter()
                .chain(g.state.stack.iter())
                .any(|item| item.source.is_some_and(|source| source.id == kiora))
        );
    }
}

#[test]
fn kiora_target_leaving_and_returning_does_not_receive_effect() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        put(&mut g, player, "Kiora of Salt and Sand", Zone::Battlefield);
        let attacker = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let walker = loyalty_walker(&mut g, player);
        activate_jace(&mut g, walker);
        settle(&mut g);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        select_kiora_target(&mut g, attacker);
        let old = g.object_ref(attacker);
        g.move_object(attacker, Zone::Exile);
        g.move_object(attacker, Zone::Battlefield);
        assert_ne!(g.object_ref(attacker), old);
        let checkpoint = g.snapshot();
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&checkpoint).unwrap();
            }
            settle(&mut g);
            assert!(!g.keywords(attacker).contains(Keywords::CANT_BE_BLOCKED));
        }
    }
}
