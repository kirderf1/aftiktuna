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
			System.out.println("You use your blowtorch to cut the door open.");
			return new ForceResult(ObjectType.BLOWTORCH, ForceResult.Method.CUT);
		} else {
			System.out.println("You need some sort of tool to break the door open.");
			return new ForceResult(ForceResult.Status.NEED_BREAK_TOOL);
		}
	}
}
