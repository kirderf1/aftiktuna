package me.kirderf.aftiktuna.level.object;

public final class DoorStuckProperty extends DoorProperty {
	
	public boolean checkEntry(Aftik aftik) {
		System.out.println("The door is stuck.");
		return false;
	}
	
	public DoorProperty tryForce(Aftik aftik) {
		if(aftik.hasItem(ObjectType.CROWBAR)) {
			System.out.println("You use your crowbar to force the door open.");
			return EMPTY;
		} else if(aftik.hasItem(ObjectType.BLOWTORCH)) {
			System.out.println("You use your blowtorch to cut the door open.");
			return EMPTY;
		} else {
			System.out.println("You need some sort of tool to force the door open.");
			return this;
		}
	}
}