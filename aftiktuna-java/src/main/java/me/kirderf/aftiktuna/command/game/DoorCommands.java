package me.kirderf.aftiktuna.command.game;

import me.kirderf.aftiktuna.action.EnterDoorAction;
import me.kirderf.aftiktuna.action.ForceDoorAction;
import me.kirderf.aftiktuna.command.CommandContext;
import me.kirderf.aftiktuna.command.CommandUtil;
import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.location.Ship;
import me.kirderf.aftiktuna.object.ObjectArgument;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.door.DoorType;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.entity.ai.MoveToAreaTask;
import me.kirderf.aftiktuna.object.type.ObjectTypes;

import java.util.Optional;

import static me.kirderf.aftiktuna.command.game.GameCommands.*;

public final class DoorCommands {
	static void register() {
		DISPATCHER.register(literal("enter").then(argument("door", ObjectArgument.create(ObjectTypes.DOORS))
				.executes(context -> enterDoor(context.getSource(), ObjectArgument.getType(context, "door", DoorType.class)))));
		DISPATCHER.register(literal("force").then(argument("door", ObjectArgument.create(ObjectTypes.FORCEABLE))
				.executes(context -> forceDoor(context.getSource(), ObjectArgument.getType(context, "door", DoorType.class)))));
		DISPATCHER.register(literal("enter").then(literal("ship")
				.executes(context -> enterShip(context.getSource()))));
		DISPATCHER.register(literal("exit").then(literal("ship")
				.executes(context -> enterDoor(context.getSource(), ObjectTypes.SHIP_EXIT))));
		DISPATCHER.register(literal("go").then(literal("back")
				.executes(context -> goBack(context.getSource()))));
	}
	
	private static int enterDoor(CommandContext context, DoorType doorType) {
		Aftik aftik = context.getControlledAftik();
		return CommandUtil.searchForAccessible(context, aftik, Door.CAST.filter(doorType::matching), true,
				door -> context.action(out -> EnterDoorAction.moveAndEnter(aftik, door, out)),
				() -> context.printNoAction("There is no such %s here to go through.", doorType.getCategoryName()));
	}
	
	private static int forceDoor(CommandContext context, DoorType doorType) {
		Aftik aftik = context.getControlledAftik();
		return CommandUtil.searchForAccessible(context, aftik, Door.CAST.filter(doorType::matching), true,
				door -> context.action(out -> ForceDoorAction.moveAndForce(aftik, door, out)),
				() -> context.printNoAction("There is no such %s here.", doorType.getCategoryName()));
	}
	
	
	private static int enterShip(CommandContext context) {
		Aftik aftik = context.getControlledAftik();
		
		if (isNearShip(aftik, context.getCrew().getShip())) {
			return context.action(out -> aftik.getMind().setAndPerformPlayerTask(new MoveToAreaTask(context.getCrew().getShip().getRoom()), out));
		} else {
			return context.printNoAction("%s need to be near the ship in order to launch it.", aftik.getName());
		}
	}
	
	private static boolean isNearShip(Aftik aftik, Ship ship) {
		return aftik.getArea() == ship.getRoom() || MoveToAreaTask.findPathTowardsArea(aftik, ship.getRoom()).isPresent();
	}
	
	private static int goBack(CommandContext context) {
		Aftik aftik = context.getControlledAftik();
		
		Optional<Area> optionalPrevious = context.getPreviousArea();
		if (optionalPrevious.isPresent()) {
			Area previousArea = optionalPrevious.get();
			return context.action(out -> aftik.getMind().setAndPerformPlayerTask(new MoveToAreaTask(previousArea), out));
		} else {
			return context.printNoAction("There is nowhere to go back to at this time.");
		}
	}
}