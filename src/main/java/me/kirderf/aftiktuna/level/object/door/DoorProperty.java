package me.kirderf.aftiktuna.level.object.door;

import me.kirderf.aftiktuna.level.object.entity.Aftik;

public abstract class DoorProperty {
	public static final DoorProperty EMPTY = new DoorProperty() {
		@Override
		public EnterResult checkEntry(Aftik aftik) {
			return new EnterResult();
		}
		
		@Override
		public ForceResult tryForce(Aftik aftik) {
			return new ForceResult(ForceResult.Status.NOT_STUCK);
		}
	};
	
	public abstract EnterResult checkEntry(Aftik aftik);
	
	public abstract ForceResult tryForce(Aftik aftik);
}