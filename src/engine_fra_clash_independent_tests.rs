use super::*;

fn independent_clash_wait(g: &mut Game) {
    for _ in 0..48 {
        if matches!(g.state.pending, Pending::TargetLibrary { .. }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Clash owner choice did not start");
}
fn independent_clash_cast(g: &mut Game, source: CardId, target: CardId) {
    act(g, Action::AnnounceCast { card: source });
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(g);
    independent_clash_wait(g);
}

#[test]
fn clash_top_damage_prevention_and_multiplier_suspend_for_owner_choice_and_replay() {
    for p in [Player::P0, Player::P1] {
        for prevent_first in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, "Clash of Elements", Zone::Hand);
            put(&mut g, p, "Twinflame Tyrant", Zone::Battlefield);
            independent_clash_cast(&mut g, spell, target);
            let prevention = put(&mut g, p.other(), "Forest", Zone::Battlefield);
            let timestamp = g.next_role_timestamp();
            let prevention_ref = g.object_ref(prevention);
            let spell_ref = g.object_ref(spell);
            g.state
                .roles
                .get_or_insert_default()
                .source_linked_effects
                .push(SourceLinkedEffectView {
                    source: prevention_ref,
                    recipient: spell_ref,
                    controller: p.other(),
                    kind: SourceLinkedEffectKind::PreventDamageBy,
                    timestamp,
                });
            let original = g.object_ref(target);
            let life = g.state.players[p.other().index()].life;
            act(
                &mut g,
                Action::ChooseLibraryDestination {
                    object: original,
                    destination: LibraryDestination::Top,
                },
            );
            assert_eq!(g.object_view(target).zone, Zone::Library);
            assert_eq!(g.state.players[p.other().index()].life, life);
            assert!(g.state.replacements.is_some());
            let before = g.snapshot();
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&before).unwrap();
                }
                let choose_prevention = prevent_first;
                choose(&mut g, |a| {
                    if choose_prevention {
                        matches!(a, Action::ChooseDamagePrevention { .. })
                    } else {
                        matches!(a, Action::ChooseReplacement { .. })
                    }
                });
                for _ in 0..32 {
                    if g.state.replacements.is_none() {
                        break;
                    }
                    choose(&mut g, |a| {
                        matches!(
                            a,
                            Action::ChooseReplacement { .. }
                                | Action::ChooseDamagePrevention { .. }
                        )
                    });
                }
                settle(&mut g);
                assert_eq!(g.state.players[p.other().index()].life, life);
                assert_eq!(g.object_view(target).zone, Zone::Library);
            }
        }
    }
}

#[test]
fn clash_token_put_on_top_still_deals_damage_and_ceases_afterwards() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.create_tokens(p.other(), Card::test_index(5453), 1, false);
        let target = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(i, o)| {
                (o.zone == Zone::Battlefield && o.controller == p.other() && o.is_token())
                    .then_some(i as CardId)
            })
            .unwrap();
        let spell = put(&mut g, p, "Clash of Elements", Zone::Hand);
        let life = g.state.players[p.other().index()].life;
        independent_clash_cast(&mut g, spell, target);
        let object = g.object_ref(target);
        act(
            &mut g,
            Action::ChooseLibraryDestination {
                object,
                destination: LibraryDestination::Top,
            },
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, life - 2);
        assert!(!g.state.players[p.other().index()].library.contains(&target));
    }
}
