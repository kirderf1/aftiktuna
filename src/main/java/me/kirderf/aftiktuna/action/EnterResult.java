package me.kirderf.aftiktuna.action;

import me.kirderf.aftiktuna.level.object.ObjectType;
import me.kirderf.aftiktuna.util.Either;

import java.util.Optional;

public record EnterResult(Either<Success, FailureType> either) {
	public EnterResult(ObjectType usedItem) {
		this(Either.left(new Success(Optional.of(usedItem))));
	}
	
	public EnterResult() {
		this(Either.left(new Success(Optional.empty())));
	}
	
	public EnterResult(FailureType failure) {
		this(Either.right(failure));
	}
	
	public boolean success() {
		return either.isLeft();
	}
	
	public static record Success(Optional<ObjectType> usedItem) {}
	
	public record FailureType(String adjective) {
		public static final EnterResult.FailureType STUCK = new EnterResult.FailureType("stuck");
		public static final EnterResult.FailureType LOCKED = new EnterResult.FailureType("locked");
		public static final EnterResult.FailureType SEALED = new EnterResult.FailureType("sealed shut");
	}
}
