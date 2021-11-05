package me.kirderf.aftiktuna.action;

import me.kirderf.aftiktuna.level.object.ObjectType;
import me.kirderf.aftiktuna.level.object.door.DoorProperty;
import me.kirderf.aftiktuna.util.Either;

import java.util.Optional;

public record ForceResult(Either<Success, Status> either) {
	
	public ForceResult(Status status) {
		this(Either.right(status));
	}
	
	public ForceResult(ObjectType item, Method method) {
		this(Either.left(new Success(DoorProperty.EMPTY, item, method)));
	}
	
	public Optional<DoorProperty> getNewProperty() {
		return either.getLeft().map(Success::newProperty);
	}
	
	public static record Success(DoorProperty newProperty, ObjectType item, Method method) {}
	
	public final record Method(String text) {
		public static final Method FORCE = new Method("force the door open");
		public static final Method CUT = new Method("cut the door open");
	}
	
	public enum Status {
		NEED_TOOL,
		NEED_BREAK_TOOL,
		NOT_STUCK
	}
}