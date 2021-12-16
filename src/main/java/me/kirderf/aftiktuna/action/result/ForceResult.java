package me.kirderf.aftiktuna.action.result;

import me.kirderf.aftiktuna.object.door.DoorPair;
import me.kirderf.aftiktuna.object.door.DoorProperty;
import me.kirderf.aftiktuna.object.type.ItemType;
import me.kirderf.aftiktuna.object.type.ObjectTypes;
import me.kirderf.aftiktuna.util.Either;

import java.util.Optional;

public record ForceResult(DoorPair pair, PropertyResult propertyResult) {
	
	public static PropertyResult success(Method method) {
		return new PropertyResult(Either.left(new Success(DoorProperty.EMPTY, method.tool(), method)));
	}
	
	public static PropertyResult status(Status status) {
		return new PropertyResult(Either.right(status));
	}
	
	public static record PropertyResult(Either<Success, Status> either) {
		public Optional<DoorProperty> getNewProperty() {
			return either.getLeft().map(Success::newProperty);
		}
	}
	
	public record Success(DoorProperty newProperty, ItemType item, Method method) {}
	
	
	public record Method(ItemType tool, String text) {
		public static final Method FORCE = new Method(ObjectTypes.CROWBAR, "forced open");
		public static final Method CUT = new Method(ObjectTypes.BLOWTORCH, "cut open");
	}
	
	public enum Status {
		NEED_TOOL,
		NEED_BREAK_TOOL,
		NOT_STUCK
	}
}