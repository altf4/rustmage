use super::*;

#[test]
fn omnipresence_free_noncreature_cast_still_pays_tax_and_duplicate_sources_offer_one_action() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let first = put(&mut g, player, "Omnipresence", Zone::Battlefield);
        let second = put(&mut g, player, "Omnipresence", Zone::Battlefield);
        put(&mut g, player, "Bear Cub", Zone::Battlefield);
        put(
            &mut g,
            player.other(),
            "Thalia, the Survivor",
            Zone::Battlefield,
        );
        let shock = put(&mut g, player, "Shock", Zone::Hand);
        let action = Action::AnnounceFree {
            card: shock,
            face: CastFace::Normal,
            flash_surcharge: false,
        };
        g.state.players[player.index()].mana = [0; 6];
        assert!(!g.legal_actions().contains(&action));
        g.state.players[player.index()].mana[5] = 1;
        assert_eq!(
            g.legal_actions().iter().filter(|a| **a == action).count(),
            1
        );
        g.move_object(first, Zone::Graveyard);
        assert!(g.legal_actions().contains(&action));
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        act(&mut g, action.clone());
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(player.other()),
            },
        );
        pay(&mut g);
        assert_eq!(g.state.players[player.index()].mana[5], 0);
        settle(&mut g);
        assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
        g.restore(&saved).unwrap();
        g.move_object(second, Zone::Graveyard);
        assert!(!g.legal_actions().contains(&action));
    }
}

#[test]
fn omnipresence_free_x_is_zero_and_copied_source_grants_permission() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        put(&mut g, player, "Bear Cub", Zone::Battlefield);
        put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, player, "Worldsoul's Rage", Zone::Hand);
        let action = Action::AnnounceFree {
            card: spell,
            face: CastFace::Normal,
            flash_surcharge: false,
        };
        g.state.players[player.index()].mana = [0; 6];
        assert!(!g.legal_actions().contains(&action));
        g.apply_temporary_copy(
            g.object_ref(source),
            CopyRecipe::new(Card::from_name("Omnipresence").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert!(g.legal_actions().contains(&action));
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        act(&mut g, action.clone());
        assert_eq!(g.state.announcing.as_ref().unwrap().item.x, Some(0));
        g.restore(&saved).unwrap();
        g.apply_temporary_copy(
            g.object_ref(source),
            CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert!(!g.legal_actions().contains(&action));
    }
}

#[test]
fn omnipresence_free_cast_still_pays_sacrifice_after_count_check() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        put(&mut g, player, "Omnipresence", Zone::Battlefield);
        let sacrifice = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let target = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, player, "Silence the Echo", Zone::Hand);
        g.state.players[player.index()].mana = [0; 6];
        let action = Action::AnnounceFree {
            card: spell,
            face: CastFace::Normal,
            flash_surcharge: false,
        };
        assert!(g.legal_actions().contains(&action));
        act(&mut g, action.clone());
        act(&mut g, Action::ChooseSpellCost { cost: 0 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        let sacrifice_ref = g.object_ref(sacrifice);
        act(
            &mut g,
            Action::ChooseAdditionalCost {
                object: sacrifice_ref,
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            pay(&mut g);
            assert_eq!(g.object_view(sacrifice).zone, Zone::Graveyard);
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        }
    }
}
