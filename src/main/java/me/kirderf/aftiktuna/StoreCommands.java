package me.kirderf.aftiktuna;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.builder.LiteralArgumentBuilder;
import com.mojang.brigadier.builder.RequiredArgumentBuilder;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.object.ObjectArgument;
import me.kirderf.aftiktuna.object.ObjectType;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.entity.Shopkeeper;
import me.kirderf.aftiktuna.print.ActionPrinter;

import java.io.PrintWriter;
import java.util.Optional;

public final class StoreCommands {
	
	private static final CommandDispatcher<StoreContext> DISPATCHER = new CommandDispatcher<>();
	
	static {
		DISPATCHER.register(LiteralArgumentBuilder.<StoreContext>literal("buy")
				.then(RequiredArgumentBuilder.<StoreContext, ObjectType>argument("item", ObjectArgument.create(ObjectTypes.ITEMS))
						.executes(context -> buyItem(context.getSource(), ObjectArgument.getType(context, "item")))));
		DISPATCHER.register(LiteralArgumentBuilder.<StoreContext>literal("exit").executes(context -> exit(context.getSource())));
		DISPATCHER.register(LiteralArgumentBuilder.<StoreContext>literal("help").executes(context -> printCommands(context.getSource())));
	}
	
	public static int handleInput(String input, StoreContext context) {
		try {
			return DISPATCHER.execute(input, context);
		} catch(CommandSyntaxException e) {
			context.out.printf("Unexpected input \"%s\"%n", input);
			return 0;
		}
	}
	
	public static record StoreContext(GameInstance game, Shopkeeper shopkeeper, PrintWriter out, ActionPrinter actionOut) {}
	
	private static int printCommands(StoreContext context) {
		context.out.printf("Commands:%n");
		
		for (String command : DISPATCHER.getSmartUsage(DISPATCHER.getRoot(), context).values()) {
			context.out.printf("> %s%n", command);
		}
		context.out.println();
		return 0;
	}
	
	private static int exit(StoreContext context) {
		context.game.restoreView();
		context.actionOut.print("%s stops trading with the shopkeeper.", context.game.getCrew().getAftik().getName());
		
		return 1;
	}
	
	private static int buyItem(StoreContext context, ObjectType item) {
		Aftik aftik = context.game.getCrew().getAftik();
		
		if (item == ObjectTypes.FUEL_CAN) {
			Optional<ObjectType> optionalItem = context.shopkeeper.buyItem(aftik.getCrew());
			optionalItem.ifPresentOrElse(_item -> {
				aftik.addItem(item);
				context.actionOut.print("%s bought a %s.", aftik.getName(), item.name());
			}, () -> context.actionOut.print("%s does not have enough points to buy a fuel can.", aftik.getName()));
			return 1;
		} else {
			context.out.printf("A %s is not in stock.%n", item.name());
			return 0;
		}
	}
}