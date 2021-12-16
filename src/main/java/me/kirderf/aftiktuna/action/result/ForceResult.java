package me.kirderf.aftiktuna.action.result;

import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.door.DoorProperty;
import me.kirderf.aftiktuna.object.type.ItemType;
import me.kirderf.aftiktuna.util.Either;

import java.util.Optional;

public record ForceResult(Door door, Area destination, PropertyResult propertyResult) {
	
	public static PropertyResult success(DoorProperty.Method method) {
		return new PropertyResult(Either.left(new Success(DoorProperty.EMPTY, method.tool(), method)));
	}
	
	public static PropertyResult status(DoorProperty.Status status) {
		return new PropertyResult(Either.right(status));
	}
	
	public record PropertyResult(Either<Success, DoorProperty.Status> either) {
		public Optional<DoorProperty> getNewProperty() {
			return either.getLeft().map(Success::newProperty);
		}
	}
	
	public record Success(DoorProperty newProperty, ItemType item, DoorProperty.Method method) {}
	
	
}