//! Focused printed outcomes; shared engine invariants live in core tests.
//! Complete leaf wiring over established divided-damage and captured-payment contracts.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..40 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("spark/letters decision budget: {:?}", g.state.pending);
}
fn optional(g: &mut Game) {
    until(g, |g| {
        g.legal_actions()
            .contains(&Action::ChooseOptional { accept: true })
    });
}
fn divide(g: &mut Game, targets: &[Target], shares: &[u16]) {
    until(g, |g| matches!(g.state.pending, Pending::Targets { .. }));
    for &target in targets {
        act(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    for &amount in shares {
        if matches!(g.state.pending, Pending::TargetAmount { .. }) {
            act(
                g,
                Action::AllocateTargetAmount {
                    amount: u32::from(amount),
                },
            );
        }
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    if g.legal_actions()
        .iter()
        .any(|a| matches!(a, Action::PayCosts { .. }))
    {
        pay(g);
    }
}
fn cast_gandalf(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Gandalf, Spark Starter", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 4];
    cast(g, id, None);
    id
}
fn letters(g: &mut Game, p: Player) -> CardId {
    put(g, p, "Uncover the Moon-Letters", Zone::Battlefield)
}

#[test]
fn spark_paid_body_reach_and_one_two_three_positive_damage_shares() {
    {
        let p = Player::P0;
        for shares in [vec![3], vec![2, 1], vec![1, 1, 1]] {
            let mut g = scenario(p);
            let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let giant = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let gandalf = cast_gandalf(&mut g, p);
            let targets = [
                Target::Player(p.other()),
                Target::Card(bear),
                Target::Card(giant),
            ];
            divide(&mut g, &targets[..shares.len()], &shares);
            assert_eq!(stats(&g, gandalf), (4, 3));
            assert!(g.keywords(gandalf).contains(Keywords::REACH));
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                20 - i128::from(shares[0])
            );
            assert_eq!(
                g.state.objects[bear as usize].damage,
                i128::from(*shares.get(1).unwrap_or(&0))
            );
            assert_eq!(
                g.state.objects[giant as usize].damage,
                i128::from(*shares.get(2).unwrap_or(&0))
            );
        }
    }
}

#[test]
fn letters_paid_enchantment_excludes_itself_creatures_opponents_and_allows_repeated_casts() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let enchantment = put(&mut g, p, "Uncover the Moon-Letters", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 3];
        cast(&mut g, enchantment, None);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        let bear = put(&mut g, p, "Bear Cub", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        cast(&mut g, bear, None);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        g.state.active = p.other();
        g.state.priority = p.other();
        let enemy = put(&mut g, p.other(), "Shock", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 0, 0, 1, 0, 0];
        cast(&mut g, enemy, Some(Target::Player(p)));
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        g.state.active = p;
        g.state.priority = p;
        for _ in 0..2 {
            let spell = put(&mut g, p, "Shock", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
            cast(&mut g, spell, Some(Target::Player(p.other())));
            optional(&mut g);
            act(&mut g, Action::ChooseOptional { accept: false });
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        }
        assert_eq!(g.state.players[p.other().index()].life, 16);
    }
}

#[test]
fn letters_free_cast_zero_can_be_accepted_or_declined_and_acceptance_still_discards_two() {
    {
        let p = Player::P0;
        for accept in [false, true] {
            let mut g = scenario(p);
            letters(&mut g, p);
            put(&mut g, p, "Omniscience", Zone::Battlefield);
            let a = put(&mut g, p, "Shock", Zone::Hand);
            let b = put(&mut g, p, "Forest", Zone::Hand);
            let spell = put(&mut g, p, "Divination", Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            act(
                &mut g,
                Action::AnnounceFree {
                    card: spell,
                    face: CastFace::Normal,
                    flash_surcharge: false,
                },
            );
            pay(&mut g);
            let packet = g
                .state
                .stack
                .iter()
                .find(|item| item.source.unwrap().id == spell && item.kind == StackKind::Spell)
                .unwrap();
            assert_eq!(packet.mana_value(), 3);
            assert!(g.counter_stack_item(packet.id, false));
            optional(&mut g);
            act(&mut g, Action::ChooseOptional { accept });
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            if accept {
                act(&mut g, Action::Discard { card: a });
                act(&mut g, Action::Discard { card: b });
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(a).zone,
                if accept { Zone::Graveyard } else { Zone::Hand }
            );
            assert_eq!(
                g.object_view(b).zone,
                if accept { Zone::Graveyard } else { Zone::Hand }
            );
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn letters_retains_chosen_x_payment_and_discards_only_available_cards() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        letters(&mut g, p);
        let spell = put(&mut g, p, "Finale of Revelation", Zone::Hand);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 2];
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseX { value: 2 });
        pay(&mut g);
        let packet = g
            .state
            .stack
            .iter()
            .find(|item| item.source.unwrap().id == spell && item.kind == StackKind::Spell)
            .unwrap();
        assert_eq!(packet.mana_value(), 4);
        assert!(g.counter_stack_item(packet.id, false));
        optional(&mut g);
        act(&mut g, Action::ChooseOptional { accept: true });
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 4);
        choose(&mut g, |a| matches!(a, Action::Discard { .. }));
        choose(&mut g, |a| matches!(a, Action::Discard { .. }));
        settle(&mut g);
        assert_eq!(g.hand_count(p), 2);
    }
    let p = Player::P0;
    let mut g = scenario(p);
    letters(&mut g, p);
    put(&mut g, p, "Omniscience", Zone::Battlefield);
    let only = put(&mut g, p, "Forest", Zone::Hand);
    let spell = put(&mut g, p, "Divination", Zone::Hand);
    act(
        &mut g,
        Action::AnnounceFree {
            card: spell,
            face: CastFace::Normal,
            flash_surcharge: false,
        },
    );
    pay(&mut g);
    assert!(
        g.counter_stack_item(
            g.state
                .stack
                .iter()
                .find(|item| item.source.unwrap().id == spell && item.kind == StackKind::Spell)
                .unwrap()
                .id,
            false
        )
    );
    optional(&mut g);
    act(&mut g, Action::ChooseOptional { accept: true });
    act(&mut g, Action::Discard { card: only });
    settle(&mut g);
    assert_eq!(g.hand_count(p), 0);
}
