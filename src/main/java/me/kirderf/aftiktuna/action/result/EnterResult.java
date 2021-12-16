package me.kirderf.aftiktuna.action.result;

import me.kirderf.aftiktuna.object.door.DoorProperty;
import me.kirderf.aftiktuna.object.type.ObjectType;
import me.kirderf.aftiktuna.util.Either;

import java.util.Optional;

public record EnterResult(Either<Success, DoorProperty.FailureType> either) {
	public EnterResult(ObjectType usedItem) {
		this(Either.left(new Success(Optional.of(usedItem))));
	}
	
	public EnterResult() {
		this(Either.left(new Success(Optional.empty())));
	}
	
	public EnterResult(DoorProperty.FailureType failure) {
		this(Either.right(failure));
	}
	
	public boolean success() {
		return either.isLeft();
	}
	
	public record Success(Optional<ObjectType> usedItem) {}
	
}
