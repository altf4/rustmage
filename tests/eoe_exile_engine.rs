//! Exact exile-play permissions and private hand selections.
mod support;

use std::num::NonZeroU32;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const TAX: NonZeroU32 = NonZeroU32::new(1).unwrap();
const ANY_CARD: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const TAXED_CHOICE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::ChooseObjects {
            kind: EffectChoiceKind::ExileFromHandWithPlaySurcharge(TAX),
            players: PlayerSet::Opponent,
            filter: ANY_CARD,
            amount: 1,
            optional_skip: None,
        }],
    }],
    ..CardRules::EMPTY
};
const TARGETED_CHOICE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        &[Instruction::ChooseObjects {
            kind: EffectChoiceKind::ExileFromHand,
            players: PlayerSet::Targeted,
            filter: ANY_CARD,
            amount: 1,
            optional_skip: None,
        }],
    ),
    ..CardRules::EMPTY
};
const DRAW: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Draw(PlayerSet::Controller, 1)],
    ),
    ..CardRules::EMPTY
};
const SUBTYPE_PLAY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ExileTopPlayWhileControlSubtype("Kavu")],
    ),
    ..CardRules::EMPTY
};
const FIRST_NONLAND: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ExileUntilNonlandCastThisTurn],
    ),
    ..CardRules::EMPTY
};

fn fixture_card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn advance_until(g: &mut Game, done: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if done(g) {
            return;
        }
        choose(g, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::FinishAttackers
                    | Action::FinishBlockers
                    | Action::PutTrigger { .. }
            )
        });
    }
    panic!("advance budget exhausted at {:?}", g.state.step);
}

fn tax_cast(object: ObjectRef) -> Action {
    Action::AnnounceTaxedExile {
        object,
        face: CastFace::Normal,
        method: CastMethod::Normal,
        flash_surcharge: false,
        surcharge: TAX,
    }
}

fn select(game: &mut Game, card: CardId) {
    act(
        game,
        Action::SelectResolutionExile {
            object: game.object_ref(card),
        },
    );
}

fn add_independent_permission(game: &mut Game, player: Player) {
    let mut other = game.observe(player).exile_permissions[0];
    other.play_surcharge = None;
    game.state.exile_permissions.push(other);
}

#[test]
fn private_taxed_selection_branches_to_spell_or_tapped_land() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &TAXED_CHOICE),
        (fixture_card("Unsummon"), &DRAW),
    ]);
    for active in [Player::P0, Player::P1] {
        let opponent = active.other();
        let mut game = scenario(active);
        let source = put(&mut game, active, "Grizzly Bears", Zone::Hand);
        let unrelated = put(&mut game, active, "Mountain", Zone::Hand);
        let spell = put(&mut game, opponent, "Unsummon", Zone::Hand);
        let land = put(&mut game, opponent, "Island", Zone::Hand);
        cast(&mut game, source, None);
        advance_until(&mut game, |g| g.observe(opponent).effect_choice.is_some());
        assert_eq!(game.actor(), Some(opponent));
        let choice = game.observe(opponent).effect_choice.unwrap();
        assert_eq!(
            choice.kind,
            EffectChoiceKind::ExileFromHandWithPlaySurcharge(TAX)
        );
        assert_eq!(
            choice.candidates,
            [game.object_ref(spell), game.object_ref(land)],
        );
        let public = game.observe(active);
        assert!(public.effect_choice.is_none());
        assert!(public.objects.iter().all(|o| o.id != spell && o.id != land));
        let unrelated = game.object_ref(unrelated);
        reject(
            &mut game,
            Action::SelectResolutionExile { object: unrelated },
        );

        let action = Action::SelectResolutionExile {
            object: game.object_ref(spell),
        };
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let checkpoint = game.snapshot();
        let mut raw = game.clone();
        let mut indexed = game.clone();
        assert_eq!(raw.apply(active, action.clone()), Err(Error::WrongActor));
        assert_eq!(raw.snapshot(), checkpoint);
        act(&mut raw, action);
        indexed.apply_index(opponent, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(
            indexed.apply_index(opponent, revision, index),
            Err(Error::StaleRevision)
        );

        let exact = raw.object_ref(spell);
        for viewer in [active, opponent] {
            let rows = raw.observe(viewer).exile_permissions;
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].object, exact);
            assert_eq!(rows[0].player, opponent);
            assert_eq!(rows[0].play_surcharge, Some(TAX));
            assert_eq!(rows[0].expiry, ExilePlayExpiry::WhileExiled);
        }
        raw.move_object(source, Zone::Graveyard);
        advance_until(&mut raw, |g| {
            g.actor() == Some(opponent) && g.state.pending == Pending::Priority
        });
        raw.state.players[opponent.index()].mana = [0, 1, 0, 0, 0, 0];
        reject(&mut raw, Action::AnnounceExile { card: spell });
        reject(&mut raw, tax_cast(exact));

        let mut ordinary = raw.clone();
        add_independent_permission(&mut ordinary, opponent);
        act(&mut ordinary, Action::AnnounceExile { card: spell });
        pay(&mut ordinary);
        assert_eq!(
            ordinary
                .state
                .stack
                .last()
                .unwrap()
                .cast
                .unwrap()
                .exile_play_surcharge,
            0
        );

        let mut stale = raw.clone();
        stale.move_object(spell, Zone::Graveyard);
        stale.move_object(spell, Zone::Exile);
        stale.state.players[opponent.index()].mana = [0, 2, 0, 0, 0, 0];
        let new = stale.object_ref(spell);
        assert_ne!(new, exact);
        reject(&mut stale, tax_cast(exact));
        reject(&mut stale, tax_cast(new));

        raw.state.players[opponent.index()].mana = [0, 2, 0, 0, 0, 0];
        act(&mut raw, tax_cast(exact));
        pay(&mut raw);
        let receipt = raw.state.stack.last().unwrap().cast.unwrap();
        assert_eq!(receipt.exile_play_surcharge, 1);
        assert_eq!(receipt.payment, [0, 2, 0, 0, 0, 0]);
        let old_hand = raw.hand_count(opponent);
        settle(&mut raw);
        assert_eq!(raw.hand_count(opponent), old_hand + 1);

        select(&mut indexed, land);
        advance_until(&mut indexed, |g| {
            g.state.active == opponent
                && g.state.step == Step::Main1
                && g.actor() == Some(opponent)
                && g.state.pending == Pending::Priority
        });
        let exact = indexed.object_ref(land);
        let mut ordinary = indexed.clone();
        add_independent_permission(&mut ordinary, opponent);
        act(&mut ordinary, Action::PlayLand { card: land });
        assert!(!ordinary.object_view(land).tapped);
        reject(&mut indexed, Action::PlayLand { card: land });
        act(&mut indexed, Action::PlayTaxedExileLand { object: exact });
        assert_eq!(indexed.object_view(land).zone, Zone::Battlefield);
        assert!(indexed.object_view(land).tapped);
        assert!(indexed.state.land_played);
        assert!(indexed.observe(opponent).exile_permissions.is_empty());
    }
}

