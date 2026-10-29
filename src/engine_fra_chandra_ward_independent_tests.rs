use super::*;

#[test]
fn chandra_paid_exiled_spell_waits_for_ward_and_countering_does_not_undo_cast() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let chandra = put(&mut g, player, "Chandra, Torch of Defiance", Zone::Battlefield);
        g.state.objects[chandra as usize].counters.loyalty = 4;
        let kwia = put(&mut g, player.other(), "Kwia Vigorbloom", Zone::Battlefield);
        let spell = put(&mut g, player, "Shock", Zone::Library);
        g.state.players[player.index()].mana = [0, 0, 0, 1, 0, 2];
        let life = g.state.players[player.other().index()].life;
        act(&mut g, Action::AnnounceActivate { card: chandra, ability: 0, counter_payment: Counters::default() });
        pay(&mut g);
        g.resolve_top();
        let before = g.snapshot();
        for pay_ward in [false, true] {
            g.restore(&before).unwrap();
            let action = Action::CastExiledDuringResolution { object: g.object_ref(spell), face: CastFace::Normal };
            assert!(g.apply(player.other(), action.clone()).is_err());
            assert_eq!(g.snapshot(), before);
            act(&mut g, action);
            act(&mut g, Action::ChooseTarget { target: Target::Card(kwia) });
            if g.legal_actions().contains(&Action::FinishTargets) { act(&mut g, Action::FinishTargets); }
            pay(&mut g);
            assert_eq!(g.state.resolution_free_casts.as_ref().unwrap().casts_remaining, Some(0));
            assert_eq!(g.object_view(spell).zone, Zone::Stack);
            assert_eq!(g.state.players[player.other().index()].life, life);
            act(&mut g, Action::FinishResolutionCasts);
            assert_eq!(g.state.players[player.other().index()].life, life);
            // Ward is put on the stack only after the resolving loyalty ability finishes.
            for _ in 0..24 {
                if g.state.pending != Pending::Priority { break; }
                act(&mut g, Action::Pass);
            }
            assert!(g.legal_actions().contains(&Action::DeclineResolutionMana));
            let ward = g.snapshot();
            let decline = Action::DeclineResolutionMana;
            assert!(g.apply(player.other(), decline).is_err());
            assert_eq!(g.snapshot(), ward);
            if pay_ward { choose(&mut g, |a| matches!(a, Action::PayResolutionMana { .. })); }
            else { act(&mut g, Action::DeclineResolutionMana); }
            settle(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(g.object_view(kwia).damage, if pay_ward { 2 } else { 0 });
            assert_eq!(g.state.players[player.other().index()].life, life);
        }
    }
}
