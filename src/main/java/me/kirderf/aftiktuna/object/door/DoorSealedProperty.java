package me.kirderf.aftiktuna.object.door;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.object.entity.Aftik;

public final class DoorSealedProperty extends DoorProperty {
	public static final DoorProperty INSTANCE = new DoorSealedProperty();
	
	private DoorSealedProperty() {
	}
	
	@Override
	public EnterResult checkEntry(Aftik aftik) {
		return new EnterResult(EnterResult.FailureType.SEALED);
	}
	
	@Override
	public ForceResult.PropertyResult tryForce(Aftik aftik) {
		if(aftik.hasItem(ForceResult.Method.CUT.tool())) {
			return ForceResult.success(ForceResult.Method.CUT);
		} else {
			return ForceResult.status(ForceResult.Status.NEED_BREAK_TOOL);
		}
	}
}
