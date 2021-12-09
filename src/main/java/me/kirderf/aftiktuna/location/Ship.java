package me.kirderf.aftiktuna.location;

import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.door.DoorProperty;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;

public final class Ship {
	private final Area room = new Area("Ship", 4);
	private final Position entrancePos = room.getPosAt(3);
	private boolean isShipLaunching = false;
	
	public Ship() {
		room.addItem(ObjectTypes.MEDKIT, 1);
	}
	
	public void separateFromLocation() {
		room.objectStream().filter(ObjectTypes.SHIP_EXIT::matching).toList().forEach(GameObject::remove);
	}
	
	public void createEntrance(Position destination) {
		Location.createDoors(ObjectTypes.SHIP_ENTRANCE, destination, ObjectTypes.SHIP_EXIT, entrancePos, DoorProperty.EMPTY);
	}
	
	public Area getRoom() {
		return room;
	}
	
	public void tryLaunchShip(Aftik aftik, ActionPrinter out) {
		if (!isShipLaunching && aftik.getArea() == this.getRoom() && aftik.removeItem(ObjectTypes.FUEL_CAN)) {
			isShipLaunching = true;
			
			out.printAt(getRoom(), "%s refueled the ship, and set it to launch.", aftik.getName());
		} else
			out.printFor(aftik, "The ship can't be launched at this time.");
	}
	
	public boolean getAndClearIsLaunching() {
		boolean isLaunching = isShipLaunching;
		isShipLaunching = false;
		return isLaunching;
	}
}