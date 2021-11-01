package me.kirderf.aftiktuna.level.object.door;

import me.kirderf.aftiktuna.level.object.Aftik;
import me.kirderf.aftiktuna.level.object.ObjectType;

import java.util.Optional;

public final class DoorSealedProperty extends DoorProperty {
	@Override
	public Optional<EnterResult> checkEntry(Aftik aftik) {
		System.out.println("The door is sealed shut.");
		return Optional.empty();
	}
	
	@Override
	public DoorProperty tryForce(Aftik aftik) {
		if(aftik.hasItem(ObjectType.BLOWTORCH)) {
			System.out.println("You use your blowtorch to cut the door open.");
			return EMPTY;
		} else {
			System.out.println("You need some sort of tool to break the door open.");
			return this;
		}
	}
}
