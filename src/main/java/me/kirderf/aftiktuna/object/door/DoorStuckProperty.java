package me.kirderf.aftiktuna.object.door;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.object.entity.Aftik;

public final class DoorStuckProperty extends DoorProperty {
	public static final DoorProperty INSTANCE = new DoorStuckProperty();
	
	private DoorStuckProperty() {
		super(ForceResult.Status.NEED_TOOL);
	}
	
	public EnterResult checkEntry(Aftik aftik) {
		return new EnterResult(EnterResult.FailureType.STUCK);
	}
}