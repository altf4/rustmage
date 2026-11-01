use super::*;

fn pay_vanguard(g: &mut Game, action: Action, target: Option<CardId>) {
    act(g, action);
    for _ in 0..32 {
        if g.state.announcing.is_none() {
            return;
        }
        choose(g, |a| match a {
            Action::ChooseTarget {
                target: Target::Card(card),
            } => Some(*card) == target,
            Action::ChooseSpellCost { .. } | Action::FinishTargets | Action::PayCosts { .. } => {
                true
            }
            _ => false,
        });
    }
    panic!("Vanguard announcement did not finish");
}

#[test]
fn seed_suture_opposing_target_counter_replacement_and_countered_copy() {
    for player in [Player::P0, Player::P1] {
        for countered in [false, true] {
            let mut g = scenario(player);
            let source = put(
                &mut g,
                player,
                "Vigorbloom Vanguard // Seed Suture",
                Zone::Hand,
            );
            pay_vanguard(&mut g, Action::AnnounceCast { card: source }, None);
            settle(&mut g);
            let permanent = g.object_ref(source);
            let prepared = g
                .state
                .preparations
                .as_ref()
                .unwrap()
                .iter()
                .find(|r| r.permanent == permanent)
                .unwrap()
                .spell_copy;
            let target = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
            put(&mut g, player.other(), "Doubling Season", Zone::Battlefield);
            let negate = put(&mut g, player.other(), "Negate", Zone::Hand);
            let life = g.state.players[player.index()].life;
            pay_vanguard(
                &mut g,
                Action::AnnounceExile {
                    card: prepared.card,
                },
                Some(target),
            );
            assert!(!g.is_prepared(permanent));
            if countered {
                act(&mut g, Action::Pass);
                cast(&mut g, negate, Some(Target::Spell(prepared.card)));
            }
            let saved = g.snapshot();
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                settle(&mut g);
                assert_eq!(
                    g.counter_count(target, CounterKind::PlusOne),
                    if countered { 0 } else { 2 }
                );
                assert_eq!(
                    g.state.players[player.index()].life,
                    life + i128::from(!countered)
                );
                assert_eq!(g.object_view(prepared.card).zone, Zone::Outside);
                assert!(!g.is_prepared(permanent));
            }
        }
    }
}
