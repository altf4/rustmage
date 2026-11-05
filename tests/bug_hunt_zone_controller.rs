mod support;
use rustmage as _;
use rustmage_core::engine::*;
use support::*;

#[test]
fn stolen_colossus_controller_chooses_its_countered_spell_replacement() {
    let caster = Player::P0;
    let owner = caster.other();
    let mut game = scenario(caster);
    put(&mut game, owner, "Rest in Peace", Zone::Battlefield);
    let colossus = put(&mut game, owner, "Darksteel Colossus", Zone::Library);
    let steal = put(&mut game, caster, "Inside Information", Zone::Hand);
    let cancel = put(&mut game, owner, "Cancel", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: steal });
    act(&mut game, Action::ChooseX { value: 1 });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(owner),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(colossus).zone, Zone::Exile);
    act(
        &mut game,
        Action::AnnounceExileLife {
            card: colossus,
            face: CastFace::Normal,
            flash_surcharge: false,
        },
    );
    for _ in 0..4 {
        match game.state.pending {
            Pending::CastCosts { .. } => {
                choose(&mut game, |a| matches!(a, Action::ChooseCastCost { .. }));
            }
            Pending::PayCosts { .. } => pay(&mut game),
            Pending::Priority => break,
            ref pending => panic!("unexpected Colossus announcement state {pending:?}"),
        }
    }
    assert_eq!(game.object_view(colossus).zone, Zone::Stack);
    assert_eq!(game.object_view(colossus).owner, owner);
    assert_eq!(game.state.stack.last().unwrap().controller, caster);
    assert_eq!(game.state.players[caster.index()].life, 9);
    act(&mut game, Action::Pass);
    cast(&mut game, cancel, Some(Target::Spell(colossus)));
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert!(matches!(
        game.state.pending,
        Pending::ZoneReplacement { .. }
    ));
    assert_eq!(
        game.actor(),
        Some(caster),
        "the spell controller, not the card owner, chooses"
    );

    let checkpoint = game.snapshot();
    let exile = Action::ChooseZoneReplacement {
        effect: ZoneReplacementKind::Exile,
    };
    assert_eq!(game.apply(owner, exile), Err(Error::WrongActor));
    assert_eq!(game.snapshot(), checkpoint);
    for (effect, destination) in [
        (ZoneReplacementKind::Exile, Zone::Exile),
        (ZoneReplacementKind::RevealShuffle, Zone::Library),
    ] {
        let action = Action::ChooseZoneReplacement { effect };
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let mut indexed = game.clone();
        let mut raw = game.clone();
        indexed.apply_index(caster, game.revision(), index).unwrap();
        act(&mut raw, action);
        assert_eq!(indexed.snapshot(), raw.snapshot());
        assert_eq!(raw.object_view(colossus).zone, destination);
        assert_eq!(raw.object_view(colossus).owner, owner);
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(indexed.actor(), Some(caster));
    }
}
