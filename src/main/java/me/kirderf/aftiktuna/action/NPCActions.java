package me.kirderf.aftiktuna.action;

import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.entity.AftikNPC;
import me.kirderf.aftiktuna.object.entity.Shopkeeper;
import me.kirderf.aftiktuna.print.ActionPrinter;

import static me.kirderf.aftiktuna.action.ActionHandler.DISPATCHER;
import static me.kirderf.aftiktuna.action.ActionHandler.literal;

public final class NPCActions {
	static void register() {
		DISPATCHER.register(literal("recruit").then(literal("aftik").executes(context -> recruitAftik(context.getSource()))));
		DISPATCHER.register(literal("trade").executes(context -> trade(context.getSource())));
	}
	
	private static int recruitAftik(InputActionContext context) {
		Aftik aftik = context.getControlledAftik();
		
		if (context.getCrew().hasCapacity()) {
			return ActionUtil.searchForAccessible(context, aftik, AftikNPC.CAST.filter(ObjectTypes.AFTIK::matching), false,
					npc -> context.action(out -> recruitAftik(context, aftik, npc, out)),
					() -> context.printNoAction("There is no aftik here to recruit."));
		} else {
			return context.printNoAction("There is not enough room for another crew member.");
		}
	}
	
	private static void recruitAftik(InputActionContext context, Aftik aftik, AftikNPC npc, ActionPrinter out) {
		boolean success = aftik.tryMoveNextTo(npc.getPosition(), out);
		
		if (success) {
			context.getCrew().addCrewMember(npc, out);
		}
	}
	
	private static int trade(InputActionContext context) {
		Aftik aftik = context.getControlledAftik();
		
		return ActionUtil.searchForAccessible(context, aftik, Shopkeeper.CAST, false,
				shopkeeper -> context.action(out -> trade(context, aftik, shopkeeper, out)),
				() -> context.printNoAction("There is no shopkeeper here to trade with."));
	}
	
	private static void trade(InputActionContext context, Aftik aftik, Shopkeeper shopkeeper, ActionPrinter out) {
		boolean success = aftik.tryMoveNextTo(shopkeeper.getPosition(), out);
		if (success) {
			context.getGame().setStoreView(shopkeeper);
			
			out.print("%s starts trading with the shopkeeper.", aftik.getName());
		}
	}
}