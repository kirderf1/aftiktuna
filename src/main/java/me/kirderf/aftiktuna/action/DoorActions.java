package me.kirderf.aftiktuna.action;

import com.mojang.brigadier.CommandDispatcher;
import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.level.object.ObjectArgument;
import me.kirderf.aftiktuna.level.object.ObjectType;
import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.door.Door;
import me.kirderf.aftiktuna.level.object.door.EnterResult;
import me.kirderf.aftiktuna.level.object.door.ForceResult;
import me.kirderf.aftiktuna.level.object.entity.Aftik;
import me.kirderf.aftiktuna.level.object.entity.Entity;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.Optional;

import static me.kirderf.aftiktuna.action.ActionHandler.argument;
import static me.kirderf.aftiktuna.action.ActionHandler.literal;

public final class DoorActions {
	static void register(CommandDispatcher<GameInstance> dispatcher) {
		dispatcher.register(literal("enter").then(argument("door", ObjectArgument.create(ObjectTypes.DOORS))
				.executes(context -> goThroughDoor(context.getSource(), ObjectArgument.getType(context, "door")))));
		dispatcher.register(literal("force").then(argument("door", ObjectArgument.create(ObjectTypes.DOORS))
				.executes(context -> forceDoor(context.getSource(), ObjectArgument.getType(context, "door")))));
	}
	
	private static int goThroughDoor(GameInstance game, ObjectType doorType) {
		Aftik aftik = game.getAftik();
		Optional<Door> optionalDoor = aftik.findNearest(OptionalFunction.of(doorType::matching).flatMap(Door.CAST));
		if(optionalDoor.isPresent()) {
			
			Optional<Entity.MoveFailure> move = aftik.tryMoveTo(optionalDoor.get().getPosition());
			if (move.isEmpty()) {
				EnterResult result = optionalDoor.get().enter(aftik);
				
				printEnterResult(game, result);
				return 1;
			} else {
				ActionHandler.printMoveFailure(game, move.get());
				return 0;
			}
		} else {
			game.out().println("There is no such door here to go through.");
			return 0;
		}
	}
	
	private static int forceDoor(GameInstance game, ObjectType doorType) {
		Aftik aftik = game.getAftik();
		Optional<Door> optionalDoor = aftik.findNearest(OptionalFunction.of(doorType::matching).flatMap(Door.CAST));
		if(optionalDoor.isPresent()) {
			
			Optional<Entity.MoveFailure> move = aftik.tryMoveTo(optionalDoor.get().getPosition());
			if (move.isEmpty()) {
				ForceResult result = optionalDoor.get().force(aftik);
				
				printForceResult(game, result);
				return 1;
			} else {
				ActionHandler.printMoveFailure(game, move.get());
				return 0;
			}
		} else {
			game.out().println("There is no such door here.");
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