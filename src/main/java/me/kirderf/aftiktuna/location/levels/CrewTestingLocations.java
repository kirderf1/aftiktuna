package me.kirderf.aftiktuna.location.levels;

import me.kirderf.aftiktuna.location.Location;
import me.kirderf.aftiktuna.location.LocationBuilder;
import me.kirderf.aftiktuna.location.Room;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.door.DoorLockedProperty;

public final class CrewTestingLocations {
	public static Location separationTest() {
		LocationBuilder builder = new LocationBuilder();
		Room firstRoom = builder.newTestRoom(4);
		Room secondRoom = builder.newTestRoom(4);
		builder.markDoors(firstRoom.getPosAt(0), secondRoom.getPosAt(1), new DoorLockedProperty());
		firstRoom.addItem(ObjectTypes.KEYCARD, 0);
		firstRoom.addItem(ObjectTypes.BLOWTORCH, 0);
		firstRoom.addCreature(ObjectTypes.EYESAUR, 3);
		
		return builder.build(firstRoom.getPosAt(1));
	}
}
