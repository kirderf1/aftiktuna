package me.kirderf.aftiktuna.level;

import me.kirderf.aftiktuna.ContextPrinter;
import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.door.DoorProperty;
import me.kirderf.aftiktuna.level.object.entity.Aftik;

import java.util.stream.Collectors;

public final class Ship {
	private final Room room = new Room(4);
	private final Position entrancePos = room.getPosAt(3);
	private boolean isShipLaunching = false;
	
	public void separateFromLocation() {
		room.objectStream().filter(ObjectTypes.SHIP_EXIT::matching).collect(Collectors.toList()).forEach(GameObject::remove);
	}
	
	public void createEntrance(Position destination) {
		Location.createDoors(ObjectTypes.SHIP_ENTRANCE, destination, ObjectTypes.SHIP_EXIT, entrancePos, DoorProperty.EMPTY);
	}
	
	public Room getRoom() {
		return room;
	}
	
	public void tryLaunchShip(Aftik aftik, ContextPrinter out) {
		if (!isShipLaunching && aftik.getRoom() == this.getRoom() && aftik.removeItem(ObjectTypes.FUEL_CAN)) {
			isShipLaunching = true;
			
			out.printAt(getRoom(), "%s got fuel to the ship.%n", aftik.getName());
		} else
			out.printFor(aftik, "The ship can't be launched at this time.%n");
	}
	
	public boolean getAndClearIsLaunching() {
		boolean isLaunching = isShipLaunching;
		isShipLaunching = false;
		return isLaunching;
	}
}