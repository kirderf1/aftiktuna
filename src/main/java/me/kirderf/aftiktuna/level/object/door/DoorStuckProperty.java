package me.kirderf.aftiktuna.level.object.door;

import me.kirderf.aftiktuna.level.object.Aftik;
import me.kirderf.aftiktuna.level.object.ObjectType;

public final class DoorStuckProperty extends DoorProperty {
	
	public EnterResult checkEntry(Aftik aftik) {
		return new EnterResult(EnterResult.FailureType.STUCK);
	}
	
	public ForceResult tryForce(Aftik aftik) {
		if(aftik.hasItem(ObjectType.CROWBAR)) {
			return new ForceResult(ObjectType.CROWBAR, ForceResult.Method.FORCE);
		} else if(aftik.hasItem(ObjectType.BLOWTORCH)) {
			return new ForceResult(ObjectType.BLOWTORCH, ForceResult.Method.CUT);
		} else {
			return new ForceResult(ForceResult.Status.NEED_TOOL);
		}
	}
}