package me.kirderf.aftiktuna.action;

import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.door.DoorProperty;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.type.ItemType;
import me.kirderf.aftiktuna.print.ActionPrinter;

import java.util.Optional;

public final class ForceDoorAction {
	
	public static void moveAndForce(Aftik aftik, Door door, ActionPrinter out) {
		moveAndForce(aftik, door, null, out);
	}
	
	public static void moveAndForce(Aftik aftik, Door door, ItemType item, ActionPrinter out) {
		boolean success = aftik.tryMoveTo(door.getPosition(), out);
		if (success) {
			ForceResult result = door.force(aftik, item);
			
			printForceResult(out, aftik, door, result);
		}
	}
	
	public static Optional<Door> findForceTargetForTool(Aftik aftik, ItemType item) {
		return aftik.findNearestAccessible(Door.CAST.filter(door -> isToolForDoor(door, item)), true);
	}
	
	private static boolean isToolForDoor(Door door, ItemType item) {
		return door.getProperty().canBeForcedWith(item.getForceMethod());
	}
	
	private static void printForceResult(ActionPrinter out, Aftik aftik, Door door, ForceResult result) {
		result.propertyResult().either().run(success -> printForceSuccess(out, aftik, result.door(), result.destination(), success), status -> printForceStatus(out, aftik, door, status));
	}
	
	private static void printForceSuccess(ActionPrinter out, Aftik aftik, Door door, Area destination, ForceResult.Success result) {
		out.printAt(door, "%s used their %s and %s the %s.", aftik.getName(), result.item().name(), result.method().text(), door.getType().getCategoryName());
		out.printAt(destination, "A %s was %s from the other side.", door.getType().getCategoryName(), result.method().text());
	}
	
	private static void printForceStatus(ActionPrinter out, Aftik aftik, Door door, DoorProperty.Status status) {
		switch(status) {
			case NOT_STUCK -> out.printFor(aftik, "The %s does not seem to be stuck.", door.getType().getCategoryName());
			case NEED_TOOL -> out.printFor(aftik, "%s need some sort of tool to force the %s open.", aftik.getName(), door.getType().getCategoryName());
			case NEED_BREAK_TOOL -> out.printFor(aftik, "%s need some sort of tool to break the %s open.", aftik.getName(), door.getType().getCategoryName());
		}
	}
}