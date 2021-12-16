package me.kirderf.aftiktuna.object.door;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.location.Position;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.type.ObjectTypes;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.concurrent.atomic.AtomicReference;

public final class Door extends GameObject {
	public static final OptionalFunction<GameObject, Door> CAST = OptionalFunction.cast(Door.class);
	
	private final DoorType type;
	private final Position destination;
	private final AtomicReference<DoorProperty> property;
	
	public Door(DoorType type, Position destination, AtomicReference<DoorProperty> property) {
		super(type, 20);
		this.destination = destination;
		this.property = property;
		this.type = type;
		
		if (!ObjectTypes.DOORS.contains(type))
			throw new IllegalArgumentException("Invalid door type %s".formatted(type.name()));
	}
	
	@Override
	public DoorType getType() {
		return type;
	}
	
	@Override
	public boolean hasCustomName() {
		return true;
	}
	
	@Override
	public String getViewLabel(Aftik aftik) {
		String name = super.getViewLabel(aftik);
		
		DoorProperty.FailureType entryType = aftik.getMind().getMemory().getObservedFailureType(this);
		if (entryType != null)
			return name + " (%s)".formatted(entryType.adjective());
		else
			return name;
	}
	
	public EnterResult enter(Aftik aftik) {
		EnterResult result = property.get().checkEntry(aftik);
		if (result.success()) {
			aftik.teleport(destination);
		}
		return result;
	}
	
	public ForceResult force(Aftik aftik) {
		ForceResult.PropertyResult result = property.get().tryForce(aftik);
		
		result.getNewProperty().ifPresent(property::set);
		return new ForceResult(new DoorPair(this, destination.area()), result);
	}
}