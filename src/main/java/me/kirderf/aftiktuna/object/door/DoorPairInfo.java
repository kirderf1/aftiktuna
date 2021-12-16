package me.kirderf.aftiktuna.object.door;

import me.kirderf.aftiktuna.object.Identifier;

public final class DoorPairInfo {
	private final Identifier id = Identifier.newId();
	private DoorProperty property;
	
	public DoorPairInfo(DoorProperty property) {
		this.property = property;
	}
	
	public Identifier getId() {
		return id;
	}
	
	public DoorProperty getProperty() {
		return property;
	}
	
	public void setProperty(DoorProperty property) {
		this.property = property;
	}
}