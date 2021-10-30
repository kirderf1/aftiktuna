package me.kirderf.aftiktuna.level.object;

public abstract class DoorProperty {
	public static final DoorProperty EMPTY = new DoorProperty() {
		@Override
		public boolean checkEntry() {
			return true;
		}
		
		@Override
		public DoorProperty tryForce(Aftik aftik) {
			System.out.println("The door does not seem to be stuck.");
			return this;
		}
	};
	
	public abstract boolean checkEntry();
	
	public abstract DoorProperty tryForce(Aftik aftik);
}