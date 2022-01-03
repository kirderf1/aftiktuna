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
		boolean success = aftik.tryMoveTo(door.getPosition(), out);
		if (success) {
			Area originalArea = aftik.getArea();
			
			EnterResult result = door.enter(aftik);
			
			Stream.concat(originalArea.objectStream(), aftik.getArea().objectStream())
					.flatMap(Aftik.CAST.toStream()).distinct()
					.forEach(other -> other.getMind().observeEnteredDoor(aftik, door, result));
			
			if (result.success())
				aftik.getMind().getMemory().observeNewConnection(originalArea, aftik.getArea(), door.getPairId());
			
			printEnterResult(out, aftik, door, result);
			
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
				item -> out.printFrom(door.getArea(), "Using their %s, %s entered the %s into a new area.", item.name(), aftik.getName(), door.getType().getCategoryName()),
				() -> out.printFrom(door.getArea(), "%s entered the %s into a new area.", aftik.getName(), door.getType().getCategoryName()));
		out.printFrom(aftik.getArea(), "%s enters the area.", aftik.getName());
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