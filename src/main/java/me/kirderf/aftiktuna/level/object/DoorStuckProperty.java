package me.kirderf.aftiktuna.level.object;

public final class DoorStuckProperty {
	public static DoorStuckProperty EMPTY = new DoorStuckProperty();
	static {
		EMPTY.isStuck = false;
	}
	
	private boolean isStuck = true;
	
	public boolean checkEntry() {
		if (isStuck) {
			System.out.println("The door is stuck.");
			return false;
		} else {
			return true;
		}
	}
	
	public void tryForce(Aftik aftik) {
		if (isStuck) {
			if (aftik.hasItem(ObjectType.CROWBAR)) {
				isStuck = false;
				System.out.println("You use your crowbar to force the door open.");
			} else if (aftik.hasItem(ObjectType.BLOWTORCH)) {
				isStuck = false;
				System.out.println("You use your blowtorch to cut the door open.");
			} else {
				System.out.println("You need some sort of tool to force the door open.");
			}
		} else {
			System.out.println("The door does not seem to be stuck.");
		}
	}
}