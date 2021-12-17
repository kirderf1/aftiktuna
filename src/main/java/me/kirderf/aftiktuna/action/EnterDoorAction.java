package me.kirderf.aftiktuna.action;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;

import java.util.Optional;
import java.util.stream.Stream;

public final class EnterDoorAction {
	
	public static Result moveAndEnter(Aftik aftik, Door door, ActionPrinter out) {
		return moveAndEnter(aftik, door, null, out);
	}
	
	public static Result moveAndEnter(Aftik aftik, Door door, Aftik followTarget, ActionPrinter out) {
		boolean success = aftik.tryMoveTo(door.getPosition(), out);
		if (success) {
			Area originalArea = aftik.getArea();
			
			EnterResult result = door.enter(aftik);
			
			Stream.concat(originalArea.objectStream(), aftik.getArea().objectStream())
					.flatMap(Aftik.CAST.toStream()).distinct()
					.forEach(other -> other.getMind().observeEnteredDoor(aftik, door, result));
			
			if (result.success())
				aftik.getMind().getMemory().observeNewConnection(originalArea, aftik.getArea(), door.getPairId());
			
			if (followTarget != null && result.success()) {
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
				adjective -> out.printFor(aftik, "The %s is %s.", door.getType().getCategoryName(), adjective));
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