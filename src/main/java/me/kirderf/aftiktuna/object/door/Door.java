package me.kirderf.aftiktuna.object.door;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.location.Position;
import me.kirderf.aftiktuna.object.Identifier;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.type.ItemType;
import me.kirderf.aftiktuna.object.type.ObjectTypes;
import me.kirderf.aftiktuna.util.OptionalFunction;

public final class Door extends GameObject {
	public static final OptionalFunction<GameObject, Door> CAST = OptionalFunction.cast(Door.class);
	
	private final DoorType type;
	private final Position destination;
	private final DoorPairInfo pairInfo;
	
	public Door(DoorType type, Position destination, DoorPairInfo pairInfo) {
		super(type, 20);
		this.destination = destination;
		this.pairInfo = pairInfo;
		this.type = type;
		
		if (!ObjectTypes.DOORS.contains(type))
			throw new IllegalArgumentException("Invalid door type %s".formatted(type.name()));
	}
	
	public Identifier<DoorPairInfo> getPairId() {
		return pairInfo.getId();
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
		
		return aftik.getMind().getMemory().getObservedProperty(this).getAdjective()
				.map(adjective -> name + " (%s)".formatted(adjective))
				.orElse(name);
	}
	
	public EnterResult enter(Aftik aftik) {
		EnterResult result = pairInfo.getProperty().checkEntry(aftik);
		if (result.success()) {
			aftik.teleport(destination);
		}
		return result;
	}
	
	public ForceResult force(Aftik aftik, ItemType item) {
		ForceResult.PropertyResult result = pairInfo.getProperty().tryForce(aftik, item);
		
		result.getNewProperty().ifPresent(pairInfo::setProperty);
		return new ForceResult(this, destination.area(), result);
	}
	
	public Area getDestinationArea() {
		return destination.area();
	}
}