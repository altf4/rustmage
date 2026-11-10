//! A live retention rule filters the pool without changing mana provenance.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const MIXED: Mana = [0, 1, 0, 2, 0, 0];
const TWO_RED: Mana = [0, 0, 0, 2, 0, 0];
const RETAIN_RED: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::RetainManaColor(ChoiceColor::Red)],
    }],
    ..CardRules::EMPTY
};
const TAGGED_MANA: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::Fixed(MIXED),
            quality: &ManaQuality {
                restriction: ManaRestriction::CreatureSpells,
                spell_bonus: ManaSpellBonus::Uncounterable,
                retention: ManaRetention::Step,
            },
        })],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn mixed_lot_keeps_only_red_with_its_original_restriction_and_rider() {
    let _rules = fixture_rules::activate(&[
        (card("Hill Giant"), &RETAIN_RED),
        (card("Short Bow"), &TAGGED_MANA),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        game.state.step = Step::Upkeep;
        game.state.players[player.index()].mana = [0; 6];
        game.state.players[player.other().index()].mana = MIXED;
        put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let mana = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let creature = put(&mut game, player, "Goblin Piker", Zone::Hand);
        let instant = put(&mut game, player, "Shock", Zone::Hand);
        choose(&mut game, |action| {
            matches!(
                action, Action::ActivateMana { card, production, .. } if *card == mana && *production == MIXED
            )
        });
        assert_eq!(game.state.players[player.index()].mana, MIXED);
        pass_pair(&mut game);
        pass_pair(&mut game);
        assert_eq!(game.state.step, Step::Main1);
        assert_eq!(game.state.players[player.index()].mana, TWO_RED);
        assert_eq!(game.state.players[player.other().index()].mana, [0; 6]);
        let lots = game.observe(player).mana_lots;
        assert_eq!(lots.len(), 1);
        assert_eq!(lots[0].amounts, TWO_RED);
        assert_eq!(lots[0].restriction, ManaRestrictionView::CreatureSpells);
        assert_eq!(lots[0].spell_bonus, ManaSpellBonusView::Uncounterable);
        assert_eq!(lots[0].retention, ManaRetention::Step);
        reject(&mut game, Action::AnnounceCast { card: instant });
        cast(&mut game, creature, None);
        let spell = game.state.stack.last().unwrap().id;
        assert!(
            game.observe(player)
                .uncounterable_spell_ids
                .contains(&spell)
        );
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        assert!(game.observe(player).mana_lots.is_empty());
    }
}

#[test]
fn combat_red_survives_once_and_retention_tracks_current_control_and_abilities() {
    let _rules = fixture_rules::activate(&[(card("Hill Giant"), &RETAIN_RED)]);
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.step = Step::EndCombat;
    let provider = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let reference = game.object_ref(provider);
    game.state.players[player.index()].mana = MIXED;
    game.add_combat_red(player, 3);
    act(&mut game, Action::Pass);
    let saved = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == Action::Pass)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, Action::Pass);
    indexed
        .apply_index(player.other(), indexed.revision(), index)
        .unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.state.step, Step::Main2);
    assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 5, 0, 0]);
    assert_eq!(game.state.players[player.index()].combat_red_mana, 0);

    game.state.step = Step::Cleanup;
    game.finish_cleanup();
    assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 5, 0, 0]);

    indexed.restore(&saved).unwrap();
    indexed.change_controller(reference, player.other());
    indexed.state.players[player.other().index()].mana = MIXED;
    assert_eq!(indexed.state.players[player.index()].mana[3], 5);
    act(&mut indexed, Action::Pass);
    assert_eq!(indexed.state.players[player.index()].mana, [0; 6]);
    assert_eq!(indexed.state.players[player.other().index()].mana, TWO_RED);

    let mut blanked = scenario(player);
    let provider = put(&mut blanked, player, "Hill Giant", Zone::Battlefield);
    let aura = put(&mut blanked, player, "Witness Protection", Zone::Hand);
    cast(&mut blanked, aura, Some(Target::Card(provider)));
    settle(&mut blanked);
    blanked.state.players[player.index()].mana = MIXED;
    let mut departed = blanked.clone();
    blanked.clear_step_mana(false, true);
    assert_eq!(blanked.state.players[player.index()].mana, [0; 6]);

    departed.move_object(aura, Zone::Graveyard);
    departed.clear_step_mana(false, true);
    assert_eq!(departed.state.players[player.index()].mana, TWO_RED);
    departed.move_object(provider, Zone::Exile);
    assert_eq!(departed.state.players[player.index()].mana, TWO_RED);
    departed.clear_step_mana(false, true);
    assert_eq!(departed.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn retained_ventmaw_red_loses_its_original_turn_deadline() {
    let _rules = fixture_rules::activate(&[(card("Hill Giant"), &RETAIN_RED)]);
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.players[player.index()].mana = [0; 6];
    let provider = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let ventmaw = put(&mut game, player, "Savage Ventmaw", Zone::Battlefield);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: ventmaw });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 3, 3, 0]);
    let lot = game.observe(player).mana_lots.pop().unwrap();
    assert_eq!(lot.retention, ManaRetention::Turn);

    let turn = game.state.turn;
    for _ in 0..24 {
        if game.state.turn != turn {
            break;
        }
        match game.state.pending {
            Pending::Blockers => act(&mut game, Action::FinishBlockers),
            Pending::Priority => act(&mut game, Action::Pass),
            ref pending => panic!("unexpected pending combat decision {pending:?}"),
        }
    }
    assert_eq!(game.state.turn, turn + 1);
    assert_eq!(game.state.step, Step::Upkeep);
    assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 3, 0, 0]);
    assert!(game.observe(player).mana_lots.is_empty());

    game.move_object(provider, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].mana[3], 3);
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::Draw);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}
