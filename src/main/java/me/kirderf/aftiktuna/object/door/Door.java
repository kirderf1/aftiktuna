package me.kirderf.aftiktuna.object.door;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.location.Position;
import me.kirderf.aftiktuna.object.ObjectType;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.concurrent.atomic.AtomicReference;

public final class Door extends GameObject {
	public static final OptionalFunction<GameObject, Door> CAST = OptionalFunction.cast(Door.class);
	
	private final Position destination;
	private final AtomicReference<DoorProperty> property;
	
	public Door(ObjectType type, Position destination, AtomicReference<DoorProperty> property) {
		super(type, 20);
		this.destination = destination;
		this.property = property;
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
		return new ForceResult(new DoorPair(this, destination.room()), result);
	}
}