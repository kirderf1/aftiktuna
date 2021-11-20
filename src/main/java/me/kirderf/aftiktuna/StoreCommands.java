package me.kirderf.aftiktuna;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.builder.LiteralArgumentBuilder;
import com.mojang.brigadier.builder.RequiredArgumentBuilder;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.action.InputActionContext;
import me.kirderf.aftiktuna.object.ObjectArgument;
import me.kirderf.aftiktuna.object.ObjectType;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.entity.Shopkeeper;

import java.util.Optional;

public final class StoreCommands {
	
	private static final CommandDispatcher<StoreContext> DISPATCHER = new CommandDispatcher<>();
	
	static {
		DISPATCHER.register(LiteralArgumentBuilder.<StoreContext>literal("buy")
				.then(RequiredArgumentBuilder.<StoreContext, ObjectType>argument("item", ObjectArgument.create(ObjectTypes.ITEMS))
						.executes(context -> buyItem(context.getSource().inputContext, context.getSource().shopkeeper, ObjectArgument.getType(context, "item")))));
		DISPATCHER.register(LiteralArgumentBuilder.<StoreContext>literal("exit").executes(context -> exit(context.getSource().inputContext)));
		DISPATCHER.register(LiteralArgumentBuilder.<StoreContext>literal("help").executes(context -> printCommands(context.getSource())));
	}
	
	public static int handleInput(String input, StoreContext context) throws CommandSyntaxException {
		return DISPATCHER.execute(input, context);
	}
	
	public static record StoreContext(InputActionContext inputContext, Shopkeeper shopkeeper) {}
	
	private static int printCommands(StoreContext context) {
		return context.inputContext().noAction(out -> {
			out.printf("Commands:%n");
			
			for (String command : DISPATCHER.getSmartUsage(DISPATCHER.getRoot(), context).values()) {
				out.printf("> %s%n", command);
			}
			out.println();
		});
	}
	
	private static int exit(InputActionContext context) {
		return context.noAction(out -> {
			context.getGame().restoreViewAndPrintArea();
			
			out.printf("%s stops trading with the shopkeeper.%n", context.getControlledAftik().getName());
		});
	}
	
	private static int buyItem(InputActionContext context, Shopkeeper shopkeeper, ObjectType item) {
		Aftik aftik = context.getControlledAftik();
		
		if (item == ObjectTypes.FUEL_CAN) {
			return context.action(out -> {
				Optional<ObjectType> optionalItem = shopkeeper.buyItem(aftik.getCrew());
				optionalItem.ifPresentOrElse(_item -> {
					aftik.addItem(item);
					out.print("%s bought a %s.", aftik.getName(), item.name());
				}, () -> out.print("%s does not have enough points to buy a fuel can.", aftik.getName()));
			});
		} else {
			return context.printNoAction("A %s is not in stock.%n", item.name());
		}
	}
}