# Hungering Puppetbeast: resolution-time keyword choice

The card remains unsupported. Its enter trigger creates one Heartwood. Its
activated ability costs {1} and sacrifice of another controlled artifact;
the cost must reject itself even though it is an artifact. On resolution put
a +1/+1 counter on the source, then its controller chooses trample, hexproof,
or haste to grant it until end of turn. The keyword choice is not a mode
chosen when the ability is announced.

The isolated draft includes an owned resolution choice intended to retain the exact source incarnation
and controller, expose only the three legal choices to the proper actor, and
resume the same instruction stream after the decision. If the source has left
or changed incarnation before that instruction, no permanent remains to gain
the ability and there is no keyword choice. Counter placement replacements
may create their own choices before the keyword choice. The effect must work
with independent concurrent activations and snapshot/branch/restore at either
choice; cleanup removes the selected temporary keyword but not the counter.

Before admission, compile and run the draft tests for both players, each keyword, no eligible sacrifice,
self/opponent artifact rejection, mana payment, source departure/reentry,
preexisting counters and replacement effects, and Heartwood token doubling.
The provenance records the pinned executable XMage sources, but does not
establish that the draft behavior is correct.
