package me.kirderf.aftiktuna.location.levels;

import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.location.Location;
import me.kirderf.aftiktuna.location.LocationBuilder;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.door.DoorLockedProperty;
import me.kirderf.aftiktuna.object.entity.AftikNPC;
import me.kirderf.aftiktuna.object.entity.Stats;

@SuppressWarnings("unused")
public final class CrewTestingLocations {
	public static Location separationTest() {
		LocationBuilder builder = new LocationBuilder();
		Area firstRoom = builder.newTestRoom(4);
		Area secondRoom = builder.newTestRoom(4);
		builder.markDoors(firstRoom.getPosAt(0), secondRoom.getPosAt(1), new DoorLockedProperty());
		firstRoom.addItem(ObjectTypes.KEYCARD, 0);
		firstRoom.addItem(ObjectTypes.BLOWTORCH, 0);
		firstRoom.addCreature(ObjectTypes.EYESAUR, 3);
		
		return builder.build(firstRoom.getPosAt(1));
	}
	
	public static Location recruitment() {
		LocationBuilder builder = new LocationBuilder();
		Area room = builder.newTestRoom(4);
		
		room.addObject(new AftikNPC("Plum", new Stats(10, 2, 9)), 3);
		
		return builder.build(room.getPosAt(0));
	}
}
