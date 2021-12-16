package me.kirderf.aftiktuna.object.door;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.object.entity.Aftik;

public class DoorProperty {
	public static final DoorProperty STUCK = new DoorProperty(EnterResult.FailureType.STUCK, ForceResult.Status.NEED_TOOL);
	public static final DoorProperty SEALED = new DoorProperty(EnterResult.FailureType.SEALED, ForceResult.Status.NEED_BREAK_TOOL);
	public static final DoorProperty EMPTY = new DoorProperty(null, ForceResult.Status.NOT_STUCK);
	
	private final EnterResult.FailureType entryFailure;
	private final ForceResult.Status forceStatus;
	
	protected DoorProperty(EnterResult.FailureType entryFailure, ForceResult.Status forceStatus) {
		this.entryFailure = entryFailure;
		this.forceStatus = forceStatus;
	}
	
	public EnterResult checkEntry(Aftik aftik) {
		if (entryFailure == null)
			return new EnterResult();
		else
			return new EnterResult(entryFailure);
	}
	
	public final ForceResult.PropertyResult tryForce(Aftik aftik) {
		for (ForceResult.Method method : forceStatus.getAvailableMethods()) {
			if(aftik.hasItem(method.tool())) {
				return ForceResult.success(method);
			}
		}
		return ForceResult.status(forceStatus);
	}
}