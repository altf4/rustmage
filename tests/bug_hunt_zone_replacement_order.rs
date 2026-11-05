mod support;
use rustmage as _;
use rustmage_core::engine::*;
use support::*;

#[test]
fn active_player_has_the_first_simultaneous_death_replacement_choice() {
    let active = Player::P1;
    let nonactive = active.other();
    let mut game = scenario(active);
    let nonactive_head = put(&mut game, nonactive, "Head of the Hunt", Zone::Battlefield);
    let active_head = put(&mut game, active, "Head of the Hunt", Zone::Battlefield);
    assert!(nonactive_head < active_head);
    put(&mut game, active, "Rest in Peace", Zone::Battlefield);
    let wrath = put(&mut game, active, "Day of Judgment", Zone::Hand);
    cast(&mut game, wrath, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert!(matches!(
        game.state.pending,
        Pending::ZoneReplacement { .. }
    ));
    assert_eq!(
        game.actor(),
        Some(active),
        "simultaneous replacement choices are made in APNAP order"
    );

    let source = game.object_ref(nonactive_head);
    let choose_head = Action::ChooseZoneReplacement {
        effect: ZoneReplacementKind::ExileReflexive { source, ability: 0 },
    };
    let before_choices = game.snapshot();
    let mut plain_exile = game.clone();
    act(
        &mut plain_exile,
        Action::ChooseZoneReplacement {
            effect: ZoneReplacementKind::Exile,
        },
    );
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == choose_head)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(active, game.revision(), index).unwrap();
    act(&mut game, choose_head);
    assert_eq!(indexed.snapshot(), game.snapshot());
    assert_eq!(game.actor(), Some(nonactive));
    assert_eq!(game.object_view(active_head).zone, Zone::Battlefield);
    assert_eq!(game.object_view(nonactive_head).zone, Zone::Battlefield);
    for (branch, effect) in [
        (
            &game,
            ZoneReplacementKind::ExileReflexive { source, ability: 0 },
        ),
        (&plain_exile, ZoneReplacementKind::Exile),
    ] {
        assert_eq!(
            branch
                .observe(nonactive)
                .zone_replacement_choice
                .unwrap()
                .prior_public_choices,
            [ZoneReplacementPickView {
                object: branch.object_ref(active_head),
                player: active,
                effect,
            }]
        );
    }

    act(
        &mut game,
        Action::ChooseZoneReplacement {
            effect: ZoneReplacementKind::Exile,
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(active_head).zone, Zone::Exile);
    assert_eq!(game.object_view(nonactive_head).zone, Zone::Exile);
    let wolves: Vec<_> = game
        .observe(nonactive)
        .objects
        .into_iter()
        .filter(|o| o.zone == Zone::Battlefield && o.is_token() && o.card.has_subtype("Wolf"))
        .collect();
    assert_eq!(wolves.len(), 1);
    assert_eq!(wolves[0].controller, nonactive);
    indexed.restore(&before_choices).unwrap();
    assert_eq!(indexed.actor(), Some(active));
}

#[test]
fn simultaneous_private_discards_do_not_reveal_prior_card_choices() {
    let active = Player::P1;
    let nonactive = active.other();
    let mut game = scenario(active);
    let private = put(&mut game, active, "Darksteel Colossus", Zone::Hand);
    let other = put(&mut game, nonactive, "Darksteel Colossus", Zone::Hand);
    let mage = put(&mut game, active, "Dragon Mage", Zone::Battlefield);
    put(&mut game, active, "Rest in Peace", Zone::Battlefield);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: mage });
    act(&mut game, Action::FinishAttackers);
    for _ in 0..20 {
        let action = match game.state.pending {
            Pending::Blockers => Action::FinishBlockers,
            Pending::EffectDiscard { player, .. } => Action::Discard {
                card: if player == active { private } else { other },
            },
            Pending::Priority => Action::Pass,
            Pending::ZoneReplacement { .. } => break,
            ref pending => panic!("unexpected Dragon Mage decision {pending:?}"),
        };
        act(&mut game, action);
    }
    assert_eq!(
        game.state.pending,
        Pending::ZoneReplacement { player: active }
    );
    act(
        &mut game,
        Action::ChooseZoneReplacement {
            effect: ZoneReplacementKind::RevealShuffle,
        },
    );
    assert_eq!(
        game.state.pending,
        Pending::ZoneReplacement { player: nonactive }
    );
    let view = game.observe(nonactive);
    assert!(
        view.zone_replacement_choice
            .unwrap()
            .prior_public_choices
            .is_empty()
    );
    assert!(!view.objects.iter().any(|o| o.id == private));
    act(
        &mut game,
        Action::ChooseZoneReplacement {
            effect: ZoneReplacementKind::Exile,
        },
    );
    assert_eq!(game.object_view(other).zone, Zone::Exile);
}
