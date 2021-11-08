package me.kirderf.aftiktuna.level.object.door;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.entity.Aftik;

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
	public ForceResult tryForce(Aftik aftik) {
		if(aftik.hasItem(ObjectTypes.BLOWTORCH)) {
			return new ForceResult(ObjectTypes.BLOWTORCH, ForceResult.Method.CUT);
		} else {
			return new ForceResult(ForceResult.Status.NEED_BREAK_TOOL);
		}
	}
}
