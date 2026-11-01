use super::*;

fn draft(g: &mut Game, owner: Player, name: &str) -> CardId {
    let id = put(g, owner, "Bear Cub", Zone::Hand);
    g.state.objects[id as usize].card = Card::from_name(name).unwrap();
    id
}

#[test]
fn wrath_discount_uses_control_not_ownership_and_deals_exact_four() {
    for p in [Player::P0, Player::P1] {
        for control in [false, true] {
            let mut g = scenario(p);
            let legend = put(
                &mut g,
                p.other(),
                "Haliya, Guided by Light",
                Zone::Battlefield,
            );
            if control {
                let reference = g.object_ref(legend);
                g.change_controller(reference, p);
            }
            let stone = put(&mut g, p, "The Mind Stone", Zone::Battlefield);
            g.state.objects[stone as usize].tapped = true;
            let target = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
            g.add_counter(target, CounterKind::PlusOne, 1);
            let spell = draft(&mut g, p, "Wrath of the Bloodmane");
            let announce = Action::AnnounceCast { card: spell };
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
            if !control {
                reject(&mut g, announce.clone());
                assert_eq!(g.object_view(spell).zone, Zone::Hand);
                g.state.players[p.index()].mana[5] = 2;
            }
            act(&mut g, announce);
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            super::contracts::branch_contract(g.clone());
            pay(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let checkpoint = g.snapshot();
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
            assert_eq!(g.object_view(target).damage, 4);
            g.restore(&checkpoint).unwrap();
            settle(&mut g);
            assert_eq!(g.object_view(target).damage, 4);
        }
    }
}

#[test]
fn wrath_fizzles_if_target_becomes_hexproof_before_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let spell = draft(&mut g, p, "Wrath of the Bloodmane");
        cast(&mut g, spell, Some(Target::Card(target)));
        g.state.objects[target as usize].gained_keywords = Keywords::HEXPROOF;
        settle(&mut g);
        assert_eq!(g.object_view(target).damage, 0);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    }
}

#[test]
fn twisted_fates_noncreature_target_and_current_player_creatures() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let artifact = put(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
        let old = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = draft(&mut g, p, "Twisted Fates");
        act(&mut g, Action::AnnounceCast { card: spell });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        pay(&mut g);
        let new = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let snapshot = g.snapshot();
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.object_view(artifact).zone, Zone::Graveyard);
        assert_eq!(g.counter_count(old, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(new, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
        g.restore(&snapshot).unwrap();
        settle(&mut g);
        assert_eq!(g.counter_count(new, CounterKind::PlusOne), 1);
    }
}

#[test]
fn twisted_fates_fizzles_when_both_targets_become_illegal() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let artifact = put(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
        let creature = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = draft(&mut g, p, "Twisted Fates");
        act(&mut g, Action::AnnounceCast { card: spell });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        g.move_object(artifact, Zone::Exile);
        g.state.players[p.other().index()].hexproof_until_end_turn = true;
        settle(&mut g);
        assert_eq!(g.object_view(artifact).zone, Zone::Exile);
        assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 0);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    }
}
