package me.kirderf.aftiktuna.action;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.builder.LiteralArgumentBuilder;
import com.mojang.brigadier.builder.RequiredArgumentBuilder;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.object.ItemType;
import me.kirderf.aftiktuna.object.ObjectArgument;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.entity.Shopkeeper;

public final class StoreCommands {
	
	private static final CommandDispatcher<StoreContext> DISPATCHER = new CommandDispatcher<>();
	
	static {
		DISPATCHER.register(LiteralArgumentBuilder.<StoreContext>literal("buy")
				.then(RequiredArgumentBuilder.<StoreContext, ItemType>argument("item", ObjectArgument.create(ObjectTypes.ITEMS))
						.executes(context -> buyItem(context.getSource().inputContext, context.getSource().shopkeeper, ObjectArgument.getType(context, "item", ItemType.class)))));
		DISPATCHER.register(LiteralArgumentBuilder.<StoreContext>literal("exit").executes(context -> exit(context.getSource().inputContext)));
		DISPATCHER.register(LiteralArgumentBuilder.<StoreContext>literal("help").executes(context -> printCommands(context.getSource())));
		DISPATCHER.register(LiteralArgumentBuilder.<StoreContext>literal("status").executes(context -> ActionHandler.printStatus(context.getSource().inputContext())));
	}
	
	public static int handleInput(String input, StoreContext context) throws CommandSyntaxException {
		return DISPATCHER.execute(input, context);
	}
	
	public static record StoreContext(InputActionContext inputContext, Shopkeeper shopkeeper) {}
	
	private static int printCommands(StoreContext context) {
		return context.inputContext().noAction(out -> {
			out.print("Commands:");
			
			for (String command : DISPATCHER.getSmartUsage(DISPATCHER.getRoot(), context).values()) {
				out.print("> %s", command);
			}
			out.println();
		});
	}
	
	private static int exit(InputActionContext context) {
		return context.noActionWithView(out -> {
			context.getGame().restoreView();
			
			out.print("%s stops trading with the shopkeeper.", context.getControlledAftik().getName());
		});
	}
	
	private static int buyItem(InputActionContext context, Shopkeeper shopkeeper, ItemType item) {
		Aftik aftik = context.getControlledAftik();
		
		if (shopkeeper.getItemsInStock().contains(item)) {
			return context.action(out -> {
				boolean success = shopkeeper.buyItem(aftik.getCrew(), item);
				if (success) {
					aftik.addItem(item);
					out.print("%s bought a %s.", aftik.getName(), item.name());
				} else
					out.print("%s does not have enough points to buy a %s.", aftik.getName(), item.name());
			});
		} else {
			return context.printNoAction("A %s is not in stock.", item.name());
		}
	}
}