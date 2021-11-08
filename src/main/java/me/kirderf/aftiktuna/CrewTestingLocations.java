package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.level.Location;
import me.kirderf.aftiktuna.level.LocationBuilder;
import me.kirderf.aftiktuna.level.Room;
import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.door.DoorLockedProperty;
import me.kirderf.aftiktuna.level.object.entity.Creature;

public final class CrewTestingLocations {
	public static Location separationTest() {
		LocationBuilder builder = new LocationBuilder();
		Room firstRoom = builder.newRoom(4);
		Room secondRoom = builder.newRoom(4);
		builder.markDoors(firstRoom.getPosAt(0), secondRoom.getPosAt(1), new DoorLockedProperty());
		firstRoom.addItem(ObjectTypes.KEYCARD, 0);
		firstRoom.addItem(ObjectTypes.BLOWTORCH, 0);
		firstRoom.addObject(new Creature(true), 3);
		
		return builder.build(firstRoom.getPosAt(1));
	}
}
