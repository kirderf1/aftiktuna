package me.kirderf.aftiktuna.level.object.door;

import me.kirderf.aftiktuna.level.object.Aftik;
import me.kirderf.aftiktuna.level.object.ObjectType;

import java.util.Optional;

public abstract class DoorProperty {
	public static final DoorProperty EMPTY = new DoorProperty() {
		@Override
		public Optional<EnterResult> checkEntry(Aftik aftik) {
			return Optional.of(new EnterResult());
		}
		
		@Override
		public DoorProperty tryForce(Aftik aftik) {
			System.out.println("The door does not seem to be stuck.");
			return this;
		}
	};
	
	public abstract Optional<EnterResult> checkEntry(Aftik aftik);
	
	public static record EnterResult(Optional<ObjectType> usedItem) {
		public EnterResult(ObjectType usedItem) {
			this(Optional.of(usedItem));
		}
		public EnterResult() {
			this(Optional.empty());
		}
	}
	
	public abstract DoorProperty tryForce(Aftik aftik);
}