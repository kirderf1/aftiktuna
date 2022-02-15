package me.kirderf.aftiktuna.command.store;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.ArgumentType;
import com.mojang.brigadier.arguments.IntegerArgumentType;
import com.mojang.brigadier.builder.LiteralArgumentBuilder;
import com.mojang.brigadier.builder.RequiredArgumentBuilder;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.command.CommandContext;
import me.kirderf.aftiktuna.command.game.GameCommands;
import me.kirderf.aftiktuna.object.ObjectArgument;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.entity.Shopkeeper;
import me.kirderf.aftiktuna.object.type.ItemType;
import me.kirderf.aftiktuna.object.type.ObjectTypes;
import me.kirderf.aftiktuna.print.ActionPrinter;

public final class StoreCommands {
	
	private static final CommandDispatcher<StoreContext> DISPATCHER = new CommandDispatcher<>();
	
	static {
		DISPATCHER.register(literal("buy")
				.then(argument("item", ObjectArgument.create(ObjectTypes.ITEMS))
						.executes(context -> buyItem(context.getSource().inputContext, context.getSource().shopkeeper,
								1, ObjectArgument.getType(context, "item", ItemType.class))))
				.then(argument("count", IntegerArgumentType.integer(1))
						.then(argument("item", ObjectArgument.create(ObjectTypes.ITEMS))
								.executes(context -> buyItem(context.getSource().inputContext, context.getSource().shopkeeper,
										IntegerArgumentType.getInteger(context, "count"),
										ObjectArgument.getType(context, "item", ItemType.class))))));
		DISPATCHER.register(literal("sell")
				.then(argument("item", ObjectArgument.create(ObjectTypes.ITEMS))
						.executes(context -> sellItem(context.getSource().inputContext(), 1, ObjectArgument.getType(context, "item", ItemType.class))))
				.then(argument("count", IntegerArgumentType.integer(1))
						.then(argument("item", ObjectArgument.create(ObjectTypes.ITEMS))
								.executes(context -> sellItem(context.getSource().inputContext(),
										IntegerArgumentType.getInteger(context, "count"),
										ObjectArgument.getType(context, "item", ItemType.class)))))
				.then(literal("all")
						.then(argument("item", ObjectArgument.create(ObjectTypes.ITEMS))
								.executes(context -> sellAll(context.getSource().inputContext(),
										ObjectArgument.getType(context, "item", ItemType.class))))));
		DISPATCHER.register(literal("exit").executes(context -> exit(context.getSource().inputContext)));
		DISPATCHER.register(literal("help").executes(context -> printCommands(context.getSource())));
		DISPATCHER.register(literal("status").executes(context -> GameCommands.printStatus(context.getSource().inputContext())));
		DISPATCHER.register(literal("examine")
				.then(argument("item", ObjectArgument.create(ObjectTypes.ITEMS))
						.executes(context -> examineItem(context.getSource().inputContext, context.getSource().shopkeeper, ObjectArgument.getType(context, "item", ItemType.class)))));
	}
	
	static LiteralArgumentBuilder<StoreContext> literal(String str) {
		return LiteralArgumentBuilder.literal(str);
	}
	
	static <T> RequiredArgumentBuilder<StoreContext, T> argument(String name, ArgumentType<T> argumentType) {
		return RequiredArgumentBuilder.argument(name, argumentType);
	}
	
	public static int handleInput(String input, StoreContext context) throws CommandSyntaxException {
		return DISPATCHER.execute(input, context);
	}
	
	public record StoreContext(CommandContext inputContext, Shopkeeper shopkeeper) {}
	
	private static int printCommands(StoreContext context) {
		return context.inputContext().noAction(out -> {
			out.print("Commands:");
			
			for (String command : DISPATCHER.getSmartUsage(DISPATCHER.getRoot(), context).values()) {
				out.print("> %s", command);
			}
			out.println();
		});
	}
	
	private static int exit(CommandContext context) {
		return context.noActionWithView(out -> {
			context.getGame().restoreView();
			
			out.print("%s stops trading with the shopkeeper.", context.getControlledAftik().getName());
		});
	}
	
	private static int buyItem(CommandContext context, Shopkeeper shopkeeper, int count, ItemType item) {
		Aftik aftik = context.getControlledAftik();
		
		if (shopkeeper.getItemsInStock().contains(item)) {
			return context.action(out -> buyItems(shopkeeper, count, item, aftik, out));
		} else {
			return context.printNoAction("There are no %s in stock.", item.pluralName());
		}
	}
	
	private static void buyItems(Shopkeeper shopkeeper, int count, ItemType item, Aftik aftik, ActionPrinter out) {
		boolean success = shopkeeper.buyItem(aftik.getCrew(), item, count);
		if (success) {
			for (int i = 0; i < count; i++)
				aftik.addItem(item);
			out.print("%s bought %s.", aftik.getName(), getCountAndName(count, item));
		} else
			out.print("%s does not have enough points to buy %s.", aftik.getName(), getCountAndName(count, item));
	}
	
	private static String getCountAndName(int count, ItemType item) {
		return count + " " + (count == 1 ? item.name() : item.pluralName());
	}
	
	private static int sellAll(CommandContext context, ItemType item) {
		int count = context.getControlledAftik().getItemCount(item);
		if (count > 0) {
			return sellItem(context, count, item);
		} else {
			return context.printNoAction("%s does not have a %s.", context.getControlledAftik().getName(), item.name());
		}
	}
	
	private static int sellItem(CommandContext context, int count, ItemType item) {
		Aftik aftik = context.getControlledAftik();
		
		if (aftik.getItemCount(item) >= count) {
			if (item.getPrice() >= 0) {
				return context.action(out -> sellItems(aftik, count, item, out));
			} else {
				return context.printNoAction("The %s is not sellable.", item.name());
			}
		} else {
			if (count == 1)
				return context.printNoAction("%s does not have a %s.", aftik.getName(), item.name());
			else
				return context.printNoAction("%s does not have that many %ss.", aftik.getName(), item.name());
		}
	}
	
	private static void sellItems(Aftik aftik, int count, ItemType item, ActionPrinter out) {
		int soldCount = 0;
		int points = item.getPrice() * 3 / 4;
		for (int i = 0; i < count; i++) {
			if (aftik.removeItem(item)) {
				aftik.getCrew().addPoints(points);
				soldCount++;
			}
		}
		// soldCount should match count, but due to limitations of the inventory interface, this method is used for the sake of robustness
		if (soldCount > 0) {
			if (soldCount == 1)
				out.printFor(aftik, "%s sold a %s for %dp.", aftik.getName(), item.name(), points);
			else
				out.printFor(aftik, "%s sold %d %ss for %dp.", aftik.getName(), soldCount, item.name(), soldCount*points);
		}
	}
	
	private static int examineItem(CommandContext context, Shopkeeper shopkeeper, ItemType type) {
		
		if (shopkeeper.getItemsInStock().contains(type)) {
			return context.printNoAction(type.getExamineText());
		} else {
			return context.printNoAction("There is no such item here.");
		}
	}
}