#[test]
fn targeted_opponent_chooses_from_their_own_hand() {
    let _rules = fixture_rules::activate(&[(fixture_card("Think Twice"), &TARGETED_CHOICE)]);
    let caster = Player::P0;
    let target = caster.other();
    let mut game = scenario(caster);
    let spell = put(&mut game, caster, "Think Twice", Zone::Hand);
    let own = put(&mut game, caster, "Island", Zone::Hand);
    let their = put(&mut game, target, "Mountain", Zone::Hand);
    cast(&mut game, spell, Some(Target::Player(target)));
    advance_until(&mut game, |g| g.observe(target).effect_choice.is_some());
    assert_eq!(game.actor(), Some(target));
    assert!(game.observe(caster).effect_choice.is_none());
    assert_eq!(
        game.observe(target).effect_choice.unwrap().candidates,
        [game.object_ref(their)],
    );
    let own = game.object_ref(own);
    reject(&mut game, Action::SelectResolutionExile { object: own });
    select(&mut game, their);
    assert_eq!(game.object_view(their).zone, Zone::Exile);
    assert!(game.observe(target).exile_permissions.is_empty());
}

#[test]
fn exact_subtype_play_rechecks_control_and_bruntar_permission_expires() {
    let _rules = fixture_rules::activate(&[
        (fixture_card("Think Twice"), &SUBTYPE_PLAY),
        (fixture_card("Cancel"), &FIRST_NONLAND),
        (fixture_card("Unsummon"), &DRAW),
    ]);
    let p = Player::P0;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Think Twice", Zone::Hand);
    let land = put(&mut game, p, "Island", Zone::Library);
    cast(&mut game, spell, None);
    settle(&mut game);
    let row = game.observe(p).exile_permissions[0];
    assert_eq!(row.object, game.object_ref(land));
    assert_eq!(row.requires_subtype, Some("Kavu"));
    assert_eq!(row.expiry, ExilePlayExpiry::WhileExiled);
    put(
        &mut game,
        p.other(),
        "Memorial Team Leader",
        Zone::Battlefield,
    );
    reject(&mut game, Action::PlayLand { card: land });
    let kavu = put(&mut game, p, "Memorial Team Leader", Zone::Battlefield);
    let mut later = game.clone();
    later.move_object(kavu, Zone::Graveyard);
    reject(&mut later, Action::PlayLand { card: land });
    act(&mut game, Action::PlayLand { card: land });
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert!(!game.object_view(land).tapped);

    let mut game = scenario(p);
    let spell = put(&mut game, p, "Cancel", Zone::Hand);
    let hit = put(&mut game, p, "Unsummon", Zone::Library);
    let land = put(&mut game, p, "Island", Zone::Library);
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(game.object_view(hit).zone, Zone::Exile);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    let rows = game.observe(p).exile_permissions;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].object, game.object_ref(hit));
    assert!(rows[0].cast_only);
    assert_eq!(rows[0].expiry, ExilePlayExpiry::Turn(game.state.turn));
    reject(&mut game, Action::PlayLand { card: land });
    let mut now = game.clone();
    act(&mut now, Action::AnnounceExile { card: hit });
    pay(&mut now);
    let this_turn = game.state.turn;
    advance_until(&mut game, |g| {
        g.state.turn > this_turn && g.actor() == Some(p) && g.state.pending == Pending::Priority
    });
    assert!(game.observe(p).exile_permissions.is_empty());
    reject(&mut game, Action::AnnounceExile { card: hit });
    assert_eq!(game.object_view(hit).zone, Zone::Exile);
}
