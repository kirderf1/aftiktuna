package me.kirderf.aftiktuna.action;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.object.ObjectArgument;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.door.DoorPair;
import me.kirderf.aftiktuna.object.door.DoorType;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;

import static me.kirderf.aftiktuna.action.ActionHandler.*;

public final class DoorActions {
	static void register() {
		DISPATCHER.register(literal("enter").then(argument("door", ObjectArgument.create(ObjectTypes.DOORS))
				.executes(context -> enterDoor(context.getSource(), ObjectArgument.getType(context, "door", DoorType.class)))));
		DISPATCHER.register(literal("force").then(argument("door", ObjectArgument.create(ObjectTypes.FORCEABLE))
				.executes(context -> forceDoor(context.getSource(), ObjectArgument.getType(context, "door", DoorType.class)))));
		DISPATCHER.register(literal("enter").then(literal("ship")
				.executes(context -> enterDoor(context.getSource(), ObjectTypes.SHIP_ENTRANCE))));
		DISPATCHER.register(literal("exit").then(literal("ship")
				.executes(context -> enterDoor(context.getSource(), ObjectTypes.SHIP_EXIT))));
	}
	
	private static int enterDoor(InputActionContext context, DoorType doorType) {
		Aftik aftik = context.getControlledAftik();
		return ActionUtil.searchForAccessible(context, aftik, Door.CAST.filter(doorType::matching), true,
				door -> context.action(out -> aftik.moveAndEnter(door, out)),
				() -> context.printNoAction("There is no such %s here to go through.", doorType.getCategoryName()));
	}
	
	private static int forceDoor(InputActionContext context, DoorType doorType) {
		Aftik aftik = context.getControlledAftik();
		return ActionUtil.searchForAccessible(context, aftik, Door.CAST.filter(doorType::matching), true,
				door -> context.action(out -> aftik.moveAndForce(door, out)),
				() -> context.printNoAction("There is no such %s here.", doorType.getCategoryName()));
	}
	
	public static void printEnterResult(ActionPrinter out, Aftik aftik, Door door, EnterResult result) {
		result.either().run(success -> printEnterSuccess(out, aftik, door, success),
				failureType -> out.printFor(aftik, "The %s is %s.", door.getType().getCategoryName(), failureType.adjective()));
	}
	
	private static void printEnterSuccess(ActionPrinter out, Aftik aftik, Door door, EnterResult.Success result) {
		result.usedItem().ifPresentOrElse(
				item -> out.printFor(aftik, "Using their %s, %s entered the %s into a new area.", item.name(), aftik.getName(), door.getType().getCategoryName()),
				() -> out.printFor(aftik, "%s entered the %s into a new area.", aftik.getName(), door.getType().getCategoryName()));
	}
	
	public static void printForceResult(ActionPrinter out, Aftik aftik, Door door, ForceResult result) {
		result.propertyResult().either().run(success -> printForceSuccess(out, aftik, result.pair(), success), status -> printForceStatus(out, aftik, door, status));
	}
	
	private static void printForceSuccess(ActionPrinter out, Aftik aftik, DoorPair pair, ForceResult.Success result) {
		out.printAt(pair.targetDoor(), "%s used their %s and %s the %s.", aftik.getName(), result.item().name(), result.method().text(), pair.targetDoor().getType().getCategoryName());
		out.printAt(pair.destination(), "A %s was %s from the other side.", pair.targetDoor().getType().getCategoryName(), result.method().text());
	}
	
	private static void printForceStatus(ActionPrinter out, Aftik aftik, Door door, ForceResult.Status status) {
		switch(status) {
			case NOT_STUCK -> out.printFor(aftik, "The %s does not seem to be stuck.", door.getType().getCategoryName());
			case NEED_TOOL -> out.printFor(aftik, "%s need some sort of tool to force the %s open.", aftik.getName(), door.getType().getCategoryName());
			case NEED_BREAK_TOOL -> out.printFor(aftik, "%s need some sort of tool to break the %s open.", aftik.getName(), door.getType().getCategoryName());
		}
	}
}