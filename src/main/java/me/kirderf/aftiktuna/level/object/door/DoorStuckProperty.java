package me.kirderf.aftiktuna.level.object.door;

import me.kirderf.aftiktuna.level.object.Aftik;
import me.kirderf.aftiktuna.level.object.ObjectType;

public final class DoorStuckProperty extends DoorProperty {
	
	public EnterResult checkEntry(Aftik aftik) {
		return new EnterResult(EnterResult.FailureType.STUCK);
	}
	
	public ForceResult tryForce(Aftik aftik) {
		if(aftik.hasItem(ObjectType.CROWBAR)) {
			System.out.println("You use your crowbar to force the door open.");
			return new ForceResult(ObjectType.CROWBAR, ForceResult.Method.FORCE);
		} else if(aftik.hasItem(ObjectType.BLOWTORCH)) {
			System.out.println("You use your blowtorch to cut the door open.");
			return new ForceResult(ObjectType.BLOWTORCH, ForceResult.Method.CUT);
		} else {
			System.out.println("You need some sort of tool to force the door open.");
			return new ForceResult(ForceResult.Status.NEED_TOOL);
		}
	}
}