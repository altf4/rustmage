mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ONE: ManaCost = ManaCost {
    colored: [0; 6],
    generic: 1,
};
const GRANTER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::OwnTurnGraveyardFlashbackExceptSubtype("Lesson")],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::OwnTurnGraveyardSubtypeFlashback {
                subtype: "Lesson",
                cost: ONE,
            }],
        },
    ],
    ..CardRules::EMPTY
};
const LESSON: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Draw(PlayerSet::Controller, 1)],
    ),
    ..CardRules::EMPTY
};

fn fixture() -> fixture_rules::Guard {
    fixture_rules::activate(&[
        (Card::HillGiant, &GRANTER),
        (Card::from_name("Accumulate Wisdom").unwrap(), &LESSON),
    ])
}

fn fixed(card: CardId) -> Action {
    Action::AnnounceGrantedFixedFlashback {
        card,
        cost: ONE,
        flash_surcharge: false,
    }
}

fn printed(card: CardId) -> Action {
    Action::AnnounceGrantedFlashback {
        card,
        flash_surcharge: false,
    }
}

#[test]
fn live_lesson_flashback_requires_own_turn_control_and_own_graveyard() {
    let _rules = fixture();
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let lesson = put(&mut game, player, "Accumulate Wisdom", Zone::Graveyard);
        let enemy = put(
            &mut game,
            player.other(),
            "Accumulate Wisdom",
            Zone::Graveyard,
        );
        let nonlesson = put(&mut game, player, "Opt", Zone::Graveyard);
        reject(&mut game, printed(lesson));
        reject(&mut game, fixed(enemy));
        reject(&mut game, fixed(nonlesson));
        assert!(game.legal_actions().contains(&printed(nonlesson)));

        let mut stolen = game.clone();
        stolen.record_control(stolen.object_ref(source), player.other(), None);
        reject(&mut stolen, fixed(lesson));
        reject(&mut stolen, printed(nonlesson));
        let mut off_turn = game.clone();
        off_turn.state.active = player.other();
        reject(&mut off_turn, fixed(lesson));
        reject(&mut off_turn, printed(nonlesson));

        game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
        let before = game.snapshot();
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == fixed(lesson))
            .unwrap();
        let mut indexed = game.clone();
        indexed.apply_index(player, revision, index).unwrap();
        act(&mut game, fixed(lesson));
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&before).unwrap();
        act(&mut indexed, fixed(lesson));
        assert_eq!(game.snapshot(), indexed.snapshot());
        pay(&mut game);
        let cast = game.state.stack.last().unwrap().cast.unwrap();
        assert_eq!(cast.method, CastMethod::GrantedFixedFlashback(ONE));
        assert_eq!(cast.payment, [0, 0, 0, 0, 0, 1]);
        game.move_object(source, Zone::Exile);
        settle(&mut game);
        assert_eq!(game.object_view(lesson).zone, Zone::Exile);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    }
}

#[test]
fn printed_cost_flashback_pays_the_printed_x_and_exiles_on_resolution() {
    let _rules = fixture();
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, player, "Exsanguinate", Zone::Graveyard);
    game.state.players[player.index()].mana = [0, 0, 2, 0, 0, 3];
    act(&mut game, printed(spell));
    reject(&mut game, Action::ChooseX { value: 4 });
    act(&mut game, Action::ChooseX { value: 3 });
    pay(&mut game);
    let packet = game.state.stack.last().unwrap();
    assert_eq!(packet.x, Some(3));
    assert_eq!(packet.mana_value(), 5);
    assert_eq!(packet.cast.unwrap().payment, [0, 0, 2, 0, 0, 3]);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    assert_eq!(game.state.players[player.other().index()].life, 17);
}
