package me.kirderf.aftiktuna.object.door;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.object.entity.Aftik;

public abstract class DoorProperty {
	public static final DoorProperty EMPTY = new DoorProperty() {
		@Override
		public EnterResult checkEntry(Aftik aftik) {
			return new EnterResult();
		}
		
		@Override
		public ForceResult.PropertyResult tryForce(Aftik aftik) {
			return ForceResult.status(ForceResult.Status.NOT_STUCK);
		}
	};
	
	public abstract EnterResult checkEntry(Aftik aftik);
	
	public abstract ForceResult.PropertyResult tryForce(Aftik aftik);
}