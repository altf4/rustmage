//! Regression for a combat restriction with layered unrelated providers.
use super::*;

#[test]
fn combat_restriction_checks_source_fragment_before_layer_availability() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        // Samut is registered before Yuriko, but does not have Yuriko's fragment.
        let samut = put(&mut g, p, "Samut, Tyrant of Naktamun", Zone::Battlefield);
        let yuriko = put(&mut g, p, "Yuriko, Blade of the Mighty", Zone::Battlefield);
        let aura = put(&mut g, p.other(), "Fresh Start", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(samut));
        assert!(g.evaluated_abilities().is_some());
        g.state.step = Step::BeginCombat;
        assert!(g.nonmana_actions_prohibited());
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: shock });
        super::super::contracts::branch_contract(g.clone());
        // Removing the sole applicable provider immediately restores actions.
        g.move_object(yuriko, Zone::Graveyard);
        assert!(!g.nonmana_actions_prohibited());
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: shock })
        );
    }
}

#[test]
fn combat_restriction_tracks_temporary_copies_and_loses_old_ability() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let reference = g.object_ref(source);
        let yuriko = Card::from_name("Yuriko, Blade of the Mighty").unwrap();
        g.state.step = Step::BeginCombat;
        assert!(!g.nonmana_actions_prohibited());
        g.apply_temporary_copy(reference, CopyRecipe::new(yuriko), CopyDeadline::EndOfTurn);
        assert!(g.nonmana_actions_prohibited());
        super::super::contracts::branch_contract(g.clone());
        g.apply_temporary_copy(
            reference,
            CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert!(!g.nonmana_actions_prohibited());
    }
}

#[test]
fn samut_split_second_blocks_casting_from_a_resolving_effect() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let samut = put(&mut g, p, "Samut, Tyrant of Naktamun", Zone::Battlefield);
        let first = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, first, Some(Target::Player(p.other())));
        let exile = put(&mut g, p, "Shock", Zone::Exile);
        let object = g.object_ref(exile);
        // Model the pending choice inside a resolving cascade or similar effect.
        g.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
            siege_transformed: false,
            paid: None,
            casts_remaining: None,
            pending_cast: None,
            alternative_cost: None,
            maximum_mana_value: None,
            origin: Zone::Exile,
            exile_on_graveyard: false,
            allow_lands: false,
            player: p,
            remaining: vec![object],
        }));
        g.state.pending = Pending::ResolutionCasts { player: p };
        let cast = Action::CastExiledDuringResolution {
            object,
            face: CastFace::Normal,
        };
        assert!(!g.legal_actions().contains(&cast));
        assert!(g.legal_actions().contains(&Action::FinishResolutionCasts));
        g.move_object(samut, Zone::Graveyard);
        assert!(g.legal_actions().contains(&cast));
    }
}
