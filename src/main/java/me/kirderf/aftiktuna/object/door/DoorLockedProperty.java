package me.kirderf.aftiktuna.object.door;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.type.ObjectTypes;

public final class DoorLockedProperty extends DoorProperty {
	public static final DoorProperty INSTANCE = new DoorLockedProperty();
	
	private DoorLockedProperty() {
		super(DoorProperty.FailureType.LOCKED, DoorProperty.Status.NEED_BREAK_TOOL);
	}
	
	@Override
	public EnterResult checkEntry(Aftik aftik) {
		if (aftik.hasItem(ObjectTypes.KEYCARD)) {
			return new EnterResult(ObjectTypes.KEYCARD);
		} else {
			return super.checkEntry(aftik);
		}
	}
}
