package me.kirderf.aftiktuna.level.object.door;

import me.kirderf.aftiktuna.level.object.Aftik;

public abstract class DoorProperty {
	public static final DoorProperty EMPTY = new DoorProperty() {
		@Override
		public EnterResult checkEntry(Aftik aftik) {
			return new EnterResult();
		}
		
		@Override
		public ForceResult tryForce(Aftik aftik) {
			System.out.println("The door does not seem to be stuck.");
			return new ForceResult(ForceResult.Status.NOT_STUCK);
		}
	};
	
	public abstract EnterResult checkEntry(Aftik aftik);
	
	public abstract ForceResult tryForce(Aftik aftik);
}