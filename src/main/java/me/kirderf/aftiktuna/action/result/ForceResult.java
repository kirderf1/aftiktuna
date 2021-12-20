package me.kirderf.aftiktuna.action.result;

import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.door.DoorProperty;
import me.kirderf.aftiktuna.object.type.ItemType;
import me.kirderf.aftiktuna.util.Either;

import java.util.Objects;
import java.util.Optional;

public record ForceResult(Door door, Area destination, PropertyResult propertyResult) {
	
	public static PropertyResult success(ItemType tool) {
		return new PropertyResult(Either.left(
				new Success(DoorProperty.EMPTY, tool, Objects.requireNonNull(tool.getForceMethod()))
		));
	}
	
	public static PropertyResult status(DoorProperty.ForceStatus status) {
		return new PropertyResult(Either.right(status));
	}
	
	public record PropertyResult(Either<Success, DoorProperty.ForceStatus> either) {
		public Optional<DoorProperty> getNewProperty() {
			return either.getLeft().map(Success::newProperty);
		}
	}
	
	public record Success(DoorProperty newProperty, ItemType item, DoorProperty.Method method) {}
	
	
}