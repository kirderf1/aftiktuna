package me.kirderf.aftiktuna.level.object.door;

import me.kirderf.aftiktuna.level.object.Aftik;
import me.kirderf.aftiktuna.level.object.ObjectType;

public final class DoorLockedProperty extends DoorProperty {
	
	@Override
	public EnterResult checkEntry(Aftik aftik) {
		if (aftik.hasItem(ObjectType.KEYCARD)) {
			return new EnterResult(ObjectType.KEYCARD);
		} else {
			return new EnterResult(EnterResult.FailureType.LOCKED);
		}
	}
	
	@Override
	public ForceResult tryForce(Aftik aftik) {
		if(aftik.hasItem(ObjectType.BLOWTORCH)) {
			return new ForceResult(ObjectType.BLOWTORCH, ForceResult.Method.CUT);
		} else {
			return new ForceResult(ForceResult.Status.NEED_BREAK_TOOL);
		}
	}
}
