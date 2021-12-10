package me.kirderf.aftiktuna.action;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;

import java.util.Optional;

public final class EnterDoorAction {
	
	public static Result moveAndEnter(Aftik aftik, Door door, ActionPrinter out) {
		return moveAndEnter(aftik, door, null, out);
	}
	
	public static Result moveAndEnter(Aftik aftik, Door door, Aftik followTarget, ActionPrinter out) {
		boolean success = aftik.tryMoveTo(door.getPosition(), out);
		if (success) {
			Area originalArea = aftik.getArea();
			
			EnterResult result = door.enter(aftik);
			
			originalArea.objectStream().flatMap(Aftik.CAST.toStream())
					.forEach(other -> other.getMind().observeEnteredDoor(aftik, door, result));
			
			if (followTarget != null) {
				out.printAt(aftik, "%s follows %s into the area.", aftik.getName(), followTarget.getName());
			} else {
				printEnterResult(out, aftik, door, result);
			}
			
			return new Result(result);
		} else
			return new Result();
	}
	
	private static void printEnterResult(ActionPrinter out, Aftik aftik, Door door, EnterResult result) {
		result.either().run(success -> printEnterSuccess(out, aftik, door, success),
				failureType -> out.printFor(aftik, "The %s is %s.", door.getType().getCategoryName(), failureType.adjective()));
	}
	
	private static void printEnterSuccess(ActionPrinter out, Aftik aftik, Door door, EnterResult.Success result) {
		result.usedItem().ifPresentOrElse(
				item -> out.printFor(aftik, "Using their %s, %s entered the %s into a new area.", item.name(), aftik.getName(), door.getType().getCategoryName()),
				() -> out.printFor(aftik, "%s entered the %s into a new area.", aftik.getName(), door.getType().getCategoryName()));
	}
	
	public record Result(Optional<EnterResult> optional) {
		public Result(EnterResult result) {
			this(Optional.of(result));
		}
		public Result() {
			this(Optional.empty());
		}
		
		public boolean success() {
			return optional.map(EnterResult::success).orElse(false);
		}
	}
}