package me.kirderf.aftiktuna.level.object;

public class DoorLockedProperty extends DoorProperty {
	
	@Override
	public boolean checkEntry(Aftik aftik) {
		if (aftik.hasItem(ObjectType.KEYCARD)) {
			System.out.println("Using your keycard, you entered the door into a new room.");
			return true;
		} else {
			System.out.println("The door is locked.");
			return false;
		}
	}
	
	@Override
	public DoorProperty tryForce(Aftik aftik) {
		if(aftik.hasItem(ObjectType.BLOWTORCH)) {
			System.out.println("You use your blowtorch to cut the door open.");
			return EMPTY;
		} else {
			System.out.println("You need some sort of tool to break the door open.");
			return this;
		}
	}
}
