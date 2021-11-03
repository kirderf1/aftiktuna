package me.kirderf.aftiktuna.action;

import com.mojang.brigadier.CommandDispatcher;
import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.object.ObjectArgument;
import me.kirderf.aftiktuna.level.object.ObjectType;
import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.door.Door;
import me.kirderf.aftiktuna.level.object.door.EnterResult;
import me.kirderf.aftiktuna.level.object.door.ForceResult;
import me.kirderf.aftiktuna.level.object.entity.Aftik;
import me.kirderf.aftiktuna.level.object.entity.Entity;

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
	}
	
	private static int enterDoor(GameInstance game, ObjectType doorType) {
		Aftik aftik = game.getAftik();
		return searchForAndIfNotBlocked(game, aftik, doorType,
				door -> enterDoor(game, aftik, door),
				() -> game.out().println("There is no such door here to go through."));
	}
	
	private static void enterDoor(GameInstance game, Aftik aftik, Door door) {
		Optional<Entity.MoveFailure> move = aftik.tryMoveTo(door.getPosition());
		if (move.isEmpty()) {
			EnterResult result = door.enter(aftik);
			
			printEnterResult(game, result);
		} else {
			ActionHandler.printMoveFailure(game, move.get());
		}
	}
	
	private static int forceDoor(GameInstance game, ObjectType doorType) {
		Aftik aftik = game.getAftik();
		return searchForAndIfNotBlocked(game, aftik, doorType,
				door -> forceDoor(game, aftik, door),
				() -> game.out().println("There is no such door here."));
	}
	
	private static void forceDoor(GameInstance game, Aftik aftik, Door door) {
		Optional<Entity.MoveFailure> move = aftik.tryMoveTo(door.getPosition());
		if (move.isEmpty()) {
			ForceResult result = door.force(aftik);
			
			printForceResult(game, result);
		} else {
			ActionHandler.printMoveFailure(game, move.get());
		}
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
	
	private static void printEnterResult(GameInstance game, EnterResult result) {
		result.either().run(success -> printEnterSuccess(game, success),
				failureType -> game.out().printf("The door is %s.%n", failureType.adjective()));
	}
	
	private static void printEnterSuccess(GameInstance game, EnterResult.Success result) {
		result.usedItem().ifPresentOrElse(
				item -> game.out().printf("Using your %s, you entered the door into a new room.%n", item.lowerCaseName()),
				() -> game.out().printf("You entered the door into a new room.%n"));
	}
	
	private static void printForceResult(GameInstance game, ForceResult result) {
		result.either().run(success -> printForceSuccess(game, success), status -> printForceStatus(game, status));
	}
	
	private static void printForceSuccess(GameInstance game, ForceResult.Success result) {
		game.out().printf("You use your %s to %s.%n", result.item().lowerCaseName(), result.method().text());
	}
	
	private static void printForceStatus(GameInstance game, ForceResult.Status status) {
		switch(status) {
			case NOT_STUCK -> game.out().println("The door does not seem to be stuck.");
			case NEED_TOOL -> game.out().println("You need some sort of tool to force the door open.");
			case NEED_BREAK_TOOL -> game.out().println("You need some sort of tool to break the door open.");
		}
	}
}