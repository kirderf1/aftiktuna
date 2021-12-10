package me.kirderf.aftiktuna.command.game;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.ArgumentType;
import com.mojang.brigadier.arguments.StringArgumentType;
import com.mojang.brigadier.builder.LiteralArgumentBuilder;
import com.mojang.brigadier.builder.RequiredArgumentBuilder;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.command.CommandContext;
import me.kirderf.aftiktuna.command.CommandUtil;
import me.kirderf.aftiktuna.location.Ship;
import me.kirderf.aftiktuna.object.ObjectArgument;
import me.kirderf.aftiktuna.object.ObjectType;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.entity.Creature;
import me.kirderf.aftiktuna.object.entity.ai.LaunchShipTask;
import me.kirderf.aftiktuna.object.entity.ai.RestTask;

import java.util.Optional;

public final class GameCommands {
	static final CommandDispatcher<CommandContext> DISPATCHER = new CommandDispatcher<>();
	
	static {
		ItemCommands.register();
		DoorCommands.register();
		NPCCommands.register();
		DISPATCHER.register(literal("attack")
				.executes(context -> attack(context.getSource()))
				.then(argument("creature", ObjectArgument.create(ObjectTypes.CREATURES))
						.executes(context -> attack(context.getSource(), ObjectArgument.getType(context, "creature")))));
		DISPATCHER.register(literal("launch").then(literal("ship").executes(context -> launchShip(context.getSource()))));
		DISPATCHER.register(literal("control").then(argument("name", StringArgumentType.string())
				.executes(context -> controlAftik(context.getSource(), StringArgumentType.getString(context, "name")))));
		DISPATCHER.register(literal("wait").executes(context -> context.getSource().action()));
		DISPATCHER.register(literal("rest").executes(context -> rest(context.getSource())));
		DISPATCHER.register(literal("status").executes(context -> printStatus(context.getSource())));
		DISPATCHER.register(literal("help").executes(context -> printCommands(context.getSource())));
	}
	
	static LiteralArgumentBuilder<CommandContext> literal(String str) {
		return LiteralArgumentBuilder.literal(str);
	}
	
	static <T> RequiredArgumentBuilder<CommandContext, T> argument(String name, ArgumentType<T> argumentType) {
		return RequiredArgumentBuilder.argument(name, argumentType);
	}
	
	public static int handleInput(CommandContext context, String input) throws CommandSyntaxException {
		return DISPATCHER.execute(input, context);
	}
	
	public static int printStatus(CommandContext context) {
		return context.noAction(out -> context.getGame().getStatusPrinter().printCrewStatus());
	}
	
	private static int printCommands(CommandContext context) {
		return context.noAction(out -> {
			out.print("Commands:");
			
			for (String command : DISPATCHER.getSmartUsage(DISPATCHER.getRoot(), context).values()) {
				out.print("> %s", command);
			}
			out.println();
		});
	}
	
	private static int attack(CommandContext context) {
		Aftik aftik = context.getControlledAftik();
		
		return CommandUtil.searchForAccessible(context, aftik, Creature.CAST, false,
				creature -> context.action(out -> aftik.moveAndAttack(creature, out)),
				() -> context.printNoAction("There is no such creature to attack."));
	}
	
	private static int attack(CommandContext context, ObjectType creatureType) {
		Aftik aftik = context.getControlledAftik();
		
		return CommandUtil.searchForAccessible(context, aftik, Creature.CAST.filter(creatureType::matching), false,
				creature -> context.action(out -> aftik.moveAndAttack(creature, out)),
				() -> context.printNoAction("There is no such creature to attack."));
	}
	
	static int launchShip(CommandContext context) {
		Aftik aftik = context.getControlledAftik();
		
		if (aftik.hasItem(ObjectTypes.FUEL_CAN)) {
			if (isNearShip(aftik, context.getCrew().getShip())) {
				
				return context.action(out -> {
					aftik.getMind().setAndPerformPlayerTask(new LaunchShipTask(context.getCrew().getShip()), out);
				});
			} else {
				return context.printNoAction("%s need to be near the ship in order to launch it.", aftik.getName());
			}
		} else {
			return context.printNoAction("%s need a fuel can to launch the ship.", aftik.getName());
		}
	}
	
	private static boolean isNearShip(Aftik aftik, Ship ship) {
		return aftik.getArea() == ship.getRoom() || aftik.isAnyNear(ObjectTypes.SHIP_ENTRANCE::matching);
	}
	
	private static int controlAftik(CommandContext context, String name) {
		Optional<Aftik> aftikOptional = context.getCrew().findByName(name);
		if (aftikOptional.isPresent()) {
			Aftik aftik = aftikOptional.get();
			if (aftik != context.getControlledAftik()) {
				return context.noActionWithView(out -> context.getCrew().setControllingAftik(aftik, out));
			} else {
				return context.printNoAction("You're already in control of them.");
			}
		} else {
			return context.printNoAction("There is no crew member by that name.");
		}
	}
	
	private static int rest(CommandContext context) {
		Aftik aftik = context.getControlledAftik();
		
		if (RestTask.isAreaSafe(aftik)) {
			if (RestTask.isAllRested(aftik)) {
				return context.printNoAction("All crew in the area is already rested.");
			} else {
				return context.action(out -> {
					aftik.getMind().setAndPerformPlayerTask(new RestTask(), out);
					out.print("%s takes some time to rest up.", aftik.getName());
				});
			}
		} else {
			return context.printNoAction("This area is not safe to rest in.");
		}
	}
}