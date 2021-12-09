package me.kirderf.aftiktuna.action;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.ArgumentType;
import com.mojang.brigadier.arguments.StringArgumentType;
import com.mojang.brigadier.builder.LiteralArgumentBuilder;
import com.mojang.brigadier.builder.RequiredArgumentBuilder;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.location.Position;
import me.kirderf.aftiktuna.location.Ship;
import me.kirderf.aftiktuna.object.ObjectArgument;
import me.kirderf.aftiktuna.object.ObjectType;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.entity.Creature;
import me.kirderf.aftiktuna.object.entity.Entity;
import me.kirderf.aftiktuna.print.ActionPrinter;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.Optional;
import java.util.function.IntSupplier;
import java.util.function.ToIntFunction;
import java.util.stream.Collectors;

public final class ActionHandler {
	static final CommandDispatcher<InputActionContext> DISPATCHER = new CommandDispatcher<>();
	
	static {
		ItemActions.register();
		DoorActions.register();
		NPCActions.register();
		DISPATCHER.register(literal("attack")
				.executes(context -> attack(context.getSource()))
				.then(argument("creature", ObjectArgument.create(ObjectTypes.CREATURES))
						.executes(context -> attack(context.getSource(), ObjectArgument.getType(context, "creature")))));
		DISPATCHER.register(literal("launch").then(literal("ship").executes(context -> launchShip(context.getSource()))));
		DISPATCHER.register(literal("control").then(argument("name", StringArgumentType.string())
				.executes(context -> controlAftik(context.getSource(), StringArgumentType.getString(context, "name")))));
		DISPATCHER.register(literal("wait").executes(context -> context.getSource().action()));
		DISPATCHER.register(literal("status").executes(context -> printStatus(context.getSource())));
		DISPATCHER.register(literal("help").executes(context -> printCommands(context.getSource())));
	}
	
	static LiteralArgumentBuilder<InputActionContext> literal(String str) {
		return LiteralArgumentBuilder.literal(str);
	}
	
	static <T> RequiredArgumentBuilder<InputActionContext, T> argument(String name, ArgumentType<T> argumentType) {
		return RequiredArgumentBuilder.argument(name, argumentType);
	}
	
	public static int handleInput(InputActionContext context, String input) throws CommandSyntaxException {
		return DISPATCHER.execute(input, context);
	}
	
	static int printStatus(InputActionContext context) {
		return context.noAction(out -> context.getGame().getStatusPrinter().printCrewStatus());
	}
	
	private static int printCommands(InputActionContext context) {
		return context.noAction(out -> {
			out.print("Commands:");
			
			for (String command : DISPATCHER.getSmartUsage(DISPATCHER.getRoot(), context).values()) {
				out.print("> %s", command);
			}
			out.println();
		});
	}
	
	private static int attack(InputActionContext context) {
		Aftik aftik = context.getControlledAftik();
		
		return searchForAccessible(context, aftik, Creature.CAST, false,
				creature -> context.action(out -> aftik.moveAndAttack(creature, out)),
				() -> context.printNoAction("There is no such creature to attack."));
	}
	
	private static int attack(InputActionContext context, ObjectType creatureType) {
		Aftik aftik = context.getControlledAftik();
		
		return searchForAccessible(context, aftik, Creature.CAST.filter(creatureType::matching), false,
				creature -> context.action(out -> aftik.moveAndAttack(creature, out)),
				() -> context.printNoAction("There is no such creature to attack."));
	}
	
	static int launchShip(InputActionContext context) {
		Aftik aftik = context.getControlledAftik();
		
		if (aftik.hasItem(ObjectTypes.FUEL_CAN)) {
			if (isNearShip(aftik, context.getCrew().getShip())) {
				
				return context.action(aftik.getMind()::setLaunchShip);
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
	
	private static int controlAftik(InputActionContext context, String name) {
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
	
	static <T extends GameObject> int searchForAccessible(InputActionContext context, Aftik aftik,
														  OptionalFunction<GameObject, T> mapper, boolean exactPos,
														  ToIntFunction<T> onSuccess, IntSupplier onNoMatch) {
		Optional<T> optionalDoor = aftik.findNearest(mapper, exactPos);
		if (optionalDoor.isPresent()) {
			T object = optionalDoor.get();
			Position pos = exactPos ? object.getPosition()
					: object.getPosition().getPosTowards(aftik.getCoord());
			
			return ifAccessible(context, aftik, pos, () -> onSuccess.applyAsInt(object));
		} else {
			return onNoMatch.getAsInt();
		}
	}
	
	static int ifAccessible(InputActionContext context, Aftik aftik, Position pos, IntSupplier onSuccess) {
		Optional<GameObject> blocking = aftik.findBlockingTo(pos.coord());
		if (blocking.isEmpty()) {
			return onSuccess.getAsInt();
		} else {
			return context.printNoAction(createBlockingMessage(blocking.get()));
		}
	}
	
	public static String createBlockingMessage(GameObject blocking) {
		return "%s is blocking the way.".formatted(blocking.getDisplayName(true, true));
	}
	
	public static void handleEntities(GameInstance game, ActionPrinter out) {
		
		for (Entity entity : game.getGameObjectStream().flatMap(Entity.CAST.toStream()).collect(Collectors.toList())) {
			if (entity.isAlive() && entity != game.getCrew().getAftik()) {
				entity.performAction(out);
			}
		}
	}
	
	public static String condition(String text, boolean b) {
		if (b)
			return text.replaceAll("[\\[\\]]", "");
		else return text.replaceAll("\\[.*]", "");
	}
}
