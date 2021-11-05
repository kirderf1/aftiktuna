package me.kirderf.aftiktuna.action;

import com.mojang.brigadier.CommandDispatcher;
import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Room;
import me.kirderf.aftiktuna.level.object.ObjectArgument;
import me.kirderf.aftiktuna.level.object.ObjectType;
import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.door.Door;
import me.kirderf.aftiktuna.level.object.entity.Aftik;
import me.kirderf.aftiktuna.level.object.entity.Entity;
import me.kirderf.aftiktuna.util.Either;

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
				door -> enterDoor(game, aftik, door),
				() -> game.out().println("There is no such door here to go through."));
	}
	
	private static void enterDoor(GameInstance game, Aftik aftik, Door door) {
		Room originalRoom = aftik.getRoom();
		
		Either<EnterResult, Entity.MoveFailure> result = aftik.moveAndEnter(door);
		
		result.getLeft().ifPresent(enterResult -> {
			if (enterResult.success()) {
				originalRoom.objectStream().flatMap(Aftik.CAST.toStream()).forEach(other -> other.observeEnteredDoor(door));
			}
		});
		
		result.run(enterResult -> printEnterResult(game, aftik, enterResult),
				moveFailure -> ActionHandler.printMoveFailure(game, moveFailure));
	}
	
	private static int forceDoor(GameInstance game, ObjectType doorType) {
		Aftik aftik = game.getAftik();
		return searchForAndIfNotBlocked(game, aftik, doorType,
				door -> forceDoor(game, aftik, door),
				() -> game.out().println("There is no such door here."));
	}
	
	private static void forceDoor(GameInstance game, Aftik aftik, Door door) {
		Either<ForceResult, Entity.MoveFailure> result = aftik.moveAndForce(door);
		
		result.run(forceResult -> printForceResult(game, aftik, forceResult),
				moveFailure -> ActionHandler.printMoveFailure(game, moveFailure));
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
	
	private static void printEnterResult(GameInstance game, Aftik aftik, EnterResult result) {
		result.either().run(success -> printEnterSuccess(game, aftik, success),
				failureType -> game.out().printf("The door is %s.%n", failureType.adjective()));
	}
	
	private static void printEnterSuccess(GameInstance game, Aftik aftik, EnterResult.Success result) {
		result.usedItem().ifPresentOrElse(
				item -> game.out().printf("Using their %s, %s entered the door into a new room.%n", item.name(), aftik.getName()),
				() -> game.out().printf("%s entered the door into a new room.%n", aftik.getName()));
	}
	
	private static void printForceResult(GameInstance game, Aftik aftik, ForceResult result) {
		result.either().run(success -> printForceSuccess(game, aftik, success), status -> printForceStatus(game, aftik, status));
	}
	
	private static void printForceSuccess(GameInstance game, Aftik aftik, ForceResult.Success result) {
		game.out().printf("%s used their %s to %s.%n", aftik.getName(), result.item().name(), result.method().text());
	}
	
	private static void printForceStatus(GameInstance game, Aftik aftik, ForceResult.Status status) {
		switch(status) {
			case NOT_STUCK -> game.out().println("The door does not seem to be stuck.");
			case NEED_TOOL -> game.out().printf("%s need some sort of tool to force the door open.%n", aftik.getName());
			case NEED_BREAK_TOOL -> game.out().printf("%s need some sort of tool to break the door open.%n", aftik.getName());
		}
	}
}