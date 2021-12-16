package me.kirderf.aftiktuna.object.door;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.object.entity.Aftik;

public abstract class DoorProperty {
	public static final DoorProperty EMPTY = new DoorProperty(ForceResult.Status.NOT_STUCK) {
		@Override
		public EnterResult checkEntry(Aftik aftik) {
			return new EnterResult();
		}
	};
	
	private final ForceResult.Status forceStatus;
	
	protected DoorProperty(ForceResult.Status forceStatus) {
		this.forceStatus = forceStatus;
	}
	
	public abstract EnterResult checkEntry(Aftik aftik);
	
	public final ForceResult.PropertyResult tryForce(Aftik aftik) {
		for (ForceResult.Method method : forceStatus.getAvailableMethods()) {
			if(aftik.hasItem(method.tool())) {
				return ForceResult.success(method);
			}
		}
		return ForceResult.status(forceStatus);
	}
}