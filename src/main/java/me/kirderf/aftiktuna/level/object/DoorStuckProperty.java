package me.kirderf.aftiktuna.level.object;

public final class DoorStuckProperty {
	public static DoorStuckProperty EMPTY = new DoorStuckProperty();
	static {
		EMPTY.unstuck();
	}
	
	private boolean isStuck = true;
	
	public boolean isStuck() {
		return isStuck;
	}
	
	public void unstuck() {
		isStuck = false;
	}
}