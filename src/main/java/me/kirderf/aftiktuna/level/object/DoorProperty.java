package me.kirderf.aftiktuna.level.object;

public abstract class DoorProperty {
	public static final DoorProperty EMPTY = new DoorProperty() {
		@Override
		public boolean checkEntry(Aftik aftik) {
			System.out.println("You entered the door into a new room.");
			return true;
		}
		
		@Override
		public DoorProperty tryForce(Aftik aftik) {
			System.out.println("The door does not seem to be stuck.");
			return this;
		}
	};
	
	public abstract boolean checkEntry(Aftik aftik);
	
	public abstract DoorProperty tryForce(Aftik aftik);
}