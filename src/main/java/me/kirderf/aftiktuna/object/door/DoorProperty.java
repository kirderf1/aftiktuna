package me.kirderf.aftiktuna.object.door;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.type.ItemType;
import me.kirderf.aftiktuna.object.type.ObjectTypes;

import java.util.List;
import java.util.Optional;

public final class DoorProperty {
	public static final DoorProperty STUCK = new DoorProperty(new EntryBlockingInfo("stuck"), Status.NEED_TOOL);
	public static final DoorProperty SEALED = new DoorProperty(new EntryBlockingInfo("sealed shut"), Status.NEED_BREAK_TOOL);
	public static final DoorProperty LOCKED = new DoorProperty(new EntryBlockingInfo("locked", ObjectTypes.KEYCARD), Status.NEED_BREAK_TOOL);
	public static final DoorProperty EMPTY = new DoorProperty(null, Status.NOT_STUCK);
	
	private final EntryBlockingInfo entryFailure;
	private final Status forceStatus;
	
	private DoorProperty(EntryBlockingInfo entryFailure, Status forceStatus) {
		this.entryFailure = entryFailure;
		this.forceStatus = forceStatus;
	}
	
	public Optional<String> getAdjective() {
		return Optional.ofNullable(entryFailure).map(EntryBlockingInfo::adjective);
	}
	
	public List<Method> relevantForceMethods() {
		return forceStatus.getAvailableMethods();
	}
	
	public EnterResult checkEntry(Aftik aftik) {
		if (entryFailure == null)
			return EnterResult.success(this);
		else {
			if (aftik.hasItem(entryFailure.itemToPass)) {
				return EnterResult.success(this, entryFailure.itemToPass);
			} else {
				return EnterResult.failure(this, entryFailure.adjective);
			}
		}
	}
	
	public ForceResult.PropertyResult tryForce(Aftik aftik) {
		for (Method method : forceStatus.getAvailableMethods()) {
			Optional<ItemType> toolOptional = aftik.findItem(method::canBeUsedBy);
			if(toolOptional.isPresent()) {
				return ForceResult.success(toolOptional.get());
			}
		}
		return ForceResult.status(forceStatus);
	}
	
	private record EntryBlockingInfo(String adjective, ItemType itemToPass) {
		private EntryBlockingInfo(String adjective) {
			this(adjective, null);
		}
	}
	
	public record Method(String text) {
		public static final Method FORCE = new Method("forced open");
		public static final Method CUT = new Method("cut open");
		
		public boolean canBeUsedBy(ItemType item) {
			return item.getForceMethod() == this;
		}
	}
	
	public enum Status {
		NEED_TOOL(Method.FORCE, Method.CUT),
		NEED_BREAK_TOOL(Method.CUT),
		NOT_STUCK;
		
		private final List<Method> availableMethods;
		
		Status(Method... availableMethods) {
			this.availableMethods = List.of(availableMethods);
		}
		
		public List<Method> getAvailableMethods() {
			return availableMethods;
		}
	}
}