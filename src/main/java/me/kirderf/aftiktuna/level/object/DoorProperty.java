package me.kirderf.aftiktuna.level.object;

public abstract class DoorProperty {
	public static final DoorProperty EMPTY = new DoorProperty() {
		@Override
		public boolean checkEntry() {
			return true;
		}
	};
	
	public abstract boolean checkEntry();
	
	public void tryForce(Aftik aftik) {
		System.out.println("The door does not seem to be stuck.");
	}
}
