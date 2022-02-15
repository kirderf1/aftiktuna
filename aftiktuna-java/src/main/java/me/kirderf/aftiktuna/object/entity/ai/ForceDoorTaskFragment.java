package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.action.ForceDoorAction;
import me.kirderf.aftiktuna.object.Reference;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.door.DoorProperty;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;

import java.util.Optional;

public final class ForceDoorTaskFragment {
	private final Reference<Door> doorRef;
	
	public ForceDoorTaskFragment(Door door) {
		this.doorRef = new Reference<>(door, Door.class);
	}
	
	public boolean performAction(Aftik aftik, ActionPrinter out) {
		Optional<Door> doorOptional = doorRef.find(aftik.getArea());
		if (doorOptional.isPresent() && canForceDoor(aftik, doorOptional.get())) {
			ForceDoorAction.moveAndForce(aftik, doorOptional.get(), out);
			return true;
		} else
			return false;
	}
	
	private boolean canForceDoor(Aftik aftik, Door door) {
		for(DoorProperty.Method method : door.getProperty().relevantForceMethods()) {
			if (aftik.findItem(method::canBeUsedBy).isPresent())
				return true;
		}
		return false;
	}
}
