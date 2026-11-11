//! Exact exile-play permissions and creature entry batches from exile.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const CREATURES_OR_LANDS: TargetPlan = TargetPlan::Single(TargetSpec {
    rule: TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 1 | 2,
        relation: -1,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    },
    min: 1,
    max: 3,
});
const OWNER_PLAY: CardRules = CardRules {
    effect: Effect::Spell(
        CREATURES_OR_LANDS,
        &[Instruction::ExileTargetsPlayForOwnersWhileExiled],
    ),
    ..CardRules::EMPTY
};
const BLINK_GROUP: CardRules = CardRules {
    effect: Effect::Spell(
        CREATURES_OR_LANDS,
        &[Instruction::BlinkImmediately {
            counter: None,
            tapped: false,
        }],
    ),
    ..CardRules::EMPTY
};
const EXILE_ENTRY_OBSERVER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(
            &TriggerCondition::NontokenCreaturesEnterFromExile,
        ),
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

fn fixture_card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn next_main(game: &mut Game) {
    let next = game.state.active.other();
    for _ in 0..80 {
        if game.state.active == next && game.state.step == Step::Main1 {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::FinishAttackers
                    | Action::FinishBlockers
                    | Action::PutTrigger { .. }
            )
        });
    }
    panic!("next main phase exceeded its action budget");
}

#[test]
fn each_exact_exile_card_owner_can_pay_to_cast_or_play_a_land_later() {
    let _rules = fixture_rules::activate(&[(fixture_card("Kellan's Lightblades"), &OWNER_PLAY)]);
    for caster in [Player::P0, Player::P1] {
        let owner = caster.other();
        let mut game = scenario(caster);
        let spell = put(&mut game, caster, "Kellan's Lightblades", Zone::Hand);
        let creature = put(&mut game, owner, "Grizzly Bears", Zone::Battlefield);
        game.state.objects[creature as usize].controller = caster;
        let land = put(&mut game, owner, "Forest", Zone::Battlefield);
        let pack = put(&mut game, caster, "Edgewall Pack", Zone::Hand);
        cast(&mut game, pack, None);
        settle(&mut game);
        let token = (0..game.state.objects.len() as CardId)
            .find(|&id| game.object_view(id).is_token())
            .unwrap();
        let tops = [caster, owner]
            .map(|player| *game.state.players[player.index()].library.last().unwrap());

        announce(&mut game, spell, &[creature, land, token].map(Target::Card));
        settle(&mut game);
        assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
        for (player, top) in [caster, owner].into_iter().zip(tops) {
            assert_eq!(
                game.state.players[player.index()].library.last(),
                Some(&top)
            );
        }
        let creature_exiled = game.object_ref(creature);
        let land_exiled = game.object_ref(land);
        for viewer in [caster, owner] {
            let rows = game.observe(viewer).exile_permissions;
            assert_eq!(rows.len(), 2);
            for object in [creature_exiled, land_exiled] {
                let row = rows.iter().find(|row| row.object == object).unwrap();
                assert_eq!(row.player, owner);
                assert_eq!(row.expiry, ExilePlayExpiry::WhileExiled);
                assert_eq!(row.source, None);
                assert!(!row.cast_only && !row.without_paying_mana_cost);
            }
        }
        reject(&mut game, Action::AnnounceExile { card: creature });
        reject(&mut game, Action::PlayLand { card: land });

        next_main(&mut game);
        let cast_action = Action::AnnounceExile { card: creature };
        reject(&mut game, cast_action.clone()); // No mana remains from the previous turn.
        let play_land = Action::PlayLand { card: land };
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == play_land)
            .unwrap();
        let checkpoint = game.snapshot();
        let mut indexed = game.clone();
        act(&mut game, play_land.clone());
        indexed.apply_index(owner, revision, index).unwrap();
        assert_eq!(game.snapshot(), indexed.snapshot());
        assert_eq!(game.object_view(land).zone, Zone::Battlefield);
        assert!(!game.object_view(land).tapped);
        indexed.restore(&checkpoint).unwrap();
        assert!(indexed.legal_actions().contains(&play_land));

        let mut stale = game.clone();
        stale.move_object(creature, Zone::Graveyard);
        stale.move_object(creature, Zone::Exile);
        stale.state.players[owner.index()].mana[4] = 2;
        assert_ne!(stale.object_ref(creature), creature_exiled);
        reject(&mut stale, cast_action.clone());

        game.state.players[owner.index()].mana[4] = 2;
        act(&mut game, cast_action);
        pay(&mut game);
        assert_eq!(game.state.players[owner.index()].mana[4], 0);
        settle(&mut game);
        assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
        assert_eq!(game.object_view(creature).controller, owner);
        assert!(game.observe(owner).exile_permissions.is_empty());
    }
}

#[test]
fn exile_casts_and_simultaneous_exile_returns_share_the_once_per_turn_limit() {
    let _rules = fixture_rules::activate(&[
        (fixture_card("Phyrexian Arena"), &EXILE_ENTRY_OBSERVER),
        (fixture_card("Kellan's Lightblades"), &BLINK_GROUP),
    ]);
    for caster in [Player::P0, Player::P1] {
        let watcher = caster.other();
        let mut game = scenario(caster);
        put(&mut game, watcher, "Phyrexian Arena", Zone::Battlefield);
        let bear = put(&mut game, caster, "Grizzly Bears", Zone::Hand);
        cast(&mut game, bear, None);
        settle(&mut game);
        assert_eq!(game.hand_count(watcher), 0);

        let adventure = put(&mut game, caster, "Hearth Elemental", Zone::Hand);
        act(&mut game, Action::AnnounceAdventure { card: adventure });
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(adventure).zone, Zone::Exile);
        assert_eq!(game.hand_count(watcher), 0);
        act(&mut game, Action::AnnounceExile { card: adventure });
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(adventure).zone, Zone::Battlefield);
        assert_eq!(game.hand_count(watcher), 1);

        let targets = [bear, adventure].map(Target::Card);
        let blink = put(&mut game, caster, "Kellan's Lightblades", Zone::Hand);
        announce(&mut game, blink, &targets);
        settle(&mut game);
        assert_eq!(game.hand_count(watcher), 1);

        next_main(&mut game);
        let blink = put(&mut game, watcher, "Kellan's Lightblades", Zone::Hand);
        game.state.players[watcher.index()].mana = [8; 6];
        let before = [bear, adventure].map(|id| game.object_ref(id));
        let mut suppressed = game.clone();
        put(&mut suppressed, caster, "Torpor Orb", Zone::Battlefield);
        // Both targets return regardless of entry suppression. The normal
        // event queues one draw for the whole batch, not one per creature.
        for (branch, drawn) in [(&mut game, 1), (&mut suppressed, 0)] {
            let hand = branch.hand_count(watcher);
            announce(branch, blink, &targets);
            settle(branch);
            assert_eq!(branch.hand_count(watcher), hand - 1 + drawn);
            for old in before {
                assert_eq!(branch.object_view(old.card).zone, Zone::Battlefield);
                assert_ne!(branch.object_ref(old.card), old);
            }
        }
    }
}
