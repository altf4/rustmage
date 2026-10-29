use super::*;

fn activate_torch(g: &mut Game, chandra: CardId) {
    act(g, Action::AnnounceActivate { card: chandra, ability: 0, counter_payment: Counters::default() });
    pay(g);
    g.resolve_top();
}

#[test]
fn chandra_can_cast_the_adventure_face_from_exile_and_replay() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let chandra = put(&mut g, player, "Chandra, Torch of Defiance", Zone::Battlefield);
        g.state.objects[chandra as usize].counters.loyalty = 4;
        let goblin = put(&mut g, player, "Great Ugly-Looking Goblin // Clap! Snap!", Zone::Library);
        g.state.players[player.index()].mana = [0, 0, 1, 0, 0, 2];
        let life = g.state.players[player.other().index()].life;
        activate_torch(&mut g, chandra);
        let saved = g.snapshot();
        for _ in 0..2 {
            let object = g.object_ref(goblin);
            act(&mut g, Action::CastExiledDuringResolution { object, face: CastFace::Adventure });
            pay(&mut g);
            let spell = g.state.stack.last().unwrap();
            assert_eq!(spell.cast.unwrap().face, CastFace::Adventure);
            assert_eq!(g.state.players[player.index()].mana, [0; 6]);
            act(&mut g, Action::FinishResolutionCasts);
            assert_eq!(g.state.players[player.other().index()].life, life);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn chandra_paid_permission_offers_alternate_cost_and_additional_sacrifice() {
    for player in [Player::P0, Player::P1] {
        for route in [0u16, 1] {
            let mut g = scenario(player);
            let chandra = put(&mut g, player, "Chandra, Torch of Defiance", Zone::Battlefield);
            g.state.objects[chandra as usize].counters.loyalty = 4;
            let sacrifice = put(&mut g, player, "Bear Cub", Zone::Battlefield);
            let target = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
            let spell = put(&mut g, player, "Silence the Echo", Zone::Library);
            g.state.players[player.index()].mana = [0, 0, 1, 0, 0, 5];
            activate_torch(&mut g, chandra);
            let saved = g.snapshot();
            for _ in 0..2 {
                let object = g.object_ref(spell);
                act(&mut g, Action::CastExiledDuringResolution { object, face: CastFace::Normal });
                act(&mut g, Action::ChooseSpellCost { cost: route });
                act(&mut g, Action::ChooseTarget { target: Target::Card(target) });
                if route == 0 {
                    let object = g.object_ref(sacrifice);
                    act(&mut g, Action::ChooseAdditionalCost { object });
                }
                if g.legal_actions().contains(&Action::FinishTargets) { act(&mut g, Action::FinishTargets); }
                pay(&mut g);
                assert_eq!(g.state.players[player.index()].mana[5], if route == 0 { 4 } else { 1 });
                assert_eq!(g.object_view(sacrifice).zone, if route == 0 { Zone::Graveyard } else { Zone::Battlefield });
                assert_eq!(g.state.resolution_free_casts.as_ref().unwrap().casts_remaining, Some(0));
                act(&mut g, Action::FinishResolutionCasts);
                g.restore(&saved).unwrap();
            }
        }
    }
}

#[test]
fn chandra_loyalty_controller_and_damage_source_survive_control_and_zone_changes() {
    for player in [Player::P0, Player::P1] {
        for departure in [false, true] {
            for prevent in [false, true] {
                let mut g = scenario(player);
                let chandra = put(&mut g, player, "Chandra, Torch of Defiance", Zone::Battlefield);
                g.state.objects[chandra as usize].counters.loyalty = 4;
                put(&mut g, player, "Mountain", Zone::Library);
                let initial = [g.state.players[0].life, g.state.players[1].life];
                let original = g.object_ref(chandra);
                act(&mut g, Action::AnnounceActivate { card: chandra, ability: 0, counter_payment: Counters::default() });
                pay(&mut g);
                if prevent {
                    let source = put(&mut g, player.other(), "Forest", Zone::Battlefield);
                    let source = g.object_ref(source);
                    let timestamp = g.next_role_timestamp();
                    g.state.roles.get_or_insert_default().source_linked_effects.push(SourceLinkedEffectView {
                        source, recipient: original, controller: player.other(),
                        kind: SourceLinkedEffectKind::PreventDamageBy, timestamp,
                    });
                }
                g.change_controller(original, player.other());
                if departure { g.move_object(chandra, Zone::Graveyard); }
                g.resolve_top();
                let saved = g.snapshot();
                for _ in 0..2 {
                    act(&mut g, Action::FinishResolutionCasts);
                    assert_eq!(g.state.players[player.index()].life, initial[player.index()]);
                    assert_eq!(g.state.players[player.other().index()].life,
                        initial[player.other().index()] - if prevent { 0 } else { 2 });
                    g.restore(&saved).unwrap();
                }
            }
        }
    }
}
