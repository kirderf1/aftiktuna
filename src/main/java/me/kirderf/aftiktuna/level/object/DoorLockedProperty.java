package me.kirderf.aftiktuna.level.object;

public class DoorLockedProperty extends DoorProperty {
	private boolean intact = true;
	
	@Override
	public boolean checkEntry() {
		if (intact) {
			System.out.println("The door is locked.");
			return false;
		} else {
			return true;
		}
	}
	
	@Override
	public void tryForce(Aftik aftik) {
		if (intact) {
			if (aftik.hasItem(ObjectType.BLOWTORCH)) {
				intact = false;
				System.out.println("You use your blowtorch to cut the door open.");
			} else {
				System.out.println("You need some sort of tool to break the door open.");
			}
		} else super.tryForce(aftik);
	}
}
