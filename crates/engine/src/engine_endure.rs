//! Exact-source resolving endure choices; the continuation is part of Pending.
use super::*;
use crate::cards::EndureRecipient;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// CR701.63: the enduring permanent's controller makes this choice. The
    /// packet already freezes its actual battlefield controller on departure.
    pub(super) fn start_fixed_endure(
        &mut self,
        item: &SpellView,
        amount: u16,
        token: Card,
    ) -> bool {
        if amount == 0 {
            return false;
        }
        let source = item.source.expect("endure requires an object source");
        let live = self.on_battlefield(source.id) && self.object_ref(source.id) == source.object;
        let player = if live {
            self.state.objects[source.id as usize].controller
        } else {
            source.facts.controller
        };
        if live && !replacements::counters_prohibited(self, source.object) {
            self.state.pending = Pending::Endure {
                player,
                source: source.object,
                amount: u128::from(amount),
                token,
                sampled_size: None,
            };
            return true;
        }
        // A prohibited placement cannot be chosen instead of creating the
        // Spirit. Nor may a departed source bind a later incarnation.
        self.create_tokens(player, token, 1, false);
        false
    }

    pub(super) fn create_sized_token(&mut self, player: Player, token: Card, size: u128) {
        let _ = i128::try_from(size).capacity("token base size exhausted");
        let recipe = CopyRecipe::new(token).with_sampled_size(copies::SampledTokenSize::new(size));
        self.stage_copy_creation(player, recipe, 1);
    }

    pub(super) fn start_scaled_endure(
        &mut self,
        item: &SpellView,
        recipient: EndureRecipient,
        amount: u128,
        token: Card,
    ) -> bool {
        if amount == 0 {
            return false;
        }
        let _ = i128::try_from(amount).capacity("endure token size exhausted");
        let (source, previous_controller) = match recipient {
            EndureRecipient::Source => {
                let source = item.source.expect("endure requires an object source");
                (source.object, source.facts.controller)
            }
            EndureRecipient::Entering => {
                let event = item
                    .event_object
                    .as_ref()
                    .expect("endure event controller facts");
                let EventObjectFacts::Controller(controller) = event.facts else {
                    unreachable!("endure event controller facts");
                };
                (event.object, controller)
            }
        };
        let live = self.on_battlefield(source.card) && self.object_ref(source.card) == source;
        let player = if live {
            self.state.objects[source.card as usize].controller
        } else {
            previous_controller
        };
        if live && !replacements::counters_prohibited(self, source) {
            self.state.pending = Pending::Endure {
                player,
                source,
                amount,
                token,
                sampled_size: Some(copies::SampledTokenSize::new(amount)),
            };
            return true;
        }
        self.create_sized_token(player, token, amount);
        false
    }

    pub(super) fn choose_endure(&mut self, put_counters: bool) {
        let Pending::Endure {
            player,
            source,
            amount,
            token,
            sampled_size,
        } = self.state.pending
        else {
            unreachable!()
        };
        // No action other than the actor's choice or concession can interleave
        // while resolution is suspended. The source was validated at creation.
        assert!(self.on_battlefield(source.card) && self.object_ref(source.card) == source);
        if put_counters {
            self.effect_counter_by(
                source.card,
                CounterKind::PlusOne,
                u64::try_from(amount).capacity("endure counter quantity exhausted"),
                player,
                replacements::ResumeMode::Program,
            );
        } else if let Some(size) = sampled_size {
            self.create_sized_token(player, token, size.value());
        } else {
            self.create_tokens(player, token, 1, false);
        }
        self.run_program();
    }
}
