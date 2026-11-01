//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;
pub(super) fn drive(g: &mut Game) -> usize {
    let mut scries = 0;
    for _ in 0..96 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return scries;
        }
        let actions = g.legal_actions();
        let action = if let Some(a) = actions.iter().find(|a| {
            matches!(
                a,
                Action::ArrangeLibrary {
                    destination: LibraryDestination::Bottom,
                    ..
                }
            )
        }) {
            let p = g.actor().unwrap();
            let view = g.observe(p).library_choice.unwrap();
            assert_eq!(view.remaining.len(), 1);
            assert!(g.observe(p.other()).library_choice.is_none());
            scries += 1;
            a.clone()
        } else {
            actions
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::PutTrigger { .. } | Action::FinishTargets | Action::Pass
                    )
                })
                .unwrap_or_else(|| panic!("explicit choice {:?}", g.state.pending))
        };
        act(g, action);
    }
    panic!("death/return resolution budget");
}

fn spell(g: &mut Game, p: Player, name: &str, mana: Mana, target: Option<Target>) -> CardId {
    let id = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, target);
    id
}
fn bee(g: &mut Game, p: Player) -> CardId {
    let id = spell(g, p, "Great Fierce Bee", [0, 0, 1, 0, 0, 2], None);
    settle(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(stats(g, id), (2, 2));
    assert!(g.keywords(id).contains(Keywords::FLYING));
    id
}
fn staff(g: &mut Game, p: Player, body: CardId) -> CardId {
    let id = put(g, p, "Wizard's Staff", Zone::Hand);
    g.state.players[p.index()].mana = [12; 6];
    cast(g, id, None);
    settle(g);
    act(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(body),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    settle(g);
    id
}
#[test]
fn hob_death_return_bee_batch_other_any_controller_tokens_and_distinct_deaths() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = bee(&mut g, p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        put(&mut g, p, "@token/hob-wolf", Zone::Battlefield);
        spell(&mut g, p, "Day of Judgment", [2, 0, 0, 0, 0, 2], None);
        assert_eq!(drive(&mut g), 1);
        for id in [source, own, enemy] {
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        }
        let mut g = scenario(p);
        let source = bee(&mut g, p);
        for owner in [p, p.other()] {
            let body = put(&mut g, owner, "Bear Cub", Zone::Battlefield);
            spell(
                &mut g,
                p,
                "Murder",
                [0, 0, 2, 0, 0, 1],
                Some(Target::Card(body)),
            );
            assert_eq!(drive(&mut g), 1);
        }
        spell(
            &mut g,
            p,
            "Murder",
            [0, 0, 2, 0, 0, 1],
            Some(Target::Card(source)),
        );
        assert_eq!(drive(&mut g), 0, "self alone is excluded");
        let mut g = scenario(p);
        bee(&mut g, p);
        put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        spell(&mut g, p, "Day of Judgment", [2, 0, 0, 0, 0, 2], None);
        assert_eq!(drive(&mut g), 0, "exiled creatures did not die");
    }
}
#[test]
fn hob_death_return_bee_staff_uses_frozen_predeath_multiplier() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = bee(&mut g, p);
        let staff = staff(&mut g, p, source);
        put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert_eq!(g.object_view(source).additional_triggers, 1);
        spell(&mut g, p, "Day of Judgment", [2, 0, 0, 0, 0, 2], None);
        assert_eq!(
            drive(&mut g),
            2,
            "one batch trigger plus one extra instance, not one per victim"
        );
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.object_view(staff).attached_to, None);
    }
}
