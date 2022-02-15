package me.kirderf.aftiktuna.action.result;

import me.kirderf.aftiktuna.object.door.DoorProperty;
import me.kirderf.aftiktuna.object.type.ObjectType;
import me.kirderf.aftiktuna.util.Either;

import java.util.Optional;

public record EnterResult(DoorProperty property, Either<Success, String> either) {
	
	public static EnterResult success(DoorProperty property, ObjectType usedItem) {
		return new EnterResult(property, Either.left(new Success(Optional.of(usedItem))));
	}
	
	public static EnterResult success(DoorProperty property) {
		return new EnterResult(property, Either.left(new Success(Optional.empty())));
	}
	 public static EnterResult failure(DoorProperty property, String adjective) {
		return new EnterResult(property, Either.right(adjective));
	 }
	 
	public boolean success() {
		return either.isLeft();
	}
	
	public record Success(Optional<ObjectType> usedItem) {}
	
}
