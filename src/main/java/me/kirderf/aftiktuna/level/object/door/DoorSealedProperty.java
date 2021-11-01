package me.kirderf.aftiktuna.level.object.door;

import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.entity.Aftik;

public final class DoorSealedProperty extends DoorProperty {
	@Override
	public EnterResult checkEntry(Aftik aftik) {
		return new EnterResult(EnterResult.FailureType.SEALED);
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
