package me.kirderf.aftiktuna.object.door;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.entity.Aftik;

public final class DoorLockedProperty extends DoorProperty {
	
	@Override
	public EnterResult checkEntry(Aftik aftik) {
		if (aftik.hasItem(ObjectTypes.KEYCARD)) {
			return new EnterResult(ObjectTypes.KEYCARD);
		} else {
			return new EnterResult(EnterResult.FailureType.LOCKED);
		}
	}
	
	@Override
	public ForceResult.PropertyResult tryForce(Aftik aftik) {
		if(aftik.hasItem(ObjectTypes.BLOWTORCH)) {
			return ForceResult.success(ObjectTypes.BLOWTORCH, ForceResult.Method.CUT);
		} else {
			return ForceResult.status(ForceResult.Status.NEED_BREAK_TOOL);
		}
	}
}
