package me.kirderf.aftiktuna.object.door;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.object.entity.Aftik;

public final class DoorStuckProperty extends DoorProperty {
	public static final DoorProperty INSTANCE = new DoorStuckProperty();
	
	private DoorStuckProperty() {
	}
	
	public EnterResult checkEntry(Aftik aftik) {
		return new EnterResult(EnterResult.FailureType.STUCK);
	}
	
	public ForceResult.PropertyResult tryForce(Aftik aftik) {
		if(aftik.hasItem(ForceResult.Method.FORCE.tool())) {
			return ForceResult.success(ForceResult.Method.FORCE);
		} else if(aftik.hasItem(ForceResult.Method.CUT.tool())) {
			return ForceResult.success(ForceResult.Method.CUT);
		} else {
			return ForceResult.status(ForceResult.Status.NEED_TOOL);
		}
	}
}