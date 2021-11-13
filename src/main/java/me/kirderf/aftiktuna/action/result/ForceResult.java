package me.kirderf.aftiktuna.action.result;

import me.kirderf.aftiktuna.object.ObjectType;
import me.kirderf.aftiktuna.object.door.DoorPair;
import me.kirderf.aftiktuna.object.door.DoorProperty;
import me.kirderf.aftiktuna.util.Either;

import java.util.Optional;

public record ForceResult(DoorPair pair, PropertyResult propertyResult) {
	
	public static PropertyResult success(ObjectType item, Method method) {
		return new PropertyResult(Either.left(new Success(DoorProperty.EMPTY, item, method)));
	}
	
	public static PropertyResult status(Status status) {
		return new PropertyResult(Either.right(status));
	}
	
	public static record PropertyResult(Either<Success, Status> either) {
		public Optional<DoorProperty> getNewProperty() {
			return either.getLeft().map(Success::newProperty);
		}
	}
	
	public static record Success(DoorProperty newProperty, ObjectType item, Method method) {}
	
	
	public final record Method(String text) {
		public static final Method FORCE = new Method("forced open");
		public static final Method CUT = new Method("cut open");
	}
	
	public enum Status {
		NEED_TOOL,
		NEED_BREAK_TOOL,
		NOT_STUCK
	}
}