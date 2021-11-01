package me.kirderf.aftiktuna.level.object.door;

import me.kirderf.aftiktuna.level.object.Aftik;

public abstract class DoorProperty {
	public static final DoorProperty EMPTY = new DoorProperty() {
		@Override
		public EnterResult checkEntry(Aftik aftik) {
			return new EnterResult();
		}
		
		@Override
		public DoorProperty tryForce(Aftik aftik) {
			System.out.println("The door does not seem to be stuck.");
			return this;
		}
	};
	
	public abstract EnterResult checkEntry(Aftik aftik);
	
	public abstract DoorProperty tryForce(Aftik aftik);
}