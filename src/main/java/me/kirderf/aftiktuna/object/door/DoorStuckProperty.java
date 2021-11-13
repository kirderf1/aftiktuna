package me.kirderf.aftiktuna.object.door;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.entity.Aftik;

public final class DoorStuckProperty extends DoorProperty {
	
	public EnterResult checkEntry(Aftik aftik) {
		return new EnterResult(EnterResult.FailureType.STUCK);
	}
	
	public ForceResult.PropertyResult tryForce(Aftik aftik) {
		if(aftik.hasItem(ObjectTypes.CROWBAR)) {
			return ForceResult.success(ObjectTypes.CROWBAR, ForceResult.Method.FORCE);
		} else if(aftik.hasItem(ObjectTypes.BLOWTORCH)) {
			return ForceResult.success(ObjectTypes.BLOWTORCH, ForceResult.Method.CUT);
		} else {
			return ForceResult.status(ForceResult.Status.NEED_TOOL);
		}
	}
}