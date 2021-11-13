package me.kirderf.aftiktuna.action;

import com.mojang.brigadier.CommandDispatcher;
import me.kirderf.aftiktuna.ContextPrinter;
import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.object.ObjectArgument;
import me.kirderf.aftiktuna.level.object.ObjectType;
import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.door.Door;
import me.kirderf.aftiktuna.level.object.door.DoorPair;
import me.kirderf.aftiktuna.level.object.entity.Aftik;

import java.util.Optional;
import java.util.function.Consumer;

import static me.kirderf.aftiktuna.action.ActionHandler.argument;
import static me.kirderf.aftiktuna.action.ActionHandler.literal;

public final class DoorActions {
	static void register(CommandDispatcher<GameInstance> dispatcher) {
		dispatcher.register(literal("enter").then(argument("door", ObjectArgument.create(ObjectTypes.DOORS))
				.executes(context -> enterDoor(context.getSource(), ObjectArgument.getType(context, "door")))));
		dispatcher.register(literal("force").then(argument("door", ObjectArgument.create(ObjectTypes.DOORS))
				.executes(context -> forceDoor(context.getSource(), ObjectArgument.getType(context, "door")))));
		dispatcher.register(literal("enter").then(literal("ship")
				.executes(context -> enterDoor(context.getSource(), ObjectTypes.SHIP_ENTRANCE))));
		dispatcher.register(literal("exit").then(literal("ship")
				.executes(context -> enterDoor(context.getSource(), ObjectTypes.SHIP_EXIT))));
	}
	
	private static int enterDoor(GameInstance game, ObjectType doorType) {
		Aftik aftik = game.getAftik();
		return searchForAndIfNotBlocked(game, aftik, doorType,
				door -> aftik.moveEnterMain(door, game.out()),
				() -> game.directOut().println("There is no such door here to go through."));
	}
	
	private static int forceDoor(GameInstance game, ObjectType doorType) {
		Aftik aftik = game.getAftik();
		return searchForAndIfNotBlocked(game, aftik, doorType,
				door -> aftik.moveAndForce(door, game.out()),
				() -> game.directOut().println("There is no such door here."));
	}
	
	private static int searchForAndIfNotBlocked(GameInstance game, Aftik aftik, ObjectType type, Consumer<Door> onSuccess, Runnable onNoMatch) {
		Optional<Door> optionalDoor = aftik.findNearest(Door.CAST.filter(type::matching));
		if (optionalDoor.isPresent()) {
			Door door = optionalDoor.get();
			
			Optional<GameObject> blocking = aftik.findBlockingTo(door.getCoord());
			if (blocking.isEmpty()) {
				onSuccess.accept(door);
				return 1;
			} else {
				ActionHandler.printBlocking(game, blocking.get());
				return 0;
			}
		} else {
			onNoMatch.run();
			return 0;
		}
	}
	
	public static void printEnterResult(ContextPrinter out, Aftik aftik, EnterResult result) {
		result.either().run(success -> printEnterSuccess(out, aftik, success),
				failureType -> out.printFor(aftik, "The door is %s.%n", failureType.adjective()));
	}
	
	private static void printEnterSuccess(ContextPrinter out, Aftik aftik, EnterResult.Success result) {
		result.usedItem().ifPresentOrElse(
				item -> out.printFor(aftik, "Using their %s, %s entered the door into a new room.%n", item.name(), aftik.getName()),
				() -> out.printFor(aftik, "%s entered the door into a new room.%n", aftik.getName()));
	}
	
	public static void printForceResult(ContextPrinter out, Aftik aftik, ForceResult result) {
		result.propertyResult().either().run(success -> printForceSuccess(out, aftik, result.pair(), success), status -> printForceStatus(out, aftik, status));
	}
	
	private static void printForceSuccess(ContextPrinter out, Aftik aftik, DoorPair pair, ForceResult.Success result) {
		out.printAt(pair.targetDoor(), "%s used their %s and %s the door.%n", aftik.getName(), result.item().name(), result.method().text());
		out.printAt(pair.destination(), "A door was %s from the other side.%n", result.method().text());
	}
	
	private static void printForceStatus(ContextPrinter out, Aftik aftik, ForceResult.Status status) {
		switch(status) {
			case NOT_STUCK -> out.printFor(aftik, "The door does not seem to be stuck.%n");
			case NEED_TOOL -> out.printFor(aftik, "%s need some sort of tool to force the door open.%n", aftik.getName());
			case NEED_BREAK_TOOL -> out.printFor(aftik, "%s need some sort of tool to break the door open.%n", aftik.getName());
		}
	}
}