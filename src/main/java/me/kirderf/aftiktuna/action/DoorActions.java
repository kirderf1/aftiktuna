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
			
			Entity.MoveResult move = aftik.tryMoveTo(optionalDoor.get().getPosition());
			if (move.success()) {
				EnterResult result = optionalDoor.get().enter(aftik);
				
				printEnterResult(result);
				return 1;
			} else {
				ActionHandler.printMoveFailure(move);
				return 0;
			}
		} else {
			System.out.println("There is no such door here to go through.");
			return 0;
		}
	}
	
	private static int forceDoor(GameInstance game, ObjectType doorType) {
		Aftik aftik = game.getAftik();
		Optional<Door> optionalDoor = aftik.findNearest(OptionalFunction.of(doorType::matching).flatMap(Door.CAST));
		if(optionalDoor.isPresent()) {
			
			Aftik.MoveResult move = aftik.tryMoveTo(optionalDoor.get().getPosition());
			if (move.success()) {
				ForceResult result = optionalDoor.get().force(aftik);
				
				printForceResult(result);
				return 1;
			} else {
				ActionHandler.printMoveFailure(move);
				return 0;
			}
		} else {
			System.out.println("There is no such door here.");
			return 0;
		}
	}
	
	
	private static void printEnterResult(EnterResult result) {
		result.either().run(DoorActions::printEnterSuccess,
				failureType -> System.out.printf("The door is %s.%n", failureType.adjective()));
	}
	
	private static void printEnterSuccess(EnterResult.Success result) {
		result.usedItem().ifPresentOrElse(
				item -> System.out.printf("Using your %s, you entered the door into a new room.%n", item.lowerCaseName()),
				() -> System.out.printf("You entered the door into a new room.%n"));
	}
	
	private static void printForceResult(ForceResult result) {
		result.either().run(DoorActions::printForceSuccess, DoorActions::printForceStatus);
	}
	
	private static void printForceSuccess(ForceResult.Success result) {
		System.out.printf("You use your %s to %s.%n", result.item().lowerCaseName(), result.method().text());
	}
	
	private static void printForceStatus(ForceResult.Status status) {
		switch(status) {
			case NOT_STUCK -> System.out.println("The door does not seem to be stuck.");
			case NEED_TOOL -> System.out.println("You need some sort of tool to force the door open.");
			case NEED_BREAK_TOOL -> System.out.println("You need some sort of tool to break the door open.");
		}
	}
}