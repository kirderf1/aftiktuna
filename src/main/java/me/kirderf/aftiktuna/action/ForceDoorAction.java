package me.kirderf.aftiktuna.action;

import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.door.DoorProperty;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;

public final class ForceDoorAction {
	
	public static void moveAndForce(Aftik aftik, Door door, ActionPrinter out) {
		boolean success = aftik.tryMoveTo(door.getPosition(), out);
		if (success) {
			ForceResult result = door.force(aftik);
			
			door.getArea().objectStream().flatMap(Aftik.CAST.toStream())
					.forEach(other -> other.getMind().observeForcedDoor(door, result));
			
			printForceResult(out, aftik, door, result);
		}
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