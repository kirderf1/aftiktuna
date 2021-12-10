package me.kirderf.aftiktuna.command.game;

import me.kirderf.aftiktuna.command.CommandContext;
import me.kirderf.aftiktuna.command.CommandUtil;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.entity.AftikNPC;
import me.kirderf.aftiktuna.object.entity.Shopkeeper;
import me.kirderf.aftiktuna.print.ActionPrinter;

public final class NPCCommands {
	static void register() {
		GameCommands.DISPATCHER.register(GameCommands.literal("recruit").then(GameCommands.literal("aftik").executes(context -> recruitAftik(context.getSource()))));
		GameCommands.DISPATCHER.register(GameCommands.literal("trade").executes(context -> trade(context.getSource())));
	}
	
	private static int recruitAftik(CommandContext context) {
		Aftik aftik = context.getControlledAftik();
		
		if (context.getCrew().hasCapacity()) {
			return CommandUtil.searchForAccessible(context, aftik, AftikNPC.CAST.filter(ObjectTypes.AFTIK::matching), false,
					npc -> context.action(out -> recruitAftik(context, aftik, npc, out)),
					() -> context.printNoAction("There is no aftik here to recruit."));
		} else {
			return context.printNoAction("There is not enough room for another crew member.");
		}
	}
	
	private static void recruitAftik(CommandContext context, Aftik aftik, AftikNPC npc, ActionPrinter out) {
		boolean success = aftik.tryMoveNextTo(npc.getPosition(), out);
		
		if (success) {
			context.getCrew().addCrewMember(npc, out);
		}
	}
	
	private static int trade(CommandContext context) {
		Aftik aftik = context.getControlledAftik();
		
		return CommandUtil.searchForAccessible(context, aftik, Shopkeeper.CAST, false,
				shopkeeper -> context.action(out -> trade(context, aftik, shopkeeper, out)),
				() -> context.printNoAction("There is no shopkeeper here to trade with."));
	}
	
	private static void trade(CommandContext context, Aftik aftik, Shopkeeper shopkeeper, ActionPrinter out) {
		boolean success = aftik.tryMoveNextTo(shopkeeper.getPosition(), out);
		if (success) {
			context.getGame().setStoreView(shopkeeper);
			
			out.print("%s starts trading with the shopkeeper.", aftik.getName());
		}
	}
}