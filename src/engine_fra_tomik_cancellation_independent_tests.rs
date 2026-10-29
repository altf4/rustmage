use super::*;

#[test]
fn suppressed_copied_tomik_permits_two_attackers_and_reversible_cancellation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.apply_temporary_copy(
            g.object_ref(provider),
            CopyRecipe::new(Card::from_name("Tomik, Orzhov Lawmage").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        let walker = put(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.state.objects[walker as usize].counters.loyalty = 5;
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let aura = put(&mut g, p, "Fresh Start", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(provider));
        let defender = g.object_ref(walker);
        assert!(!g.planeswalker_attack_limited(defender));
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::AttackPlaneswalker { card: a, defender });
        let selected = g.snapshot();
        assert!(g.apply(p.other(), Action::CancelAttackDeclaration).is_err());
        assert_eq!(g.snapshot(), selected);
        act(&mut g, Action::CancelAttackDeclaration);
        let cleared = g.snapshot();
        assert!(g.apply(p, Action::CancelAttackDeclaration).is_err());
        assert_eq!(g.snapshot(), cleared);
        act(&mut g, Action::AttackPlaneswalker { card: a, defender });
        act(&mut g, Action::AttackPlaneswalker { card: b, defender });
        assert!(g.legal_actions().contains(&Action::FinishAttackers));
        g.restore(&selected).unwrap();
        assert!(!g.planeswalker_attack_limited(defender));
        assert!(
            g.legal_actions()
                .contains(&Action::AttackPlaneswalker { card: b, defender })
        );
    }
}

#[test]
fn copied_tomik_restricts_again_after_suppression_source_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.apply_temporary_copy(
            g.object_ref(provider),
            CopyRecipe::new(Card::from_name("Tomik, Orzhov Lawmage").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        let walker = put(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.state.objects[walker as usize].counters.loyalty = 5;
        let aura = put(&mut g, p, "Fresh Start", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(provider));
        let defender = g.object_ref(walker);
        assert!(!g.planeswalker_attack_limited(defender));
        let saved = g.snapshot();
        for _ in 0..2 {
            g.move_object(aura, Zone::Graveyard);
            assert!(g.planeswalker_attack_limited(defender));
            g.restore(&saved).unwrap();
            assert!(!g.planeswalker_attack_limited(defender));
        }
    }
}

#[test]
fn copied_tomik_selected_attack_tax_abort_preserves_choices_and_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.apply_temporary_copy(
            g.object_ref(provider),
            CopyRecipe::new(Card::from_name("Tomik, Orzhov Lawmage").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        put(
            &mut g,
            p.other(),
            "Dáin, Lord of the Iron Hills",
            Zone::Battlefield,
        );
        put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        g.refresh_enduring_stories();
        let walker = put(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.state.objects[walker as usize].counters.loyalty = 5;
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: a });
        let defender = g.object_ref(walker);
        act(&mut g, Action::AttackPlaneswalker { card: b, defender });
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::FinishAttackers);
            act(&mut g, Action::TapMana { card: land });
            reject(&mut g, Action::PayAttackCost { payment: [0; 6] });
            act(&mut g, Action::AbortAttackPayment);
            if matches!(g.state.pending, Pending::ManaRollback { .. }) {
                act(&mut g, Action::ChooseManaRollback { keep: false });
            }
            assert_eq!(g.snapshot(), saved);
        }
    }
}
